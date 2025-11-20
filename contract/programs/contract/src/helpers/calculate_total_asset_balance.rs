use std::slice::Iter;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount};

use crate::{ 
    errors::AggregatorError, 
    helpers::{
        juplend::get_juplend_balance::get_juplend_balance, 
        kamino::{
            get_kamino_balance::get_kamino_balance, 
            get_kamino_farm_active_balance
        }
    }, 
    states::{
        ReserveWithdrawAccounts, 
        lending::Lending, 
        lending_rewards_rate_model::LendingRewardsRateModel
    }
};


pub fn skip_accounts<'info>(account_iter: &mut Iter<'info, AccountInfo<'info>>, n: usize) -> Result<()> {
    for _ in 0..n {
        let _ = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
    }
    Ok(())
}


pub fn calculate_total_asset_balance<'info>(
    remaining_accounts: &'info [AccountInfo<'info>]
) -> Result<Vec<u64>> {
    
    msg!("calculate_total_asset_balance: Starting with {} remaining accounts", remaining_accounts.len());

    let mut account_iter = remaining_accounts.iter();
    

    let jup_lending_acc = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
   
    let jup_lending = Lending::try_deserialize(&mut &jup_lending_acc.try_borrow_data()?[..])?;
    
    let jup_rewards_acc = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
    let jup_lending_rewards_rate_model = LendingRewardsRateModel::try_deserialize(&mut &jup_rewards_acc.try_borrow_data()?[..])?;
    
    skip_accounts(&mut account_iter, 1)?;
    let jup_vault_ftokens = InterfaceAccount::<'info, TokenAccount>::try_from(account_iter.next().ok_or(AggregatorError::MissingAccount)?)?;
    
    skip_accounts(&mut account_iter, 1)?;
    let jup_supply_token_reserves_liquidity = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
    
    skip_accounts(&mut account_iter, 7)?;
    
    let kamino_vault_state = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
    
    skip_accounts(&mut account_iter, 16)?;
    
    let kamino_user_state = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
    
    let kamino_user_shares_ata = InterfaceAccount::<'info, TokenAccount>::try_from(account_iter.next().ok_or(AggregatorError::MissingAccount)?)?;
    
    // Reserve 1 (7 accounts)
    let reserve_1 = ReserveWithdrawAccounts {
        reserve: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        ctoken_vault: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        lending_market: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        lending_market_authority: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        reserve_liquidity_supply: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        reserve_collateral_mint: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        reserve_collateral_token_program: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
    };
    
    // Reserve 2 (7 accounts)
    let reserve_2 = ReserveWithdrawAccounts {
        reserve: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        ctoken_vault: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        lending_market: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        lending_market_authority: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        reserve_liquidity_supply: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        reserve_collateral_mint: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        reserve_collateral_token_program: account_iter.next().ok_or(AggregatorError::MissingAccount)?.to_account_info(),
    };

    let juplend_balance = get_juplend_balance(
        jup_supply_token_reserves_liquidity,
        &jup_lending,
        &jup_lending_rewards_rate_model,
        &jup_vault_ftokens,
    )?;

    msg!("Calculated Juplend balance: {}", juplend_balance);

    let current_slot = Clock::get()?.slot;

    let kamino_farm_active_balance = get_kamino_farm_active_balance(
        &kamino_user_shares_ata,
        &kamino_user_state,
    )?;

    let kamino_balance = get_kamino_balance(
        kamino_vault_state,
        kamino_farm_active_balance,
        &[reserve_1.reserve.clone(), reserve_2.reserve.clone()],
        Some(current_slot),
    )?;

    

    msg!("Calculated Kamino balance: {}", kamino_balance);
    msg!("Calculated Kamino farm active balance: {}", kamino_farm_active_balance);

    Ok(vec![
        juplend_balance,
        kamino_balance,
    ])
}