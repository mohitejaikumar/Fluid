use anchor_lang::prelude::{instruction::Instruction, program::{invoke, invoke_signed}, *};
use anchor_spl::{
    associated_token::{
        Create, 
        create_idempotent
    }, 
    token::{
        TransferChecked, 
        spl_token::state::Account as SplTokenAccount, 
        transfer_checked
    }, 
    token_interface::{
        Mint, 
        TokenAccount, 
        TokenInterface
    }
};

use crate::{errors::AggregatorError, states::{AggregatorConfig, ReserveWithdrawAccounts}};
use anchor_lang::solana_program::program_pack::Pack;


fn get_deposit_discriminator() -> Vec<u8> {
    vec![242, 35, 198, 137, 82, 225, 242, 182]
}

fn get_farm_initialize_user_discriminator() -> Vec<u8> {
    vec![111, 17, 185, 250, 60, 122, 38, 254]
}

fn get_farm_stake_discriminator() -> Vec<u8> {
    vec![206, 176, 202, 18, 200, 209, 179, 108]
}

fn get_farm_transfer_ownership_discriminator() -> Vec<u8> {
    vec![65, 177, 215, 73, 53, 45, 99, 47]
}





pub struct KaminoVault<'info> {
    // Core vault accounts
    pub signer: AccountInfo<'info>,
    pub config: AccountInfo<'info>,
    pub vault_state: AccountInfo<'info>,
    pub token_vault: AccountInfo<'info>,
    pub token_mint: AccountInfo<'info>,
    pub base_vault_authority: AccountInfo<'info>,
    pub shares_mint: AccountInfo<'info>,
    pub user_token_ata: AccountInfo<'info>,
    pub user_shares_ata: AccountInfo<'info>,
    pub config_shares_ata: AccountInfo<'info>,
    pub klend_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub shares_token_program: AccountInfo<'info>,
    pub event_authority: AccountInfo<'info>,
    pub kamino_lending_vault_program: AccountInfo<'info>,
    pub farm_vault_authority: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
    
    pub vault_farm: AccountInfo<'info>,          
    pub farm_state: AccountInfo<'info>,          
    pub user_farm_state: AccountInfo<'info>,
    pub config_state: AccountInfo<'info>,
    pub farm_vault: AccountInfo<'info>,          
    pub scope_prices: AccountInfo<'info>,        
    pub farm_program: AccountInfo<'info>,   
    
    
    pub reserve_accounts: Vec<ReserveWithdrawAccounts<'info>>,
    pub instruction_sysvar: AccountInfo<'info>,
    
    pub kamino_vault_program: AccountInfo<'info>,
}

