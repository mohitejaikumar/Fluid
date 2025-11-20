use anchor_lang::prelude::{instruction::Instruction, program::invoke_signed, *};
use anchor_spl::{ token::{CloseAccount, close_account}, token_interface::TokenAccount};

use crate::{
    errors::AggregatorError, helpers::{
        deposit_to_kamino::KaminoVault,
        kamino::{
            get_kamino_balance::get_kamino_shares_amount_from_usdc, 
            get_kamino_farm_active_balance
        },
    }, states::ReserveWithdrawAccounts
};



fn get_farm_unstake_discriminator() -> Vec<u8> {
    vec![90, 95, 107, 42, 205, 124, 50, 225]
}

fn get_farm_withdraw_unstaked_discriminator() -> Vec<u8> {
    vec![36, 102, 187, 49, 220, 36, 132, 67]
}

fn get_withdraw_discriminator() -> Vec<u8> {
    vec![183, 18, 70, 156, 148, 109, 161, 34]
}

fn get_withdraw_from_available_discriminator() -> Vec<u8> {
    vec![19, 131, 112, 155, 170, 220, 34, 57]
}




impl<'info> KaminoVault<'info> {
    

    fn vault_has_allocations(&self) -> bool {
        !self.reserve_accounts.is_empty()
    }

    
    fn unstake_from_farm(&self, shares_amount: u64, config_bump: u8) -> Result<()> {
        msg!("Unstaking {} shares from farm", shares_amount);
        
        // Convert u64 to u128 as required by the Kamino Farm unstake instruction
        let shares_amount_scaled: u128 = (shares_amount as u128) * 1_000_000_000_000_000_000; // Scale by 10^18 (WAD)
        
        let mut instruction_data = get_farm_unstake_discriminator();
        instruction_data.extend_from_slice(&shares_amount_scaled.to_le_bytes());

        let mut account_metas = Vec::with_capacity(4);
        account_metas.push(AccountMeta::new(*self.config.key, true));
        account_metas.push(AccountMeta::new(*self.config_state.key, false));
        account_metas.push(AccountMeta::new(*self.farm_state.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.scope_prices.key, false));

        let instruction = Instruction {
            program_id: *self.farm_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];

        let mut accounts_for_cpi = Vec::with_capacity(4);
        accounts_for_cpi.push(self.config.clone());
        accounts_for_cpi.push(self.config_state.clone());
        accounts_for_cpi.push(self.farm_state.clone());
        accounts_for_cpi.push(self.scope_prices.clone());

        invoke_signed(&instruction, &accounts_for_cpi, signer_seeds)
        .map_err(|e| {
            msg!("Kamino farm unstake CPI failed with error: {:?}", e);
            AggregatorError::CpiToLendingProgramFailed
        })?;

        Ok(())
    }

    /// we have kamino farm staking but this implement is here for future use
    fn withdraw_unstaked_from_farm(&self, config_bump: u8) -> Result<()> {
        msg!("Withdrawing unstaked deposits from farm");
        
        let instruction_data = get_farm_withdraw_unstaked_discriminator();

        let mut account_metas = Vec::with_capacity(7);
        account_metas.push(AccountMeta::new(*self.config.key, true));
        account_metas.push(AccountMeta::new(*self.config_state.key, false));
        account_metas.push(AccountMeta::new(*self.farm_state.key, false));
        account_metas.push(AccountMeta::new(*self.config_shares_ata.key, false));
        account_metas.push(AccountMeta::new(*self.farm_vault.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.farm_vault_authority.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.token_program.key, false));

        let instruction = Instruction {
            program_id: *self.farm_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];

        let mut accounts_for_cpi = Vec::with_capacity(7);
        accounts_for_cpi.push(self.config.clone());
        accounts_for_cpi.push(self.config_state.clone());
        accounts_for_cpi.push(self.farm_state.clone());
        accounts_for_cpi.push(self.config_shares_ata.clone());
        accounts_for_cpi.push(self.farm_vault.clone());
        accounts_for_cpi.push(self.farm_vault_authority.clone());
        accounts_for_cpi.push(self.token_program.clone());

        invoke_signed(&instruction, &accounts_for_cpi, signer_seeds)
        .map_err(|e| {
            msg!("Kamino farm withdraw unstaked CPI failed with error: {:?}", e);
            AggregatorError::CpiToLendingProgramFailed
        })?;

        Ok(())
    }

    
    
