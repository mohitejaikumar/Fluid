use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::BPS_BASE, errors::AggregatorError, events::RebalanceEvent, helpers::{deposit_to_juplend::Juplend, deposit_to_kamino::KaminoVault, get_kamino_balance::get_kamino_shares_amount_from_usdc}, states::aggregator_config::AggregatorConfig};




pub fn rebalance_allocation<'info>(
    signer: &Signer<'info>,
    remaining_accounts: &'info [AccountInfo<'info>],
    usdc_in_all_protocol: Vec<u64>,
    config: &Account<'info, AggregatorConfig>,
    vault_usdc: &InterfaceAccount<'info, TokenAccount>,
    usdc_mint: &InterfaceAccount<'info, Mint>,
    token_program: &Interface<'info, TokenInterface>,
    associated_token_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    rent: &AccountInfo<'info>,
) -> Result<()> {
        
    // TODO: Implement
    // 1. Calculate allocation split based on config.juplend_allocation_bps
    // 2. Call deposit_to_juplend for JupLend portion
    // 3. Call deposit_to_kamino for Kamino portion

    // TODO: 
    let mut total_usdc_in_all_protocols_combined  = usdc_in_all_protocol
            .iter()
            .try_fold(0u64, |acc, x| acc.checked_add(*x))
            .ok_or(AggregatorError::MathOverflow)?;

    total_usdc_in_all_protocols_combined = total_usdc_in_all_protocols_combined
            .checked_add(vault_usdc.amount)
            .ok_or(AggregatorError::MathOverflow)?;

    let target_juplend_balance = total_usdc_in_all_protocols_combined
            .checked_mul(config.juplend_allocation_bps as u64)
            .ok_or(AggregatorError::MathOverflow)?
            .checked_div(BPS_BASE as u64)
            .ok_or(AggregatorError::MathOverflow)?;

    let target_kamino_balance = total_usdc_in_all_protocols_combined
            .checked_sub(target_juplend_balance)
            .ok_or(AggregatorError::MathOverflow)?;

    

    let juplend_accounts = Juplend::new(
        config,
        remaining_accounts,
        vault_usdc,
        usdc_mint,
        token_program,
        associated_token_program,
        system_program,
    );
    let kamino_accounts = KaminoVault::new(
        signer,
        config,
        remaining_accounts,
        vault_usdc,
        usdc_mint,
        token_program,
        associated_token_program,
        system_program,
        rent,
    );

    msg!("Juplend balance: {}", target_juplend_balance);
    msg!("Kamino balance: {}", target_kamino_balance);

    execute_rebalance(
        &juplend_accounts,
        &kamino_accounts,
        vault_usdc,
        remaining_accounts,
        usdc_in_all_protocol[0],
        usdc_in_all_protocol[1],
        target_juplend_balance,
        target_kamino_balance,
        config.bump,
    )?;

    

    emit!(RebalanceEvent {
        juplend_balance: target_juplend_balance,
        kamino_balance: target_kamino_balance,
    });

    Ok(())
}


fn execute_rebalance<'info>(
    juplend_accounts: &Juplend<'info>,
    kamino_accounts: &KaminoVault<'info>,
    vault_usdc: &InterfaceAccount<'info, TokenAccount>,
    remaining_accounts: &'info [AccountInfo<'info>],
    current_juplend: u64,
    current_kamino: u64,
    target_juplend: u64,
    target_kamino: u64,
    config_bump: u8,
) -> Result<()> {

    // Step 1: If there's USDC in vault, deposit it to Juplend first
    let vault_balance = vault_usdc.amount;
    if vault_balance > 0 {
        msg!("Depositing to Juplend: {}", vault_balance);
        juplend_accounts.deposit_to_juplend(vault_balance, config_bump)?;
    }
    msg!("Vault balance: {}", vault_balance);
    // Step 2: Calculate new balances after depositing vault USDC to Juplend
    let new_juplend_balance = current_juplend
        .checked_add(vault_balance)
        .ok_or(AggregatorError::MathOverflow)?;
    let new_kamino_balance = current_kamino;
    msg!("New Juplend balance: {}", new_juplend_balance);
    msg!("New Kamino balance: {}", new_kamino_balance);
    
    // Step 4: Determine which protocol has excess and rebalance
    if new_juplend_balance > target_juplend {
        // Juplend has excess, move to Kamino
        let amount_to_move = new_juplend_balance
            .checked_sub(target_juplend)
            .ok_or(AggregatorError::MathOverflow)?;


        msg!("Withdrawing from Juplend: {}", amount_to_move);
        juplend_accounts.withdraw_from_juplend(amount_to_move, config_bump)?;
        msg!("✅ JupLend withdrawal done. Depositing to Kamino: {}", amount_to_move);

        kamino_accounts.execute_complete_deposit(amount_to_move, config_bump)?;

        
    } else if new_kamino_balance > target_kamino {
        // Kamino has excess, move to Juplend
        let amount_to_move = new_kamino_balance
            .checked_sub(target_kamino)
            .ok_or(AggregatorError::MathOverflow)?;
        
        // Get Kamino shares to withdraw
        let current_slot = Clock::get()?.slot;
        let kamino_user_shares_ata_account_info = InterfaceAccount::<TokenAccount>::try_from(&remaining_accounts[17])?;
        let kamino_vault_state_account_info = &remaining_accounts[13];

        let reserve_accounts: Vec<AccountInfo<'info>> = kamino_accounts.reserve_accounts
        .iter()
        .map(|x| x.reserve.clone())
        .collect();

        
        let shares_amount = get_kamino_shares_amount_from_usdc(
            amount_to_move,
            kamino_vault_state_account_info,
            &kamino_user_shares_ata_account_info,
            reserve_accounts.as_slice(),
            Some(current_slot),
        )?;

        msg!("Withdrawing from Kamino: {}", shares_amount);
       
        kamino_accounts.execute_complete_withdraw(
            kamino_user_shares_ata_account_info,
            shares_amount,
            config_bump,
        )?;
        
        msg!("Depositing to Juplend: {}", amount_to_move);
        juplend_accounts.deposit_to_juplend(amount_to_move, config_bump)?;
    }

    Ok(())
}