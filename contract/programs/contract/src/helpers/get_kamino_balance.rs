use anchor_lang::prelude::*;
use anchor_spl::{token_interface::TokenAccount};

use crate::{
    errors::AggregatorError,
    helpers::kamino_account_reader::{
        read_vault_state_fields, read_vault_allocation, read_reserve_fields,
        vault_offsets,
    },
    states::kamino::{Fraction, FractionExtra},
};

/// Kamino VaultState discriminator
const VAULT_STATE_DISCRIMINATOR: [u8; 8] = [228, 196, 82, 165, 98, 210, 235, 152];

/// Scale factor for exchange rate calculations (1e18)
const SCALE_FACTOR_BASE: u128 = 1_000_000_000_000_000_000;
const INITIAL_COLLATERAL_RATE: Fraction = Fraction::ONE;

/// Slots per year for interest calculation (2 slots/sec * 60 * 60 * 24 * 365)
const SLOTS_PER_YEAR: u128 = 63_072_000;

/// Kamino balance calculation using safe byte-offset reading
///
/// In Kamino Vaults:
/// 1. Users deposit tokens and receive shares (kTokens)
/// 2. The vault invests tokens in reserves (lending pools) which earn interest
/// 3. The exchange rate (tokens Per Share) increases as interest accrues
/// 4. User balance = user_shares * tokensPerShare
///
/// tokensPerShare = (available + invested - pendingFees) / sharesIssued
/// where:
/// - available: tokens in the vault not yet invested
/// - invested: sum of all tokens invested in reserves (ctokenAllocation / exchangeRate)
/// - pendingFees: management and performance fees
/// - sharesIssued: total shares minted to all users
pub fn get_kamino_balance<'info>(
    vault_state_account: &'info AccountInfo<'info>,
    user_shares_ktoken: &InterfaceAccount<'info, TokenAccount>,
    reserve_accounts: &[AccountInfo<'info>],
    current_slot: Option<u64>, // If provided, calculates estimated exchange rates with accrued interest
) -> Result<u64> {
    // Load vault state data
    let vault_data = vault_state_account.try_borrow_data()?;

    // Verify discriminator (first 8 bytes)
    if vault_data.len() < 8 || vault_data[0..8] != VAULT_STATE_DISCRIMINATOR {
        return Err(AggregatorError::InvalidAccountData.into());
    }

    // Read only the fields we need using byte offsets
    let vault_fields = read_vault_state_fields(&vault_data)?;

    // Get user's shares from their token account
    let user_shares = user_shares_ktoken.amount;

    // If no shares issued, return 0
    if vault_fields.shares_issued == 0 {
        return Ok(0);
    }

    // Calculate total invested amount from all reserve allocations
    let total_invested = calculate_total_invested_with_exchange_rate(
        &vault_data,
        reserve_accounts,
        current_slot,
    )?;

    // Calculate tokens per share
    // tokensPerShare = (available + invested - pendingFees) / sharesIssued

    // Get pending_fees (already scaled by 2^60)
    let pending_fees = Fraction::from_bits(vault_fields.pending_fees_sf);

    // Calculate total AUM: available + invested
    let total_aum = Fraction::from(vault_fields.token_available)
        .checked_add(total_invested)
        .ok_or(AggregatorError::MathOverflow)?;

    // Calculate net AUM: total_aum - pending_fees
    let net_aum = total_aum
        .checked_sub(pending_fees)
        .ok_or(AggregatorError::MathOverflow)?;

    // Calculate user balance: (user_shares * net_aum) / shares_issued
    let user_balance = (Fraction::from(user_shares)
        .checked_mul(net_aum)
        .ok_or(AggregatorError::MathOverflow)?)
        .checked_div(Fraction::from(vault_fields.shares_issued))
        .ok_or(AggregatorError::MathOverflow)?;
    
    user_balance.try_to_floor::<u64>().ok_or(AggregatorError::MathOverflow.into())
}