    fn withdraw_from_single_reserve(
        &self, 
        reserve: &ReserveWithdrawAccounts<'info>,
        shares_amount: u64, 
        config_bump: u8
    ) -> Result<()> {
        msg!("Withdrawing {} shares from reserve {}", shares_amount, reserve.reserve.key);
        
        let mut instruction_data = get_withdraw_discriminator();
        instruction_data.extend_from_slice(&shares_amount.to_le_bytes());

        let mut account_metas = Vec::with_capacity(29);
        account_metas.push(AccountMeta::new(*self.config.key, true));
        account_metas.push(AccountMeta::new(*self.vault_state.key, false));
        account_metas.push(AccountMeta::new(*self.token_vault.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.base_vault_authority.key, false));
        account_metas.push(AccountMeta::new(*self.user_token_ata.key, false));
        account_metas.push(AccountMeta::new(*self.token_mint.key, false));
        account_metas.push(AccountMeta::new(*self.config_shares_ata.key, false));
        account_metas.push(AccountMeta::new(*self.shares_mint.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.token_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.shares_token_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.klend_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.event_authority.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.kamino_lending_vault_program.key, false));
        account_metas.push(AccountMeta::new(*self.vault_state.key, false));
        account_metas.push(AccountMeta::new(*reserve.reserve.key, false));
        account_metas.push(AccountMeta::new(*reserve.ctoken_vault.key, false));
        account_metas.push(AccountMeta::new_readonly(*reserve.lending_market.key, false));
        account_metas.push(AccountMeta::new_readonly(*reserve.lending_market_authority.key, false));
        account_metas.push(AccountMeta::new(*reserve.reserve_liquidity_supply.key, false));
        account_metas.push(AccountMeta::new(*reserve.reserve_collateral_mint.key, false));
        account_metas.push(AccountMeta::new_readonly(*reserve.reserve_collateral_token_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.instruction_sysvar.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.event_authority.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.kamino_lending_vault_program.key, false));
        account_metas.push(AccountMeta::new(*self.reserve_accounts[0].reserve.key, false));
        account_metas.push(AccountMeta::new(*self.reserve_accounts[1].reserve.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.reserve_accounts[0].lending_market.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.reserve_accounts[1].lending_market.key, false));

        let instruction = Instruction {
            program_id: *self.kamino_vault_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];

        let mut accounts_for_cpi = Vec::with_capacity(29);
        accounts_for_cpi.push(self.config.clone());
        accounts_for_cpi.push(self.vault_state.clone());
        accounts_for_cpi.push(self.token_vault.clone());
        accounts_for_cpi.push(self.base_vault_authority.clone());
        accounts_for_cpi.push(self.user_token_ata.clone());
        accounts_for_cpi.push(self.token_mint.clone());
        accounts_for_cpi.push(self.config_shares_ata.clone());
        accounts_for_cpi.push(self.shares_mint.clone());
        accounts_for_cpi.push(self.token_program.clone());
        accounts_for_cpi.push(self.shares_token_program.clone());
        accounts_for_cpi.push(self.klend_program.clone());
        accounts_for_cpi.push(self.event_authority.clone());
        accounts_for_cpi.push(self.kamino_lending_vault_program.clone());
        accounts_for_cpi.push(self.vault_state.clone());
        accounts_for_cpi.push(reserve.reserve.clone());
        accounts_for_cpi.push(reserve.ctoken_vault.clone());
        accounts_for_cpi.push(reserve.lending_market.clone());
        accounts_for_cpi.push(reserve.lending_market_authority.clone());
        accounts_for_cpi.push(reserve.reserve_liquidity_supply.clone());
        accounts_for_cpi.push(reserve.reserve_collateral_mint.clone());
        accounts_for_cpi.push(reserve.reserve_collateral_token_program.clone());
        accounts_for_cpi.push(self.instruction_sysvar.clone());
        accounts_for_cpi.push(self.event_authority.clone());
        accounts_for_cpi.push(self.kamino_lending_vault_program.clone());
        accounts_for_cpi.push(self.reserve_accounts[0].reserve.clone());
        accounts_for_cpi.push(self.reserve_accounts[1].reserve.clone());
        accounts_for_cpi.push(self.reserve_accounts[0].lending_market.clone());
        accounts_for_cpi.push(self.reserve_accounts[1].lending_market.clone());

        invoke_signed(&instruction, &accounts_for_cpi, signer_seeds)
            .map_err(|e| {
                msg!("Kamino withdraw from reserve CPI failed with error: {:?}", e);
                AggregatorError::CpiToLendingProgramFailed
            })?;

        Ok(())
    }

    
    fn withdraw_with_allocations(&self, shares_amount: u64, config_bump: u8) -> Result<()> {
        msg!("Withdrawing {} shares from vault (with allocations)", shares_amount);
        msg!("Number of reserves: {}", self.reserve_accounts.len());
        
        
        let shares_remaining = shares_amount;
        
        for (index, reserve) in self.reserve_accounts.iter().enumerate() {
            msg!("Withdrawing from reserve {} of {}", index + 1, self.reserve_accounts.len());
            
            
            let amount_for_reserve = if index == self.reserve_accounts.len() - 1 {
                
                shares_remaining
            } else {
                0
            };
            
            if amount_for_reserve > 0 {
                self.withdraw_from_single_reserve(reserve, amount_for_reserve, config_bump)?;
            }
        }

        Ok(())
    }

   
    fn withdraw_from_available(&self, shares_amount: u64, config_bump: u8) -> Result<()> {
        msg!("Withdrawing {} shares from vault (from available)", shares_amount);
        
        let mut instruction_data = get_withdraw_from_available_discriminator();
        instruction_data.extend_from_slice(&shares_amount.to_le_bytes());

        let mut account_metas = Vec::with_capacity(13);
        account_metas.push(AccountMeta::new(*self.config.key, true));
        account_metas.push(AccountMeta::new(*self.vault_state.key, false));
        account_metas.push(AccountMeta::new(*self.token_vault.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.base_vault_authority.key, false));
        account_metas.push(AccountMeta::new(*self.user_token_ata.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.token_mint.key, false));
        account_metas.push(AccountMeta::new(*self.config_shares_ata.key, false));
        account_metas.push(AccountMeta::new(*self.shares_mint.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.token_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.shares_token_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.klend_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.event_authority.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.kamino_lending_vault_program.key, false));

        let instruction = Instruction {
            program_id: *self.kamino_vault_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];

        let mut accounts_for_cpi = Vec::with_capacity(13);
        accounts_for_cpi.push(self.config.clone());
        accounts_for_cpi.push(self.vault_state.clone());
        accounts_for_cpi.push(self.token_vault.clone());
        accounts_for_cpi.push(self.base_vault_authority.clone());
        accounts_for_cpi.push(self.user_token_ata.clone());
        accounts_for_cpi.push(self.token_mint.clone());
        accounts_for_cpi.push(self.config_shares_ata.clone());
        accounts_for_cpi.push(self.shares_mint.clone());
        accounts_for_cpi.push(self.token_program.clone());
        accounts_for_cpi.push(self.shares_token_program.clone());
        accounts_for_cpi.push(self.klend_program.clone());
        accounts_for_cpi.push(self.event_authority.clone());
        accounts_for_cpi.push(self.kamino_lending_vault_program.clone());

        invoke_signed(&instruction, &accounts_for_cpi, signer_seeds)
        .map_err(|e| {
            msg!("Kamino withdraw from available CPI failed with error: {:?}", e);
            AggregatorError::CpiToLendingProgramFailed
        })?;

        Ok(())
    }

