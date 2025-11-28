use anchor_lang::prelude::{instruction::Instruction, program::invoke_signed, *};

use crate::{errors::AggregatorError, helpers::deposit_to_juplend::Juplend};


fn get_withdraw_discriminator() -> Vec<u8> {
    vec![183, 18, 70, 156, 148, 109, 161, 34]
}



impl<'info> Juplend<'info> {
    pub fn withdraw_from_juplend(&self, usdc_amount: u64, config_bump: u8) -> Result<()> {
        
        let mut instruction_data = get_withdraw_discriminator();
        instruction_data.extend_from_slice(&usdc_amount.to_le_bytes());

        let account_metas = vec![
            // signer (mutable, signer)
            AccountMeta::new(*self.signer.key, true),
            // owner_token_account (mutable) - user's fToken account
            AccountMeta::new(*self.ftoken_account.key, false),
            // recipient_token_account (mutable) - user's underlying token account
            AccountMeta::new(*self.asset_token_account.key, false),
            // lending_admin (readonly)
            AccountMeta::new_readonly(*self.lending_admin.key, false),
            // lending (mutable)
            AccountMeta::new(*self.lending.key, false),
            // mint (readonly) - underlying token mint
            AccountMeta::new_readonly(*self.mint.key, false),
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
            // claim_account (mutable)
            AccountMeta::new(*self.claim_account.key, false),
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
                self.ftoken_account.clone(),
                self.asset_token_account.clone(),
                self.lending_admin.clone(),
                self.lending.clone(),
                self.mint.clone(),
                self.f_token_mint.clone(),
                self.supply_token_reserves_liquidity.clone(),
                self.lending_supply_position_on_liquidity.clone(),
                self.rate_model.clone(),
                self.vault.clone(),
                self.claim_account.clone(),
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
            msg!("JupLend withdraw CPI failed with error: {:?}", e);
            AggregatorError::CpiToLendingProgramFailed
        })?;
    
        Ok(())
    }
}