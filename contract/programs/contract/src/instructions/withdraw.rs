use anchor_lang::prelude::*;
use anchor_spl::{ associated_token::AssociatedToken, token_2022::{Burn, TransferChecked, burn, transfer_checked}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{errors::AggregatorError, events::WithdrawEvent, helpers::{calculate_total_asset_balance::calculate_total_asset_balance, calculate_usdc_for_shares::calculate_usdc_for_shares, rebalance_allocation::rebalance_allocation, withdraw_from_protocols::withdraw_from_protocols}, states::aggregator_config::AggregatorConfig};



#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, AggregatorConfig>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_usdc.mint == config.usdc_mint,
        constraint = user_usdc.owner == user.key()
    )]
    pub user_usdc: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_cusdc.mint == config.cusdc_mint,
        constraint = user_cusdc.owner == user.key()
    )]
    pub user_cusdc: InterfaceAccount<'info, TokenAccount>,

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

    #[account(
        constraint = usdc_mint.key() == config.usdc_mint
    )]
    pub usdc_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}


impl<'info> Withdraw<'info> {

    pub fn withdraw(&mut self, cusdc_amount: u64, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> 
    {
        require!(cusdc_amount > 0, AggregatorError::InvalidAmount);

        let config = &self.config;

        // TODO: Get total USDC in JupLend and Kamino combined
        let usdc_in_all_protocol = calculate_total_asset_balance(remaining_accounts)?;
        let total_usdc_in_protocols_combined: u64 = usdc_in_all_protocol
            .iter()
            .try_fold(0u64, |acc, x| acc.checked_add(*x))
            .ok_or(AggregatorError::MathOverflow)?;
        
        // calculate the usdc to withdraw based on cusdc shares
        let usdc_to_withdraw = calculate_usdc_for_shares(
            cusdc_amount,
            self.cusdc_mint.supply,
            total_usdc_in_protocols_combined
        );

        withdraw_from_protocols(
            usdc_to_withdraw,
            &self.user,
            remaining_accounts,
            self.config.clone(),
            self.vault_usdc.clone(),
            self.usdc_mint.clone(),
            self.token_program.clone(),
            self.associated_token_program.to_account_info(),
            self.system_program.to_account_info(),
            self.rent.to_account_info(),
        )?;

        // burn the cusdc amount
        burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                Burn {
                    mint: self.cusdc_mint.to_account_info(),
                    from: self.user_cusdc.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            cusdc_amount,
        )?;

        self.vault_usdc.reload()?;


        let seeds = &[b"config".as_ref(), &[config.bump]];
        let signer = &[&seeds[..]];

        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vault_usdc.to_account_info(),
                    to: self.user_usdc.to_account_info(),
                    authority: self.config.to_account_info(),
                    mint: self.usdc_mint.to_account_info(),
                },
                signer,
            ),
            usdc_to_withdraw,
            self.usdc_mint.decimals
        )?;

        self.vault_usdc.reload()?;
        
        // Rebalance JupLend and Kamino allocation
        rebalance_allocation(
            &self.user,
            remaining_accounts,
            usdc_in_all_protocol,
            &self.config,
            &self.vault_usdc,
            &self.usdc_mint,
            &self.token_program,
            &self.associated_token_program,
            &self.system_program,
            &self.rent.to_account_info()
        )?;

        emit!(WithdrawEvent {
            user: self.user.key(),
            cusdc_burned: cusdc_amount,
            usdc_returned: usdc_to_withdraw,
        });
        
        Ok(())
    }
}