    #[allow(dead_code)]
    fn close_shares_ata(&self, config_bump: u8) -> Result<()> {
        msg!("Closing shares ATA to reclaim rent");

        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];

        close_account(CpiContext::new_with_signer(
            self.associated_token_program.to_account_info(),
            CloseAccount {
                account: self.user_shares_ata.to_account_info(),
                authority: self.signer.to_account_info(),
                destination: self.signer.to_account_info(),
            },
            signer_seeds,
        ))?;

        Ok(())
    }

    

  
    pub fn execute_complete_withdraw(
        &self,
        user_shares_ata: &InterfaceAccount<'info, TokenAccount>,
        shares_amount: u64,
        config_bump: u8,
    ) -> Result<()> {

        self.create_shares_ata(
            &self.shares_mint.to_account_info(),
            &user_shares_ata.to_account_info(),
            &self.config.to_account_info(),
        )?;

        if self.has_farm() {
            let shares_in_ata = user_shares_ata.amount;
            msg!("Vault has farm. Shares in ATA: {}", shares_in_ata);
            
            // Check if we need to unstake (not enough shares in ATA)
            if shares_amount > shares_in_ata {
                msg!("Need to unstake from farm");
                
                
                let amount_to_unstake = if shares_amount == u64::MAX {
                    u64::MAX
                } else {
                    shares_amount.saturating_sub(shares_in_ata)
                };
                
                msg!("Unstaking {} shares", amount_to_unstake);
                
                
                self.unstake_from_farm(amount_to_unstake, config_bump)?;
                self.withdraw_unstaked_from_farm(config_bump)?;
            } else {
                msg!("Enough shares in ATA, no need to unstake");
            }
        } else {
            msg!("Vault has no farm, skipping farm operations");
        }


        self.create_shares_ata(
            &self.token_mint.to_account_info(),
            &self.user_token_ata.to_account_info(),
            &self.config.to_account_info(),
        )?;

        
        let has_allocations = self.vault_has_allocations();
        if has_allocations {
            
            self.withdraw_with_allocations(shares_amount, config_bump)?;
        } else {
            
            self.withdraw_from_available(shares_amount, config_bump)?;
        }


        msg!("Kamino Vault withdrawal completed successfully");
        Ok(())
    }


    pub fn withdraw_from_kamino_by_shares(
        &self,
        kamino_user_shares_ata_account_info: &InterfaceAccount<'info, TokenAccount>,
        kamino_user_state_account_info: &'info AccountInfo<'info>,
        kamino_vault_state_account_info: &'info AccountInfo<'info>,
        reserve_accounts: &Vec<AccountInfo<'info>>,
        current_slot: u64,
        usdc_to_withdraw: u64,
        config_bump: u8,
    ) -> Result<()> {

        let kamino_farm_active_balance = get_kamino_farm_active_balance(
            kamino_user_shares_ata_account_info,
            kamino_user_state_account_info,
        )?;
        // calculate the share_amount_from_usdc for kamino
        let shares_amount = get_kamino_shares_amount_from_usdc(
            usdc_to_withdraw,
            kamino_vault_state_account_info,
            kamino_farm_active_balance,
            reserve_accounts,
            Some(current_slot),
        )?;

        msg!("Withdrawing from Kamino: {}", shares_amount);
    
        self.execute_complete_withdraw(
            kamino_user_shares_ata_account_info,
            shares_amount,
            config_bump,
        )?;
    
        Ok(())
    }
}