/// Calculate the total amount invested in reserves, applying the proper exchange rate from each reserve.
/// Uses safe byte-offset reading instead of zero-copy deserialization.
fn calculate_total_invested_with_exchange_rate(
    vault_data: &[u8],
    reserve_accounts: &[AccountInfo],
    current_slot: Option<u64>,
) -> Result<Fraction> {
    let mut total_invested = Fraction::ZERO;

    // Iterate through all possible allocations (max 25)
    for i in 0..vault_offsets::MAX_RESERVES {
        let allocation = read_vault_allocation(vault_data, i)?;
        
        // Skip empty allocations
        if allocation.reserve == Pubkey::default() {
            continue;
        }

        let ctoken_allocation_f = Fraction::from(allocation.ctoken_allocation);
        if ctoken_allocation_f == Fraction::ZERO {
            continue;
        }

        // Find the corresponding reserve account
        let reserve_account = reserve_accounts
            .iter()
            .find(|acc| acc.key() == allocation.reserve);

        let exchange_rate = match reserve_account {
            Some(acc) => calculate_collateral_exchange_rate(acc, current_slot)?,
            None => INITIAL_COLLATERAL_RATE, // Fallback to 1.0 if reserve account not provided
        };

        // invested = (ctoken_allocation * SCALE_FACTOR_BASE) / exchange_rate
        let invested_amount = (ctoken_allocation_f
            .checked_mul(Fraction::from_sf(SCALE_FACTOR_BASE))
            .ok_or(AggregatorError::MathOverflow)?)
            .checked_div(exchange_rate)
            .ok_or(AggregatorError::MathOverflow)?;

        total_invested = total_invested
            .checked_add(invested_amount)
            .ok_or(AggregatorError::MathOverflow)?;
    }

    Ok(total_invested)
}


pub fn get_kamino_shares_amount_from_usdc<'info>(
    usdc_amount: u64,
    vault_state_account: &'info AccountInfo<'info>,
    user_shares_ktoken: &InterfaceAccount<'info, TokenAccount>,
    reserve_accounts: &[AccountInfo<'info>],
    current_slot: Option<u64>,
) -> Result<u64> {
    let kamino_balance = get_kamino_balance(vault_state_account, user_shares_ktoken, reserve_accounts, current_slot)?;

    if kamino_balance == 0 {
        return Ok(0);
    }

    let user_shares = user_shares_ktoken.amount;

    // shares_amount = (user_shares * usdc_amount) / kamino_balance

    let shares_amount = (Fraction::from(user_shares)
        .checked_mul(Fraction::from(usdc_amount))
        .ok_or(AggregatorError::MathOverflow)?)
        .checked_div(Fraction::from(kamino_balance))
        .ok_or(AggregatorError::MathOverflow)?;

    shares_amount.try_to_floor::<u64>().ok_or(AggregatorError::MathOverflow.into())

}

/// Calculate collateral exchange rate from a reserve using safe byte-offset reading
///
/// Formula from Kamino Reserve:
/// - If current_slot provided: estimates with compounded interest
/// - total_supply = available_amount + borrowed_amount_sf - accumulated_protocol_fees_sf
///                  - accumulated_referrer_fees_sf - pending_referrer_fees_sf
/// - exchangeRate = mint_total_supply / total_supply (if both non-zero, else 1.0)
///
/// All *_sf fields use Fraction type (2^60 scaling)
fn calculate_collateral_exchange_rate(
    reserve_account: &AccountInfo,
    current_slot: Option<u64>,
) -> Result<Fraction> {
    let reserve_data = reserve_account.try_borrow_data()?;

    // Minimum size check
    if reserve_data.len() < 100 {
        return Ok(INITIAL_COLLATERAL_RATE);
    }

    // Read only the fields we need using byte offsets
    let reserve = read_reserve_fields(&reserve_data)?;

    let mint_total_supply = Fraction::from(reserve.mint_total_supply);

    // Calculate total supply (with or without estimated interest)
    let total_supply = if let Some(slot) = current_slot {
        // Calculate estimated total supply with compounded interest
        calculate_estimated_total_supply(&reserve, slot)?
    } else {
        // Use stale total supply from last update
        calculate_total_supply(&reserve)?
    };

    // If either is zero, return initial rate (1.0)
    if mint_total_supply == Fraction::ZERO || total_supply == Fraction::ZERO {
        return Ok(INITIAL_COLLATERAL_RATE);
    }

    // Calculate exchange rate: (mintTotalSupply * SCALE_FACTOR) / totalSupply
    // This matches Kamino's CollateralExchangeRate::from_supply_and_liquidity()
    let exchange_rate = (mint_total_supply
        .checked_mul(Fraction::from_sf(SCALE_FACTOR_BASE))
        .ok_or(AggregatorError::MathOverflow)?)
        .checked_div(total_supply)
        .ok_or(AggregatorError::MathOverflow)?;

    Ok(exchange_rate)
}

