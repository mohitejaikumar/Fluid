use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::{ helpers::{get_juplend_balance, get_kamino_balance}, states::{ReserveWithdrawAccounts, lending::Lending, lending_rewards_rate_model::LendingRewardsRateModel}};



    // Juplend Protocol accounts

    // these are all remaining accounts for juplend

    // pub lending: Account<'info, Lending>,
    // pub rewards_rate_model: Account<'info, LendingRewardsRateModel>,
    // pub f_token_mint: InterfaceAccount<'info, Mint>,

    // pub vault_ftokens: AccountInfo<'info>,
    // pub lending_admin: AccountInfo<'info>,
    // pub supply_token_reserves_liquidity: AccountInfo<'info>,
    // pub lending_supply_position_on_liquidity: AccountInfo<'info>,
    // pub rate_model: AccountInfo<'info>,
    // pub vault: AccountInfo<'info>,
    // pub liquidity: AccountInfo<'info>,
    // pub liquidity_program: AccountInfo<'info>,
    // pub claim_account: AccountInfo<'info>,
    // pub lending_program: UncheckedAccount<'info>,


    // Kamino Protocol accounts

    // pub vault_state: AccountInfo<'info>,
    // pub token_vault: AccountInfo<'info>,
    // pub base_vault_authority: AccountInfo<'info>,
    // pub shares_mint: AccountInfo<'info>,

    // pub user_shares_ata: AccountInfo<'info>,
    // pub klend_program: AccountInfo<'info>,
    // pub shares_token_program: AccountInfo<'info>,
    // pub event_authority: AccountInfo<'info>,

    // pub kamino_lending_vault_program: AccountInfo<'info>,
    // pub user_state: AccountInfo<'info>,
    // pub farm_state: AccountInfo<'info>,
    // pub farm_vault: AccountInfo<'info>,

    // pub scope_prices: AccountInfo<'info>,
    // pub farm_program: AccountInfo<'info>,
    // pub kamino_vault_program: AccountInfo<'info>,

    // pub farm_vault_authority: AccountInfo<'info>,

    // pub reserve_account_1: AccountInfo<'info>,
    // pub reserve_account_2: AccountInfo<'info>,

    // pub lending_market_1: AccountInfo<'info>,
    // pub lending_market_2: AccountInfo<'info>,
    

    
    

pub fn calculate_total_asset_balance<'info>(
    remaining_accounts: &'info [AccountInfo<'info>]
) -> Result<Vec<u64>> {
    

    let mut account_iter = remaining_accounts.iter();
    
    // Deserialize JupLend accounts without ownership checks (they're owned by JupLend program, not our program)
    let jup_lending_acc = account_iter.next().unwrap();
    let jup_lending = Lending::try_deserialize(&mut &jup_lending_acc.try_borrow_data()?[..])?;
    
    let jup_rewards_acc = account_iter.next().unwrap();
    let jup_lending_rewards_rate_model = LendingRewardsRateModel::try_deserialize(&mut &jup_rewards_acc.try_borrow_data()?[..])?;
    
    let _jup_f_token_mint = InterfaceAccount::<'info, Mint>::try_from(account_iter.next().unwrap())?;
    let jup_vault_ftokens = InterfaceAccount::<'info, TokenAccount>::try_from(account_iter.next().unwrap())?;
    let _jup_lending_admin = account_iter.next().unwrap();
    let jup_supply_token_reserves_liquidity = account_iter.next().unwrap();
    let _jup_lending_supply_position_on_liquidity = account_iter.next().unwrap();
    let _jup_rate_model = account_iter.next().unwrap();
    let _jup_vault = account_iter.next().unwrap();
    let _jup_liquidity = account_iter.next().unwrap();
    let _jup_liquidity_program = account_iter.next().unwrap();
    let _jup_claim_account = account_iter.next().unwrap();
    let _jup_lending_program = account_iter.next().unwrap();
    
    let kamino_vault_state = account_iter.next().unwrap();
    let _kamino_token_vault = account_iter.next().unwrap();
    let _kamino_base_vault_authority = account_iter.next().unwrap();
    let _kamino_shares_mint = account_iter.next().unwrap();
    let kamino_user_shares_ata = InterfaceAccount::<'info, TokenAccount>::try_from(account_iter.next().unwrap())?;
    let _kamino_klend_program = account_iter.next().unwrap();
    let _kamino_shares_token_program = account_iter.next().unwrap();
    let _kamino_event_authority = account_iter.next().unwrap();
    let _kamino_kamino_lending_vault_program = account_iter.next().unwrap();
    let _kamino_user_state = account_iter.next().unwrap();
    let _kamino_farm_state = account_iter.next().unwrap();
    let _kamino_farm_vault = account_iter.next().unwrap();
    let _kamino_scope_prices = account_iter.next().unwrap();
    let _kamino_farm_program = account_iter.next().unwrap();
    let _kamino_vault_program = account_iter.next().unwrap();
    let _kamino_farm_vault_authority = account_iter.next().unwrap();

    let _instruction_sysvar = account_iter.next().unwrap();

    // Reserve 1 (7 accounts)
    let reserve_1 = ReserveWithdrawAccounts {
        reserve: account_iter.next().unwrap().to_account_info(),
        ctoken_vault: account_iter.next().unwrap().to_account_info(),
        lending_market: account_iter.next().unwrap().to_account_info(),
        lending_market_authority: account_iter.next().unwrap().to_account_info(),
        reserve_liquidity_supply: account_iter.next().unwrap().to_account_info(),
        reserve_collateral_mint: account_iter.next().unwrap().to_account_info(),
        reserve_collateral_token_program: account_iter.next().unwrap().to_account_info(),
    };
    
    // Reserve 2 (7 accounts)
    let reserve_2 = ReserveWithdrawAccounts {
        reserve: account_iter.next().unwrap().to_account_info(),
        ctoken_vault: account_iter.next().unwrap().to_account_info(),
        lending_market: account_iter.next().unwrap().to_account_info(),
        lending_market_authority: account_iter.next().unwrap().to_account_info(),
        reserve_liquidity_supply: account_iter.next().unwrap().to_account_info(),
        reserve_collateral_mint: account_iter.next().unwrap().to_account_info(),
        reserve_collateral_token_program: account_iter.next().unwrap().to_account_info(),
    };

    msg!("Calculating Juplend balance");

    let juplend_balance = get_juplend_balance(
        jup_supply_token_reserves_liquidity,
        &jup_lending,
        &jup_lending_rewards_rate_model,
        &jup_vault_ftokens,
    )?;

    msg!("Calculated Juplend balance: {}", juplend_balance);

    let current_slot = Clock::get()?.slot;

    msg!("Calculating Kamino balance");

    let kamino_balance = get_kamino_balance(
        kamino_vault_state,
        &kamino_user_shares_ata,
        &[reserve_1.reserve.clone(), reserve_2.reserve.clone()],
        Some(current_slot),
    )?;

    msg!("Calculated Kamino balance: {}", kamino_balance);

    Ok(vec![
        juplend_balance,
        kamino_balance,
    ])
}