use anchor_lang::prelude::*;

/// Token configuration and exchange prices
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct TokenReserve {
    pub mint: Pubkey,
    pub vault: Pubkey,

    pub borrow_rate: u16,
    pub fee_on_interest: u16,
    pub last_utilization: u16,

    pub last_update_timestamp: u64,
    pub supply_exchange_price: u64,
    pub borrow_exchange_price: u64,

    pub max_utilization: u16,

    pub total_supply_with_interest: u64,
    pub total_supply_interest_free: u64,
    pub total_borrow_with_interest: u64,
    pub total_borrow_interest_free: u64,
    pub total_claim_amount: u64,

    pub interacting_protocol: Pubkey,
    pub interacting_timestamp: u64,
    pub interacting_balance: u64,
}