/// Calculate stale total supply from reserve (without compounding interest)
fn calculate_total_supply(
    reserve: &crate::helpers::kamino_account_reader::ReserveFields
) -> Result<Fraction> {
    let available_amount = Fraction::from(reserve.available_amount);
    let borrowed_amount = Fraction::from_bits(reserve.borrowed_amount_sf);
    let accumulated_protocol_fees = Fraction::from_bits(reserve.accumulated_protocol_fees_sf);
    let accumulated_referrer_fees = Fraction::from_bits(reserve.accumulated_referrer_fees_sf);
    let pending_referrer_fees = Fraction::from_bits(reserve.pending_referrer_fees_sf);

    // total_supply = available_amount + borrowed_amount - all fees
    let total_supply = available_amount
        .checked_add(borrowed_amount)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_sub(accumulated_protocol_fees)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_sub(accumulated_referrer_fees)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_sub(pending_referrer_fees)
        .ok_or(AggregatorError::MathOverflow)?;

    Ok(total_supply)
}

/// Calculate estimated total supply with compounded interest
/// This implements the logic from KaminoReserve.getEstimatedTotalSupply()
fn calculate_estimated_total_supply(
    reserve: &crate::helpers::kamino_account_reader::ReserveFields,
    current_slot: u64
) -> Result<Fraction> {
    let slots_elapsed = current_slot.saturating_sub(reserve.last_update_slot);

    // If no slots elapsed, return stale total supply
    if slots_elapsed == 0 {
        return calculate_total_supply(reserve);
    }

    // Compound interest to estimate new debt
    let (new_debt, new_acc_protocol_fees, pending_referral_fees) =
        compound_interest(reserve, slots_elapsed)?;

    // Calculate estimated total supply:
    // total_supply = available_amount + new_debt - new_acc_protocol_fees
    //                - accumulated_referrer_fees - pending_referral_fees
    let available_amount = Fraction::from(reserve.available_amount);
    let accumulated_referrer_fees = Fraction::from_bits(reserve.accumulated_referrer_fees_sf);

    let total_supply = available_amount
        .checked_add(new_debt)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_sub(new_acc_protocol_fees)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_sub(accumulated_referrer_fees)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_sub(pending_referral_fees)
        .ok_or(AggregatorError::MathOverflow)?;

    Ok(total_supply)
}

/// Compound interest calculation to estimate new debt and fees
/// Implements the logic from KaminoReserve.compoundInterest()
fn compound_interest(
    reserve: &crate::helpers::kamino_account_reader::ReserveFields,
    slots_elapsed: u64,
) -> Result<(Fraction, Fraction, Fraction)> {
    let previous_debt = Fraction::from_bits(reserve.borrowed_amount_sf);

    // Convert to Fraction
    let protocol_take_rate = Fraction::from_percent(reserve.protocol_take_rate_pct as u64);
    let fixed_host_interest_rate = Fraction::from_bps(reserve.host_fixed_interest_rate_bps as u64);

    // For now, use simplified interest calculation
    // TODO: Implement full borrow curve logic if needed
    // We'll approximate with fixed rate for now
    
    let approximate_borrow_rate = Fraction::from_bps(500); // ~5% APY as placeholder

    // Approximate compounded interest
    let compounded_interest_rate = approximate_compounded_interest(
        approximate_borrow_rate
            .checked_add(fixed_host_interest_rate)
            .ok_or(AggregatorError::MathOverflow)?,
        slots_elapsed,
    )?;

    let compounded_fixed_rate = approximate_compounded_interest(
        fixed_host_interest_rate,
        slots_elapsed,
    )?;

    // Calculate new debt
    let new_debt = previous_debt
        .checked_mul(compounded_interest_rate)
        .ok_or(AggregatorError::MathOverflow)?;

    // Calculate fixed host fee
    let fixed_host_fee = (previous_debt
        .checked_mul(compounded_fixed_rate)
        .ok_or(AggregatorError::MathOverflow)?)
        .checked_sub(previous_debt)
        .ok_or(AggregatorError::MathOverflow)?;

    // Calculate net new variable debt
    let net_new_variable_debt = new_debt
        .checked_sub(previous_debt)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_sub(fixed_host_fee)
        .ok_or(AggregatorError::MathOverflow)?;

    // Calculate protocol fees
    let variable_protocol_fee = net_new_variable_debt
        .checked_mul(protocol_take_rate)
        .ok_or(AggregatorError::MathOverflow)?;

    // Calculate referral fees (assuming 0% referral for estimation)
    let max_referral_fees = Fraction::ZERO;

    // Calculate new accumulated protocol fees
    let acc_protocol_fees_f = Fraction::from_bits(reserve.accumulated_protocol_fees_sf);
    let new_acc_protocol_fees = acc_protocol_fees_f
        .checked_add(fixed_host_fee)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_add(variable_protocol_fee)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_sub(max_referral_fees)
        .ok_or(AggregatorError::MathOverflow)?;

    // Calculate pending referral fees
    let pending_referrer_fees_f = Fraction::from_bits(reserve.pending_referrer_fees_sf);
    let pending_referral_fees = pending_referrer_fees_f
        .checked_add(max_referral_fees)
        .ok_or(AggregatorError::MathOverflow)?;

    Ok((new_debt, new_acc_protocol_fees, pending_referral_fees))
}

