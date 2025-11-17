use anchor_lang::prelude::*;


/// All accounts needed to withdraw from a specific reserve
#[derive(Clone)]
pub struct ReserveWithdrawAccounts<'info> {
    pub reserve: AccountInfo<'info>,
    pub ctoken_vault: AccountInfo<'info>,
    pub lending_market: AccountInfo<'info>,
    pub lending_market_authority: AccountInfo<'info>,
    pub reserve_liquidity_supply: AccountInfo<'info>,
    pub reserve_collateral_mint: AccountInfo<'info>,
    pub reserve_collateral_token_program: AccountInfo<'info>,
}
