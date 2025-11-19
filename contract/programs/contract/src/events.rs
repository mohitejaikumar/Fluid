use anchor_lang::prelude::*;


#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub cusdc_minted: u64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub cusdc_burned: u64,
    pub usdc_returned: u64,
}

#[event]
pub struct RebalanceEvent {
    pub juplend_balance: u64,
    pub kamino_balance: u64,
}

#[event]
pub struct AllocationUpdateEvent {
    pub juplend_bps: u16,
    pub kamino_bps: u16,
}

#[event]
pub struct ViewEvent {
    pub user: Pubkey,
    pub user_yeild: u64,
}