/// Approximate compounded interest over elapsed slots
/// Implements the Taylor series approximation from Kamino
fn approximate_compounded_interest(rate: Fraction, elapsed_slots: u64) -> Result<Fraction> {
    let base = rate
        .checked_div(Fraction::from_num(SLOTS_PER_YEAR))
        .ok_or(AggregatorError::MathOverflow)?;

    // Handle special cases for efficiency
    match elapsed_slots {
        0 => return Ok(Fraction::ONE),
        1 => return Ok(Fraction::ONE.checked_add(base).ok_or(AggregatorError::MathOverflow)?),
        2 => {
            let one_plus_base = Fraction::ONE.checked_add(base).ok_or(AggregatorError::MathOverflow)?;
            return one_plus_base.checked_mul(one_plus_base).ok_or(AggregatorError::MathOverflow.into());
        }
        3 => {
            let one_plus_base = Fraction::ONE.checked_add(base).ok_or(AggregatorError::MathOverflow)?;
            return one_plus_base
                .checked_mul(one_plus_base)
                .ok_or(AggregatorError::MathOverflow)?
                .checked_mul(one_plus_base)
                .ok_or(AggregatorError::MathOverflow.into());
        }
        4 => {
            let one_plus_base = Fraction::ONE.checked_add(base).ok_or(AggregatorError::MathOverflow)?;
            let pow_two = one_plus_base.checked_mul(one_plus_base).ok_or(AggregatorError::MathOverflow)?;
            return pow_two.checked_mul(pow_two).ok_or(AggregatorError::MathOverflow.into());
        }
        _ => {}
    }

    // Taylor series approximation for larger elapsed_slots:
    // (1 + base)^elapsed_slots ≈ 1 + base*elapsed_slots + (base^2 * elapsed_slots * (elapsed_slots-1))/2
    //                              + (base^3 * elapsed_slots * (elapsed_slots-1) * (elapsed_slots-2))/6

    let exp = elapsed_slots;
    let exp_minus_one = exp.saturating_sub(1);
    let exp_minus_two = exp.saturating_sub(2);

    let base_power_two = base.checked_mul(base).ok_or(AggregatorError::MathOverflow)?;
    let base_power_three = base_power_two.checked_mul(base).ok_or(AggregatorError::MathOverflow)?;

    // First term: base * exp
    let first_term = base.checked_mul(Fraction::from_num(exp)).ok_or(AggregatorError::MathOverflow)?;

    // Second term: (base^2 * exp * (exp-1)) / 2
    let second_term = (base_power_two
        .checked_mul(Fraction::from_num(exp))
        .ok_or(AggregatorError::MathOverflow)?
        .checked_mul(Fraction::from_num(exp_minus_one))
        .ok_or(AggregatorError::MathOverflow)?)
        .checked_div(Fraction::from_num(2))
        .ok_or(AggregatorError::MathOverflow)?;

    // Third term: (base^3 * exp * (exp-1) * (exp-2)) / 6
    let third_term = (base_power_three
        .checked_mul(Fraction::from_num(exp))
        .ok_or(AggregatorError::MathOverflow)?
        .checked_mul(Fraction::from_num(exp_minus_one))
        .ok_or(AggregatorError::MathOverflow)?
        .checked_mul(Fraction::from_num(exp_minus_two))
        .ok_or(AggregatorError::MathOverflow)?)
        .checked_div(Fraction::from_num(6))
        .ok_or(AggregatorError::MathOverflow)?;

    // Result: 1 + first_term + second_term + third_term
    Ok(Fraction::ONE
        .checked_add(first_term)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_add(second_term)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_add(third_term)
        .ok_or(AggregatorError::MathOverflow)?)
}
