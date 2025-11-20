use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount};

use crate::{constants::WAD, errors::AggregatorError, helpers::kamino::user_state_helper};


/// including both unstaked shares (in token account) and staked shares (in farm).

pub fn get_kamino_farm_active_balance<'info>(
    user_shares_ktoken: &InterfaceAccount<'info, TokenAccount>,
    user_state_account: &AccountInfo<'info>,
) -> Result<u64> {
    let unstaked_shares = user_shares_ktoken.amount;
    
    // Get staked shares from the farm user state (if exists)
    let staked_shares = if user_state_account.data_len() > 0 {
        
        let user_state_data = user_state_account.try_borrow_data()?;
        
        
        let discriminator = user_state_helper::get_discriminator(&user_state_data);
        if discriminator != user_state_helper::USER_STATE_DISCRIMINATOR {
            0
        } else {
            // This is scaled by WAD (10^18), so we need to scale it down
            let active_stake_scaled = user_state_helper::get_active_stake_scaled(&user_state_data);
            
            (active_stake_scaled / WAD) as u64
        }
    } else {
        0
    };
    
    // Calculate total shares: unstaked + staked
    let total_shares = unstaked_shares
        .checked_add(staked_shares)
        .ok_or(AggregatorError::MathOverflow)?;
    
    Ok(total_shares)
}


