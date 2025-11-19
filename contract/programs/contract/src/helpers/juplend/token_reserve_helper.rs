use anchor_lang::prelude::Pubkey;



/// Byte layout constants for TokenReserve (packed).
pub const TOKEN_RESERVE_LEN: usize = 184;

pub mod offset {
    pub const MINT: usize = 0;
    pub const VAULT: usize = MINT + 32; // 32
    pub const BORROW_RATE: usize = VAULT + 32; // 64
    pub const FEE_ON_INTEREST: usize = BORROW_RATE + 2; // 66
    pub const LAST_UTILIZATION: usize = FEE_ON_INTEREST + 2; // 68
    pub const LAST_UPDATE_TIMESTAMP: usize = LAST_UTILIZATION + 2; // 70
    pub const SUPPLY_EXCHANGE_PRICE: usize = LAST_UPDATE_TIMESTAMP + 8; // 78
    pub const BORROW_EXCHANGE_PRICE: usize = SUPPLY_EXCHANGE_PRICE + 8; // 86
    pub const MAX_UTILIZATION: usize = BORROW_EXCHANGE_PRICE + 8; // 94
    pub const TOTAL_SUPPLY_WITH_INTEREST: usize = MAX_UTILIZATION + 2; // 96
    pub const TOTAL_SUPPLY_INTEREST_FREE: usize = TOTAL_SUPPLY_WITH_INTEREST + 8; // 104
    pub const TOTAL_BORROW_WITH_INTEREST: usize = TOTAL_SUPPLY_INTEREST_FREE + 8; // 112
    pub const TOTAL_BORROW_INTEREST_FREE: usize = TOTAL_BORROW_WITH_INTEREST + 8; // 120
    pub const TOTAL_CLAIM_AMOUNT: usize = TOTAL_BORROW_INTEREST_FREE + 8; // 128
    pub const INTERACTING_PROTOCOL: usize = TOTAL_CLAIM_AMOUNT + 8; // 136
    pub const INTERACTING_TIMESTAMP: usize = INTERACTING_PROTOCOL + 32; // 168
    pub const INTERACTING_BALANCE: usize = INTERACTING_TIMESTAMP + 8; // 176
}

#[inline]
fn check_len(buf: &[u8]) {
    assert!(
        buf.len() >= TOKEN_RESERVE_LEN,
        "buffer too small: need at least {} bytes, got {}",
        TOKEN_RESERVE_LEN,
        buf.len()
    );
}

#[inline]
fn read_u64_from_bytes(buf: &[u8], offset: usize) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&buf[offset..offset + 8]);
    u64::from_le_bytes(arr)
}

#[inline]
fn read_u16_from_bytes(buf: &[u8], offset: usize) -> u16 {
    let mut arr = [0u8; 2];
    arr.copy_from_slice(&buf[offset..offset + 2]);
    u16::from_le_bytes(arr)
}

#[inline]
fn read_pubkey_from_bytes(buf: &[u8], offset: usize) -> Pubkey {
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&buf[offset..offset + 32]);
    Pubkey::new_from_array(arr)
}


pub fn get_mint(buf: &[u8]) -> Pubkey {
    check_len(buf);
    read_pubkey_from_bytes(buf, offset::MINT)
}


pub fn get_vault(buf: &[u8]) -> Pubkey {
    check_len(buf);
    read_pubkey_from_bytes(buf, offset::VAULT)
}

pub fn get_borrow_rate(buf: &[u8]) -> u16 {
    check_len(buf);
    read_u16_from_bytes(buf, offset::BORROW_RATE)
}


pub fn get_fee_on_interest(buf: &[u8]) -> u16 {
    check_len(buf);
    read_u16_from_bytes(buf, offset::FEE_ON_INTEREST)
}

pub fn get_last_utilization(buf: &[u8]) -> u16 {
    check_len(buf);
    read_u16_from_bytes(buf, offset::LAST_UTILIZATION)
}


pub fn get_last_update_timestamp(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::LAST_UPDATE_TIMESTAMP)
}


pub fn get_supply_exchange_price(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::SUPPLY_EXCHANGE_PRICE)
}


pub fn get_borrow_exchange_price(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::BORROW_EXCHANGE_PRICE)
}


pub fn get_max_utilization(buf: &[u8]) -> u16 {
    check_len(buf);
    read_u16_from_bytes(buf, offset::MAX_UTILIZATION)
}


pub fn get_total_supply_with_interest(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::TOTAL_SUPPLY_WITH_INTEREST)
}


pub fn get_total_supply_interest_free(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::TOTAL_SUPPLY_INTEREST_FREE)
}


pub fn get_total_borrow_with_interest(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::TOTAL_BORROW_WITH_INTEREST)
}

pub fn get_total_borrow_interest_free(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::TOTAL_BORROW_INTEREST_FREE)
}


pub fn get_total_claim_amount(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::TOTAL_CLAIM_AMOUNT)
}


pub fn get_interacting_protocol(buf: &[u8]) -> Pubkey {
    check_len(buf);
    read_pubkey_from_bytes(buf, offset::INTERACTING_PROTOCOL)
}


pub fn get_interacting_timestamp(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::INTERACTING_TIMESTAMP)
}


pub fn get_interacting_balance(buf: &[u8]) -> u64 {
    check_len(buf);
    read_u64_from_bytes(buf, offset::INTERACTING_BALANCE)
}
