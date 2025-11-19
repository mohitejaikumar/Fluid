use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::{errors::AggregatorError, events::ViewEvent, helpers::{calculate_total_asset_balance::calculate_total_asset_balance, calculate_usdc_for_shares::calculate_usdc_for_shares}, states::AggregatorConfig};


#[derive(Accounts)]
pub struct View<'info> {

    pub authority: Signer<'info>,
    #[account(
        mut,
        constraint = config.cusdc_mint == cusdc_mint.key(),
    )]
    pub config: Account<'info, AggregatorConfig>,
    
    #[account(
        constraint = user_cusdc.mint == cusdc_mint.key(),
        constraint = user_cusdc.owner == authority.key()
    )]
    pub user_cusdc: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"cusdc-mint"],
        bump
    )]
    pub cusdc_mint: InterfaceAccount<'info, Mint>,

}

impl<'info> View<'info> {
    pub fn view(&self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {

        let usdc_in_all_protocol = calculate_total_asset_balance(remaining_accounts)?;
        let total_usdc_in_protocols_combined: u64 = usdc_in_all_protocol
            .iter()
            .try_fold(0u64, |acc, x| acc.checked_add(*x))
            .ok_or(AggregatorError::MathOverflow)?;

        let user_yeild = calculate_usdc_for_shares(
            self.user_cusdc.amount, 
            self.cusdc_mint.supply, 
            total_usdc_in_protocols_combined
        );

        emit!(ViewEvent {
            user: self.authority.key(),
            user_yeild: user_yeild,
        });

        Ok(())
    }
}