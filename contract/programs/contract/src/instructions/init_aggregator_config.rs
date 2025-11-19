use anchor_lang::prelude::*;
use anchor_spl::{ associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{constants::BPS_BASE, errors::AggregatorError, states::aggregator_config::AggregatorConfig};



#[derive(Accounts)]
#[instruction(juplend_allocation_bps: u16)]
pub struct InitAggregatorConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + AggregatorConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, AggregatorConfig>,

    pub usdc_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = config,
        seeds = [b"cusdc-mint"],
        bump
    )]
    pub cusdc_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = usdc_mint,
        associated_token::authority = config,
    )]
    pub vault_usdc: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}


impl<'info> InitAggregatorConfig<'info> {
    pub fn init_aggregator_config(&mut self, juplend_allocation_bps: u16, bumps: InitAggregatorConfigBumps) -> Result<()> {
        
        require!(juplend_allocation_bps <= BPS_BASE, AggregatorError::InvalidAllocation);

        let config = &mut self.config;
        config.authority = self.authority.key();
        config.usdc_mint = self.usdc_mint.key();
        config.cusdc_mint = self.cusdc_mint.key();
        config.vault_usdc = self.vault_usdc.key();
        config.juplend_allocation_bps = juplend_allocation_bps;
        config.kamino_allocation_bps = BPS_BASE - juplend_allocation_bps;
        config.bump = bumps.config;
        config.total_deposits = 0;

        Ok(())
    }
}