use anchor_lang::prelude::{instruction::Instruction, program::invoke_signed, *};
use anchor_spl::token_interface::{TokenAccount, TokenInterface};

use crate::{errors::AggregatorError, states::AggregatorConfig};



fn get_deposit_discriminator() -> Vec<u8> {
    // discriminator = sha256("global:deposit")[0..8]
    vec![242, 35, 198, 137, 82, 225, 242, 182]
}


pub struct Juplend<'info> {
    pub signer: AccountInfo<'info>,
    pub asset_token_account: AccountInfo<'info>,
    pub ftoken_account: AccountInfo<'info>,

    pub mint: AccountInfo<'info>,

    // Protocol accounts
    pub lending_admin: AccountInfo<'info>,
    pub lending: AccountInfo<'info>,
    pub f_token_mint: AccountInfo<'info>,

    // Liquidity protocol accounts
    pub supply_token_reserves_liquidity: AccountInfo<'info>,
    pub lending_supply_position_on_liquidity: AccountInfo<'info>,
    pub rate_model: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    pub liquidity: AccountInfo<'info>,
    pub liquidity_program: AccountInfo<'info>,

    // Rewards and programs
    pub rewards_rate_model: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,

    pub claim_account: AccountInfo<'info>,

    // Target lending program
    pub lending_program: AccountInfo<'info>,
}


impl<'info> Juplend<'info> {
    pub fn new(
        config: &Account<'info, AggregatorConfig>,
        remaining_accounts: &'info [AccountInfo<'info>],
        vault_usdc: &InterfaceAccount<'info, TokenAccount>,
        usdc_mint: &InterfaceAccount<'info, anchor_spl::token_interface::Mint>,
        token_program: &Interface<'info, TokenInterface>,
        associated_token_program: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
    ) -> Result<Box<Juplend<'info>>> {
        
        let signer = config.to_account_info();
        let mut account_iter = remaining_accounts.iter();
        
        let jup_lending = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_lending_rewards_rate_model = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_f_token_mint = account_iter.next().ok_or(AggregatorError::MissingAccount)?;

        let jup_vault_ftokens = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_lending_admin = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_supply_token_reserves_liquidity = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_lending_supply_position_on_liquidity = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_rate_model = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_vault = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_liquidity = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_liquidity_program = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_claim_account = account_iter.next().ok_or(AggregatorError::MissingAccount)?;
        let jup_lending_program = account_iter.next().ok_or(AggregatorError::MissingAccount)?;


        Ok(Box::new(Self {
            signer,
            asset_token_account: vault_usdc.to_account_info(),
            ftoken_account: jup_vault_ftokens.to_account_info(),
            mint: usdc_mint.to_account_info(),
            lending_admin: jup_lending_admin.to_account_info(),
            lending: jup_lending.to_account_info(),
            f_token_mint: jup_f_token_mint.to_account_info(),
            supply_token_reserves_liquidity: jup_supply_token_reserves_liquidity.to_account_info(),
            lending_supply_position_on_liquidity: jup_lending_supply_position_on_liquidity.to_account_info(),
            rate_model: jup_rate_model.to_account_info(),
            vault: jup_vault.to_account_info(),
            liquidity: jup_liquidity.to_account_info(),
            liquidity_program: jup_liquidity_program.to_account_info(),
            rewards_rate_model: jup_lending_rewards_rate_model.to_account_info(),
            token_program: token_program.to_account_info(),
            associated_token_program: associated_token_program.to_account_info(),
            system_program: system_program.to_account_info(),
            claim_account: jup_claim_account.to_account_info(),
            lending_program: jup_lending_program.to_account_info(),
        }))
    }

    pub fn deposit_to_juplend(&self, amount: u64, config_bump: u8) -> Result<()> {
        // TODO: Implement CPI to JupLend Earn

        let mut instruction_data = get_deposit_discriminator();
        instruction_data.extend_from_slice(&amount.to_le_bytes());

        msg!("asset token account: {}", self.asset_token_account.key());
        msg!("ftoken account: {}", self.ftoken_account.key());
        msg!("mint: {}", self.mint.key());

        let account_metas = vec![
            // signer (mutable, signer)
            AccountMeta::new(*self.signer.key, true),
            // depositor_token_account (mutable)
            AccountMeta::new(*self.asset_token_account.key, false),
            // recipient_token_account (mutable)
            AccountMeta::new(*self.ftoken_account.key, false),
            // mint
            AccountMeta::new_readonly(*self.mint.key, false),
            // lending_admin (readonly)
            AccountMeta::new_readonly(*self.lending_admin.key, false),
            // lending (mutable)
            AccountMeta::new(*self.lending.key, false),
            // f_token_mint (mutable)
            AccountMeta::new(*self.f_token_mint.key, false),
            // supply_token_reserves_liquidity (mutable)
            AccountMeta::new(*self.supply_token_reserves_liquidity.key, false),
            // lending_supply_position_on_liquidity (mutable)
            AccountMeta::new(*self.lending_supply_position_on_liquidity.key, false),
            // rate_model (readonly)
            AccountMeta::new_readonly(*self.rate_model.key, false),
            // vault (mutable)
            AccountMeta::new(*self.vault.key, false),
            // liquidity (mutable)
            AccountMeta::new(*self.liquidity.key, false),
            // liquidity_program (mutable)
            AccountMeta::new(*self.liquidity_program.key, false),
            // rewards_rate_model (readonly)
            AccountMeta::new_readonly(*self.rewards_rate_model.key, false),
            // token_program
            AccountMeta::new_readonly(*self.token_program.key, false),
            // associated_token_program
            AccountMeta::new_readonly(*self.associated_token_program.key, false),
            // system_program
            AccountMeta::new_readonly(*self.system_program.key, false),
        ];

        let instruction = Instruction {
            program_id: *self.lending_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];


        invoke_signed(
            &instruction,
            &[
                self.signer.clone(),
                self.asset_token_account.clone(),
                self.ftoken_account.clone(),
                self.mint.clone(),
                self.lending_admin.clone(),
                self.lending.clone(),
                self.f_token_mint.clone(),
                self.supply_token_reserves_liquidity.clone(),
                self.lending_supply_position_on_liquidity.clone(),
                self.rate_model.clone(),
                self.vault.clone(),
                self.liquidity.clone(),
                self.liquidity_program.clone(),
                self.rewards_rate_model.clone(),
                self.token_program.clone(),
                self.associated_token_program.clone(),
                self.system_program.clone(),
            ],
            signer_seeds,
        )
        .map_err(|e| {
            msg!("JupLend deposit CPI failed with error: {:?}", e);
            AggregatorError::CpiToLendingProgramFailed
        })?;
    
        Ok(())
    }
}

