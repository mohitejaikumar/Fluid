use anchor_lang::prelude::*;
use anchor_spl::{ associated_token::AssociatedToken, token_2022::{MintTo, TransferChecked, mint_to, transfer_checked}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{errors::AggregatorError, events::DepositEvent, helpers::{calculate_shares_to_mint::calculate_shares_to_mint, calculate_total_asset_balance::calculate_total_asset_balance, rebalance_allocation::rebalance_allocation}, states::aggregator_config::AggregatorConfig};


#[derive(Accounts)]
pub struct Deposit<'info> {

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, AggregatorConfig>,

    #[account(
        mut,
        constraint = user_usdc.mint == config.usdc_mint,
        constraint = user_usdc.owner == user.key()
    )]
    pub user_usdc: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = cusdc_mint,
        associated_token::authority = user
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

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, _bumps: DepositBumps, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {

        require!(amount > 0, AggregatorError::InvalidAmount);
        
        // Log remaining accounts for debugging
        msg!("Received {} remaining accounts", remaining_accounts.len());
        
        msg!("Transferring USDC to vault");
        let _ = transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.user_usdc.to_account_info(),
                    to: self.vault_usdc.to_account_info(),
                    authority: self.user.to_account_info(),
                    mint: self.usdc_mint.to_account_info(),
                }
            ),
            amount, 
            self.usdc_mint.decimals
        );

        msg!("Transferred USDC to vault");
        
        // Reload vault_usdc account to get updated balance after transfer
        self.vault_usdc.reload()?;
        
        // Get total USDC in JupLend and Kamino combined
        let usdc_in_all_protocol = calculate_total_asset_balance(remaining_accounts)?;
        let total_usdc_in_protocols_combined: u64 = usdc_in_all_protocol
            .iter()
            .try_fold(0u64, |acc, x| acc.checked_add(*x))
            .ok_or(AggregatorError::MathOverflow)?;

        let cusdc_to_mint  = calculate_shares_to_mint(
            amount,
            self.cusdc_mint.supply,
            total_usdc_in_protocols_combined
        );

        let seeds = &[b"config".as_ref(), &[self.config.bump]];
        let signer = &[&seeds[..]];

        msg!("Minting CUSDC");

        mint_to(
            CpiContext::new_with_signer(
               self.token_program.to_account_info(),
                MintTo {
                    mint: self.cusdc_mint.to_account_info(),
                    to: self.user_cusdc.to_account_info(),
                    authority: self.config.to_account_info(),
                },
                signer,
            ),
            cusdc_to_mint,
        )?;
        msg!("Minted CUSDC");

        self.config.total_deposits = self.config
            .total_deposits
            .checked_add(amount)
            .ok_or(AggregatorError::MathOverflow)?;

        msg!("Rebalancing allocation");
        // Rebalance to all protocols 
        rebalance_allocation(
            &self.user,
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
        msg!("Rebalanced allocation");

        emit!(DepositEvent {
            user: self.user.key(),
            amount,
            cusdc_minted: cusdc_to_mint,
        });

        Ok(())
    }

}