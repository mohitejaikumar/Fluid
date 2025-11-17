use anchor_lang::prelude::*;


#[error_code]
pub enum AggregatorError {
    #[msg("Invalid allocation percentage")]
    InvalidAllocation,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("CPI to lending program failed")]
    CpiToLendingProgramFailed,
    #[msg("Invalid account data")]
    InvalidAccountData,
    #[msg("Insufficient balance")]
    InsufficientBalance,

}