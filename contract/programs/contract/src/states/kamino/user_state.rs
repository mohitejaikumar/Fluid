use anchor_lang::prelude::*;

use crate::constants::MAX_REWARDS_TOKENS;




#[derive(Debug, Eq, PartialEq)]
pub struct UserState {
    pub user_id: u64,
    pub farm_state: Pubkey,
    pub owner: Pubkey,


    pub is_farm_delegated: u8,
    pub _padding_0: [u8; 7],



    pub rewards_tally_scaled: [u128; MAX_REWARDS_TOKENS],

    pub rewards_issued_unclaimed: [u64; MAX_REWARDS_TOKENS],
    pub last_claim_ts: [u64; MAX_REWARDS_TOKENS],



    pub active_stake_scaled: u128,



    pub pending_deposit_stake_scaled: u128,


    pub pending_deposit_stake_ts: u64,



    pub pending_withdrawal_unstake_scaled: u128,

    pub pending_withdrawal_unstake_ts: u64,

    pub bump: u64,

    pub delegatee: Pubkey,

    pub last_stake_ts: u64,





    pub rewards_issued_cumulative: [u64; MAX_REWARDS_TOKENS],

    pub _padding_1: [u64; 40],
}