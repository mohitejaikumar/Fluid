use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::{constants::BPS_BASE, errors::AggregatorError, events::AllocationUpdateEvent, states::aggregator_config::AggregatorConfig};



#[derive(Accounts)]
pub struct UpdateStrategy<'info> {
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

    pub usdc_mint: InterfaceAccount<'info, Mint>,

}


impl<'info> UpdateStrategy<'info> {
    pub fn update_strategy(&mut self, new_juplend_bps: u16) -> Result<()> {
        require!(
            new_juplend_bps <= 10000,
            AggregatorError::InvalidAllocation
        );

        let config = &mut self.config;
        config.juplend_allocation_bps = new_juplend_bps;
        config.kamino_allocation_bps = BPS_BASE - new_juplend_bps;

    

        emit!(AllocationUpdateEvent {
            juplend_bps: new_juplend_bps,
            kamino_bps: BPS_BASE - new_juplend_bps,
        });
        Ok(())
    }
}