impl<'info> KaminoVault<'info> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        signer: &Signer<'info>,
        config: &Account<'info, AggregatorConfig>,
        remaining_accounts: &'info [AccountInfo<'info>],
        vault_usdc: &InterfaceAccount<'info, TokenAccount>,
        usdc_mint: &InterfaceAccount<'info, Mint>,
        token_program: &Interface<'info, TokenInterface>,
        associated_token_program: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        rent: &AccountInfo<'info>,
    ) -> Result<Box<KaminoVault<'info>>> {

        // Skip first 13 accounts (JupLend accounts) 
        let number_of_juplend_accounts = 13; 
        
        let mut reserve_accounts: Vec<ReserveWithdrawAccounts<'info>> = Vec::with_capacity(2);
        
        let reserve_idx = number_of_juplend_accounts + 9 + 7 + 3;
        
        // Reserve 1 (7 accounts starting at reserve_idx)
        reserve_accounts.push(ReserveWithdrawAccounts {
            reserve: remaining_accounts.get(reserve_idx).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            ctoken_vault: remaining_accounts.get(reserve_idx + 1).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            lending_market: remaining_accounts.get(reserve_idx + 2).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            lending_market_authority: remaining_accounts.get(reserve_idx + 3).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            reserve_liquidity_supply: remaining_accounts.get(reserve_idx + 4).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            reserve_collateral_mint: remaining_accounts.get(reserve_idx + 5).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            reserve_collateral_token_program: remaining_accounts.get(reserve_idx + 6).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        });
        
        // Reserve 2 (7 accounts starting at reserve_idx + 7)
        let reserve_idx_2 = reserve_idx + 7;
        reserve_accounts.push(ReserveWithdrawAccounts {
            reserve: remaining_accounts.get(reserve_idx_2).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            ctoken_vault: remaining_accounts.get(reserve_idx_2 + 1).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            lending_market: remaining_accounts.get(reserve_idx_2 + 2).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            lending_market_authority: remaining_accounts.get(reserve_idx_2 + 3).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            reserve_liquidity_supply: remaining_accounts.get(reserve_idx_2 + 4).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            reserve_collateral_mint: remaining_accounts.get(reserve_idx_2 + 5).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            reserve_collateral_token_program: remaining_accounts.get(reserve_idx_2 + 6).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        });
        
        // Directly construct the Box to avoid large stack allocations
        Ok(Box::new(KaminoVault {
            signer: signer.to_account_info(),
            config: config.to_account_info(),
            config_state: remaining_accounts.get(number_of_juplend_accounts + 9 + 7 + 1).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            config_shares_ata: remaining_accounts.get(number_of_juplend_accounts + 9 + 7 + 2).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            vault_state: remaining_accounts.get(number_of_juplend_accounts).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            token_vault: remaining_accounts.get(number_of_juplend_accounts + 1).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            token_mint: usdc_mint.to_account_info(),
            base_vault_authority: remaining_accounts.get(number_of_juplend_accounts + 2).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            shares_mint: remaining_accounts.get(number_of_juplend_accounts + 3).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            user_token_ata: vault_usdc.to_account_info(),
            user_shares_ata: remaining_accounts.get(number_of_juplend_accounts + 4).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            klend_program: remaining_accounts.get(number_of_juplend_accounts + 5).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            token_program: token_program.to_account_info(),
            shares_token_program: remaining_accounts.get(number_of_juplend_accounts + 6).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            event_authority: remaining_accounts.get(number_of_juplend_accounts + 7).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            kamino_lending_vault_program: remaining_accounts.get(number_of_juplend_accounts + 8).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            associated_token_program: associated_token_program.to_account_info(),
            farm_vault_authority: remaining_accounts.get(number_of_juplend_accounts + 9 + 6).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            system_program: system_program.to_account_info(),
            rent: rent.to_account_info(),
            vault_farm: remaining_accounts.get(number_of_juplend_accounts + 9 + 1).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            farm_state: remaining_accounts.get(number_of_juplend_accounts + 9 + 1).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            user_farm_state: remaining_accounts.get(number_of_juplend_accounts + 9).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            farm_vault: remaining_accounts.get(number_of_juplend_accounts + 9 + 2).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            scope_prices: remaining_accounts.get(number_of_juplend_accounts + 9 + 3).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            farm_program: remaining_accounts.get(number_of_juplend_accounts + 9 + 4).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            reserve_accounts,
            instruction_sysvar: remaining_accounts.get(number_of_juplend_accounts + 9 + 7).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
            kamino_vault_program: remaining_accounts.get(number_of_juplend_accounts + 9 + 5).ok_or(AggregatorError::MissingAccount)?.to_account_info(),
        }))
    }

    
    pub fn has_farm(&self) -> bool {
        self.vault_farm.key != &Pubkey::default()
    }



    pub fn create_shares_ata(
        &self,
        mint: &AccountInfo<'info>,
        user_ata: &AccountInfo<'info>,
        authority: &AccountInfo<'info>,
    ) -> Result<()> {

        msg!("Creating shares ATA");

        create_idempotent(
            CpiContext::new(
                self.associated_token_program.to_account_info(),
                Create{
                    payer: self.signer.to_account_info(),
                    associated_token: user_ata.to_account_info(),
                    authority: authority.to_account_info(),
                    mint: mint.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                }
            )
        )
        .map_err(|e| {
            msg!("Kamino create ATA CPI failed with error: {:?}", e);
            AggregatorError::CpiToLendingProgramFailed
        })?;

        Ok(())
    }

    pub fn deposit_to_kamino(&self, amount: u64, config_bump: u8) -> Result<()> {
        let mut instruction_data = get_deposit_discriminator();
        instruction_data.extend_from_slice(&amount.to_le_bytes());

        // Pre-allocate with capacity to avoid reallocation
        let num_reserves = self.reserve_accounts.len();
        let mut account_metas = Vec::with_capacity(13 + num_reserves * 2);
        
        account_metas.push(AccountMeta::new(*self.config.key, true));
        account_metas.push(AccountMeta::new(*self.vault_state.key, false));
        account_metas.push(AccountMeta::new(*self.token_vault.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.token_mint.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.base_vault_authority.key, false));
        account_metas.push(AccountMeta::new(*self.shares_mint.key, false));
        account_metas.push(AccountMeta::new(*self.user_token_ata.key, false));
        account_metas.push(AccountMeta::new(*self.config_shares_ata.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.klend_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.token_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.shares_token_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.event_authority.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.kamino_lending_vault_program.key, false));

        // Append remaining accounts (reserves + lending markets)
        for i in 0..num_reserves {
            account_metas.push(AccountMeta::new(*self.reserve_accounts[i].reserve.key, false));
        }
        for i in 0..num_reserves {
            account_metas.push(AccountMeta::new_readonly(*self.reserve_accounts[i].lending_market.key, false));
        }

        let instruction = Instruction {
            program_id: *self.kamino_vault_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];

        // Pre-allocate with capacity to avoid reallocation
        let mut accounts_for_cpi = Vec::with_capacity(13 + num_reserves * 2);
        
        accounts_for_cpi.push(self.config.clone());
        accounts_for_cpi.push(self.vault_state.clone());
        accounts_for_cpi.push(self.token_vault.clone());
        accounts_for_cpi.push(self.token_mint.clone());
        accounts_for_cpi.push(self.base_vault_authority.clone());
        accounts_for_cpi.push(self.shares_mint.clone());
        accounts_for_cpi.push(self.user_token_ata.clone());
        accounts_for_cpi.push(self.config_shares_ata.clone());
        accounts_for_cpi.push(self.klend_program.clone());
        accounts_for_cpi.push(self.token_program.clone());
        accounts_for_cpi.push(self.shares_token_program.clone());
        accounts_for_cpi.push(self.event_authority.clone());
        accounts_for_cpi.push(self.kamino_lending_vault_program.clone());

        for account in &self.reserve_accounts {
            accounts_for_cpi.push(account.reserve.clone());
        }
        for account in &self.reserve_accounts {
            accounts_for_cpi.push(account.lending_market.clone());
        }

        invoke_signed(&instruction, &accounts_for_cpi, signer_seeds)
            .map_err(|e| {
                msg!("Kamino deposit CPI failed with error: {:?}", e);
                AggregatorError::CpiToLendingProgramFailed
            })?;
    
        Ok(())
    }

    fn prefund_user_farm_state(&self, config_bump: u8) -> Result<()> {


        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];


        transfer_checked(CpiContext::new_with_signer(
            self.token_program.to_account_info(), 
            TransferChecked {
                from: self.config_shares_ata.to_account_info(),
                to: self.user_shares_ata.to_account_info(),
                authority: self.config.to_account_info(),
                mint: self.shares_mint.to_account_info(),
            },
            signer_seeds), 
            1000000, 
            6
        )
        .map_err(|e| {
            msg!("Kamino farm prefund user CPI failed with error: {:?}", e);
            AggregatorError::CpiToLendingProgramFailed
        })?;

        Ok(())
    }

    fn transfer_ownership_user_farm_state(&self) -> Result<()> {
        let instruction_data = get_farm_transfer_ownership_discriminator();
        
        let mut account_metas = Vec::with_capacity(9);
        account_metas.push(AccountMeta::new(*self.signer.key, true));
        account_metas.push(AccountMeta::new(*self.signer.key, true));
        account_metas.push(AccountMeta::new_readonly(*self.config.key, false));
        account_metas.push(AccountMeta::new(*self.user_farm_state.key, false));
        account_metas.push(AccountMeta::new(*self.config_state.key, false));
        account_metas.push(AccountMeta::new(*self.farm_state.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.scope_prices.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.system_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.rent.key, false));

        let instruction = Instruction {
            program_id: *self.farm_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let mut accounts_for_cpi = Vec::with_capacity(9);
        accounts_for_cpi.push(self.signer.clone());
        accounts_for_cpi.push(self.signer.clone());
        accounts_for_cpi.push(self.config.clone());
        accounts_for_cpi.push(self.user_farm_state.clone());
        accounts_for_cpi.push(self.config_state.clone());
        accounts_for_cpi.push(self.farm_state.clone());
        accounts_for_cpi.push(self.scope_prices.clone());
        accounts_for_cpi.push(self.system_program.clone());
        accounts_for_cpi.push(self.rent.clone());

        invoke(&instruction, &accounts_for_cpi)
        .map_err(|e| {
            msg!("Kamino farm init user CPI failed with error: {:?}", e);
            AggregatorError::CpiToLendingProgramFailed
        })?;

        Ok(())
    }

    
    fn initialize_user_farm_state(&self) -> Result<()> {
        let instruction_data = get_farm_initialize_user_discriminator();

        let mut account_metas = Vec::with_capacity(8);
        account_metas.push(AccountMeta::new(*self.signer.key, true));
        account_metas.push(AccountMeta::new(*self.signer.key, true));
        account_metas.push(AccountMeta::new_readonly(*self.signer.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.signer.key, false));
        account_metas.push(AccountMeta::new(*self.user_farm_state.key, false));
        account_metas.push(AccountMeta::new(*self.farm_state.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.system_program.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.rent.key, false));

        let instruction = Instruction {
            program_id: *self.farm_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let mut accounts_for_cpi = Vec::with_capacity(8);
        accounts_for_cpi.push(self.signer.clone());
        accounts_for_cpi.push(self.signer.clone());
        accounts_for_cpi.push(self.signer.clone());
        accounts_for_cpi.push(self.signer.clone());
        accounts_for_cpi.push(self.user_farm_state.clone());
        accounts_for_cpi.push(self.farm_state.clone());
        accounts_for_cpi.push(self.system_program.clone());
        accounts_for_cpi.push(self.rent.clone());

        invoke(&instruction, &accounts_for_cpi)
        .map_err(|e| {
            msg!("Kamino farm init user CPI failed with error: {:?}", e);
            AggregatorError::CpiToLendingProgramFailed
        })?;

        Ok(())
    }


    fn stake_in_farm_by_config(&self, shares_amount: u64, config_bump: u8) -> Result<()> {
        let mut instruction_data = get_farm_stake_discriminator();
        instruction_data.extend_from_slice(&shares_amount.to_le_bytes());

        let mut account_metas = Vec::with_capacity(8);
        account_metas.push(AccountMeta::new(*self.config.key, true));
        account_metas.push(AccountMeta::new(*self.config_state.key, false));
        account_metas.push(AccountMeta::new(*self.farm_state.key, false));
        account_metas.push(AccountMeta::new(*self.farm_vault.key, false));
        account_metas.push(AccountMeta::new(*self.config_shares_ata.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.shares_mint.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.scope_prices.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.token_program.key, false));

        let instruction = Instruction {
            program_id: *self.farm_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];

        let mut accounts_for_cpi = Vec::with_capacity(8);
        accounts_for_cpi.push(self.config.clone());
        accounts_for_cpi.push(self.config_state.clone());
        accounts_for_cpi.push(self.farm_state.clone());
        accounts_for_cpi.push(self.farm_vault.clone());
        accounts_for_cpi.push(self.config_shares_ata.clone());
        accounts_for_cpi.push(self.shares_mint.clone());
        accounts_for_cpi.push(self.scope_prices.clone());
        accounts_for_cpi.push(self.token_program.clone());

        invoke_signed(&instruction, &accounts_for_cpi, signer_seeds)
        .map_err(|e| {
            msg!("Kamino farm stake CPI failed with error: {:?}", e);
            AggregatorError::CpiToLendingProgramFailed
        })?;

        Ok(())
    }

    fn stake_in_farm_by_user(&self, shares_amount: u64) -> Result<()> {
        let mut instruction_data = get_farm_stake_discriminator();
        instruction_data.extend_from_slice(&shares_amount.to_le_bytes());

        let mut account_metas = Vec::with_capacity(8);
        account_metas.push(AccountMeta::new(*self.signer.key, true));
        account_metas.push(AccountMeta::new(*self.user_farm_state.key, false));
        account_metas.push(AccountMeta::new(*self.farm_state.key, false));
        account_metas.push(AccountMeta::new(*self.farm_vault.key, false));
        account_metas.push(AccountMeta::new(*self.user_shares_ata.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.shares_mint.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.scope_prices.key, false));
        account_metas.push(AccountMeta::new_readonly(*self.token_program.key, false));

        let instruction = Instruction {
            program_id: *self.farm_program.key,
            accounts: account_metas,
            data: instruction_data,
        };


        let mut accounts_for_cpi = Vec::with_capacity(8);
        accounts_for_cpi.push(self.signer.clone());
        accounts_for_cpi.push(self.user_farm_state.clone());
        accounts_for_cpi.push(self.farm_state.clone());
        accounts_for_cpi.push(self.farm_vault.clone());
        accounts_for_cpi.push(self.user_shares_ata.clone());
        accounts_for_cpi.push(self.shares_mint.clone());
        accounts_for_cpi.push(self.scope_prices.clone());
        accounts_for_cpi.push(self.token_program.clone());

        invoke(&instruction, &accounts_for_cpi)?;

        Ok(())
    }



    pub fn stake_shares_in_farm(&self, shares_amount: u64, config_bump: u8) -> Result<()> {
        if !self.has_farm() {
            msg!("Vault has no farm (vault_farm == default), skipping farm staking");
            return Ok(());
        }

        // Check if user farm state account exists before initializing
        let user_farm_state_exists = self.user_farm_state.data_len() > 0 
            && self.user_farm_state.owner == self.farm_program.key;
        
        if !user_farm_state_exists {
            let init_result = self.initialize_user_farm_state();
            match init_result {
                Ok(_) => msg!("User farm state initialized"),
                Err(_) => msg!("User farm state init failed, continuing to stake"),
            }
        } else {
            msg!("User farm state already exists, skipping initialization");
        }

        let prefund_result = self.prefund_user_farm_state(config_bump);
        match prefund_result {
            Ok(_) => msg!("User farm state prefunded"),
            Err(_) => msg!("User farm state prefund failed, continuing to stake"),
        }

        // stake the transferred amount to the farm
        let stake_result = self.stake_in_farm_by_user(1000000);
        match stake_result {
            Ok(_) => msg!("User farm state staked"),
            Err(e) => msg!("User farm state stake failed with error: {:?}", e),
        }

        let transfer_result = self.transfer_ownership_user_farm_state();
        match transfer_result {
            Ok(_) => msg!("User farm state transferred ownership"),
            Err(_) => msg!("User farm state transfer ownership failed, continuing to stake"),
        }


        self.stake_in_farm_by_config(shares_amount - 1000000, config_bump)?;

        Ok(())
    }

    
    pub fn execute_complete_deposit(&self, amount: u64, config_bump: u8) -> Result<()> {
        // Step 1: Create shares ATA if needed
        self.create_shares_ata(
            &self.shares_mint.to_account_info(),
            &self.config_shares_ata.to_account_info(),
            &self.config.to_account_info(),
        )?;
        // this is for user shares ATA
        self.create_shares_ata(
            &self.shares_mint.to_account_info(),
            &self.user_shares_ata.to_account_info(),
            &self.signer.to_account_info(),
        )?;
        // Step 2: Execute deposit
        self.deposit_to_kamino(amount, config_bump)?;

        let amount_to_stake = {
            let data = self.config_shares_ata.try_borrow_data()?;
            SplTokenAccount::unpack(&data)?.amount
        };

        msg!("Amount to stake: {}", amount_to_stake);
        // Step 3: Stake shares in farm
        self.stake_shares_in_farm(amount_to_stake, config_bump)?;
        
        Ok(())
    }

}
