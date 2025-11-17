use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct AggregatorConfig {
    pub authority: Pubkey,
    pub usdc_mint: Pubkey,
    pub cusdc_mint: Pubkey,
    pub vault_usdc: Pubkey,
    pub juplend_allocation_bps: u16,
    pub kamino_allocation_bps: u16,
    pub total_deposits: u64,
    pub bump: u8,
}