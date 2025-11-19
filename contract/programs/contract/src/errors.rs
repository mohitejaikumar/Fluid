use anchor_lang::prelude::*;


#[error_code]
pub enum AggregatorError {
    #[msg("Invalid allocation percentage")]
    InvalidAllocation,
    #[msg("Invalid amount: must be greater than 0")]
    InvalidAmount,
    #[msg("Math overflow detected")]
    MathOverflow,
    #[msg("Insufficient liquidity in protocol")]
    InsufficientLiquidity,
    #[msg("CPI to lending program failed")]
    CpiToLendingProgramFailed,
    #[msg("Invalid account data or deserialization failed")]
    InvalidAccountData,
    #[msg("Insufficient balance for withdrawal")]
    InsufficientBalance,
    #[msg("Missing required account in remaining accounts")]
    MissingAccount,
    #[msg("Invalid protocol index")]
    InvalidProtocolIndex,
    #[msg("Account reload failed")]
    AccountReloadFailed,
    #[msg("Invalid shares amount")]
    InvalidShares,
}