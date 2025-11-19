use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{errors::AggregatorError, helpers::{calculate_total_asset_balance::calculate_total_asset_balance, deposit_to_juplend::Juplend, deposit_to_kamino::KaminoVault}, states::{AggregatorConfig, ReserveWithdrawAccounts}};





pub fn withdraw_from_protocols<'c, 'info>(
    usdc_to_withdraw: u64,
    signer: &Signer<'info>,
    remaining_accounts: &'info [AccountInfo<'info>],
    config: Account<'info, AggregatorConfig>,
    vault_usdc: InterfaceAccount<'info, TokenAccount>,
    usdc_mint: InterfaceAccount<'info, Mint>,
    token_program: Interface<'info, TokenInterface>,
    associated_token_program: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    rent: AccountInfo<'info>,
) -> Result<()>
{
    // 1. withdraw from JupLend first (in juplend we pass usdc_amount)
    // 2. If still insufficient, withdraw from Kamino (we pass shares_amount)

    let usdc_balance_accross_protocols = calculate_total_asset_balance(remaining_accounts)?;
    let total_usdc_in_protocols_combined: u64 = usdc_balance_accross_protocols
        .iter()
        .try_fold(0u64, |acc, x| acc.checked_add(*x))
        .ok_or(AggregatorError::MathOverflow)?;

    if usdc_to_withdraw > total_usdc_in_protocols_combined {
        return Err(AggregatorError::InsufficientBalance.into());
    }


    let juplend_balance = usdc_balance_accross_protocols[0];
    let kamino_balance = usdc_balance_accross_protocols[1];

    let juplend_accounts = Juplend::new(
        &config,
        remaining_accounts,
        &vault_usdc,
        &usdc_mint,
        &token_program,
        &associated_token_program,
        &system_program,
    );

    let kamino_accounts = KaminoVault::new(
        &signer,
        &config,
        remaining_accounts,
        &vault_usdc,
        &usdc_mint,
        &token_program,
        &associated_token_program,
        &system_program,
        &rent,
    );


    let kamino_user_shares_ata_account_info = InterfaceAccount::<TokenAccount>::try_from(&remaining_accounts[17])?;
    let kamino_vault_state_account_info = &remaining_accounts[13];
    let temp_reserve_accounts : Vec<ReserveWithdrawAccounts<'info>> = kamino_accounts.reserve_accounts.clone();
    let reserve_accounts: Vec<AccountInfo<'info>> = temp_reserve_accounts.iter().map(|x| x.reserve.clone()).collect();

    let current_slot = Clock::get()?.slot;


    if juplend_balance >= usdc_to_withdraw {
        juplend_accounts.withdraw_from_juplend(usdc_to_withdraw, config.bump)?;
    } else {
        if kamino_balance >=usdc_to_withdraw {
        
            kamino_accounts.withdraw_from_kamino_by_shares(
                &kamino_user_shares_ata_account_info,
                kamino_vault_state_account_info,
                &reserve_accounts,
                current_slot,
                usdc_to_withdraw,
                config.bump,
            )?;
        }
        else {
            
            juplend_accounts.withdraw_from_juplend(juplend_balance, config.bump)?;
            
            kamino_accounts.withdraw_from_kamino_by_shares(
                &kamino_user_shares_ata_account_info,
                kamino_vault_state_account_info,
                &reserve_accounts,
                current_slot,
                usdc_to_withdraw - juplend_balance,
                config.bump,
            )?;
        }
    }

    Ok(())
}


