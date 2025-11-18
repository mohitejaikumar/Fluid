use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
use crate::instructions::*;

pub mod states;
pub mod helpers;
pub mod events;
pub mod constants;


declare_id!("AUQt43E6brmpbQ1zeWCtSvZX9zBUEkoYmHrthQk8eA6W");

#[program]
pub mod contract {
    use super::*;
 
    pub fn init_aggregator_config(ctx: Context<InitAggregatorConfig>, juplend_allocation_bps: u16) -> Result<()> {
        ctx.accounts.init_aggregator_config(juplend_allocation_bps, ctx.bumps)?;
        Ok(())
    }

    pub fn deposit<'info>(ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>, amount: u64) -> Result<()> {
        
        ctx.accounts.deposit(amount, ctx.bumps, ctx.remaining_accounts)?;
        Ok(())
    }

    pub fn withdraw<'info>(ctx: Context<'_, '_, 'info, 'info, Withdraw<'info>>, cusdc_amount: u64) -> Result<()> 
    {
        ctx.accounts.withdraw(cusdc_amount, ctx.remaining_accounts)?;

        Ok(())
    }

    pub fn rebalance<'info>(ctx: Context<'_, '_, 'info, 'info, Rebalance<'info>>) -> Result<()> {
        ctx.accounts.rebalance(ctx.remaining_accounts)?;
        Ok(())
    }

    pub fn update_strategy(ctx: Context<UpdateStrategy>, new_juplend_bps: u16) -> Result<()> {
        ctx.accounts.update_strategy(new_juplend_bps)?;
        Ok(())
    }
}

