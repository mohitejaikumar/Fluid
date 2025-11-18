use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{helpers::{calculate_total_asset_balance::calculate_total_asset_balance, rebalance_allocation::rebalance_allocation}, states::aggregator_config::AggregatorConfig};




#[derive(Accounts)]
pub struct Rebalance<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = authority
    )]
    pub config: Account<'info, AggregatorConfig>,

    pub authority: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = config.usdc_mint,
        associated_token::authority = config,
    )]
    pub vault_usdc: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"cusdc-mint"],
        bump
    )]
    pub cusdc_mint: InterfaceAccount<'info, Mint>,

    pub usdc_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

}


impl<'info> Rebalance<'info> {
    pub fn rebalance(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {


        let usdc_in_all_protocol = calculate_total_asset_balance(remaining_accounts)?;

        rebalance_allocation(
            &self.authority,
            remaining_accounts,
            usdc_in_all_protocol,
            &self.config,
            &mut self.vault_usdc,
            &self.usdc_mint,
            &self.token_program,
            &self.associated_token_program,
            &self.system_program,
            &self.rent.to_account_info()
        )?;

        Ok(())

    }
}