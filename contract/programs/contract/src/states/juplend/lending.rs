use anchor_lang::prelude::*;


#[derive(Debug)]
#[account]
pub struct Lending {
    pub mint: Pubkey,
    pub f_token_mint: Pubkey,
    pub lending_id: u16,
    /// @dev number of decimals for the fToken, same as ASSET
    pub decimals: u8,
    /// @dev To read PDA of rewards rate model to get_rate instruction
    pub rewards_rate_model: Pubkey,
    /// @dev exchange price for the underlying asset in the liquidity protocol (without rewards)
    pub liquidity_exchange_price: u64,
    /// @dev exchange price between fToken and the underlying asset (with rewards)
    pub token_exchange_price: u64,
    /// @dev timestamp when exchange prices were updated the last time
    pub last_update_timestamp: u64,
    pub token_reserves_liquidity: Pubkey,
    pub supply_position_on_liquidity: Pubkey,
    pub bump: u8,
}