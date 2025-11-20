use anchor_lang::prelude::Pubkey;

use crate::constants::MAX_REWARDS_TOKENS;


/// Account structure (with 8-byte discriminator):
/// - discriminator: [u8; 8]
/// - user_id: u64
/// - farm_state: Pubkey (32 bytes)
/// - owner: Pubkey (32 bytes)
/// - is_farm_delegated: u8
/// - padding_0: [u8; 7]
/// - rewards_tally_scaled: [u128; 10]
/// - rewards_issued_unclaimed: [u64; 10]
/// - last_claim_ts: [u64; 10]
/// - active_stake_scaled: u128
/// - pending_deposit_stake_scaled: u128
/// - pending_deposit_stake_ts: u64
/// - pending_withdrawal_unstake_scaled: u128
/// - pending_withdrawal_unstake_ts: u64
/// - bump: u64
/// - delegatee: Pubkey (32 bytes)
/// - last_stake_ts: u64
/// - rewards_issued_cumulative: [u64; 10]
/// - padding_1: [u64; 40]

pub const USER_STATE_DISCRIMINATOR: [u8; 8] = [72, 177, 85, 249, 76, 167, 186, 126];

pub mod offset {
    use super::MAX_REWARDS_TOKENS;
    
    pub const DISCRIMINATOR: usize = 0;
    pub const USER_ID: usize = DISCRIMINATOR + 8; // 8
    pub const FARM_STATE: usize = USER_ID + 8; // 16
    pub const OWNER: usize = FARM_STATE + 32; // 48
    pub const IS_FARM_DELEGATED: usize = OWNER + 32; // 80
    pub const PADDING_0: usize = IS_FARM_DELEGATED + 1; // 81
    pub const REWARDS_TALLY_SCALED: usize = PADDING_0 + 7; // 88
    pub const REWARDS_ISSUED_UNCLAIMED: usize = REWARDS_TALLY_SCALED + (16 * MAX_REWARDS_TOKENS); // 248
    pub const LAST_CLAIM_TS: usize = REWARDS_ISSUED_UNCLAIMED + (8 * MAX_REWARDS_TOKENS); // 328
    pub const ACTIVE_STAKE_SCALED: usize = LAST_CLAIM_TS + (8 * MAX_REWARDS_TOKENS); // 408
    pub const PENDING_DEPOSIT_STAKE_SCALED: usize = ACTIVE_STAKE_SCALED + 16; // 424
    pub const PENDING_DEPOSIT_STAKE_TS: usize = PENDING_DEPOSIT_STAKE_SCALED + 16; // 440
    pub const PENDING_WITHDRAWAL_UNSTAKE_SCALED: usize = PENDING_DEPOSIT_STAKE_TS + 8; // 448
    pub const PENDING_WITHDRAWAL_UNSTAKE_TS: usize = PENDING_WITHDRAWAL_UNSTAKE_SCALED + 16; // 464
    pub const BUMP: usize = PENDING_WITHDRAWAL_UNSTAKE_TS + 8; // 472
    pub const DELEGATEE: usize = BUMP + 8; // 480
    pub const LAST_STAKE_TS: usize = DELEGATEE + 32; // 512
}

pub const USER_STATE_MIN_LEN: usize = offset::LAST_STAKE_TS + 8; // 520 bytes minimum

#[inline]
fn check_len(buf: &[u8]) {
    assert!(
        buf.len() >= USER_STATE_MIN_LEN,
        "buffer too small: need at least {} bytes, got {}",
        USER_STATE_MIN_LEN,
        buf.len()
    );
}

#[inline]
fn read_u128_from_bytes(buf: &[u8], offset: usize) -> u128 {
    let mut arr = [0u8; 16];
    arr.copy_from_slice(&buf[offset..offset + 16]);
    u128::from_le_bytes(arr)
}

#[inline]
fn read_u64_from_bytes(buf: &[u8], offset: usize) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&buf[offset..offset + 8]);
    u64::from_le_bytes(arr)
}

#[inline]
fn read_u8_from_bytes(buf: &[u8], offset: usize) -> u8 {
    buf[offset]
}

#[inline]
fn read_pubkey_from_bytes(buf: &[u8], offset: usize) -> Pubkey {
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&buf[offset..offset + 32]);
    Pubkey::new_from_array(arr)
}


pub fn get_discriminator(buf: &[u8]) -> [u8; 8] {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&buf[offset::DISCRIMINATOR..offset::DISCRIMINATOR + 8]);
    arr
}


pub fn get_user_id(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::USER_ID)
}


pub fn get_farm_state(buf: &[u8]) -> Pubkey {
    check_len(buf);
    read_pubkey_from_bytes(buf, offset::FARM_STATE)
}


pub fn get_owner(buf: &[u8]) -> Pubkey {
    check_len(buf);
    read_pubkey_from_bytes(buf, offset::OWNER)
}

/// Reads the is_farm_delegated flag from the UserState account
pub fn get_is_farm_delegated(buf: &[u8]) -> u8 {
    check_len(buf);
    read_u8_from_bytes(buf, offset::IS_FARM_DELEGATED)
}


/// This represents the user's active stake in the farm (scaled by WAD = 10^18)
pub fn get_active_stake_scaled(buf: &[u8]) -> u128 {
    check_len(buf);
    read_u128_from_bytes(buf, offset::ACTIVE_STAKE_SCALED)
}

/// Reads the pending_deposit_stake_scaled from the UserState account
pub fn get_pending_deposit_stake_scaled(buf: &[u8]) -> u128 {
    check_len(buf);
    read_u128_from_bytes(buf, offset::PENDING_DEPOSIT_STAKE_SCALED)
}

/// Reads the pending_deposit_stake_ts from the UserState account
pub fn get_pending_deposit_stake_ts(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::PENDING_DEPOSIT_STAKE_TS)
}

/// Reads the pending_withdrawal_unstake_scaled from the UserState account
pub fn get_pending_withdrawal_unstake_scaled(buf: &[u8]) -> u128 {
    check_len(buf);
    read_u128_from_bytes(buf, offset::PENDING_WITHDRAWAL_UNSTAKE_SCALED)
}

/// Reads the pending_withdrawal_unstake_ts from the UserState account
pub fn get_pending_withdrawal_unstake_ts(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::PENDING_WITHDRAWAL_UNSTAKE_TS)
}

/// Reads the bump from the UserState account
pub fn get_bump(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::BUMP)
}

/// Reads the delegatee pubkey from the UserState account
pub fn get_delegatee(buf: &[u8]) -> Pubkey {
    check_len(buf);
    read_pubkey_from_bytes(buf, offset::DELEGATEE)
}

pub fn get_last_stake_ts(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::LAST_STAKE_TS)
}

