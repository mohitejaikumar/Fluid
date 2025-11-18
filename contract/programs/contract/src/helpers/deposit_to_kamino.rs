use anchor_lang::prelude::{instruction::Instruction, program::invoke_signed, *};
use anchor_spl::{associated_token::{Create, create_idempotent}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{errors::AggregatorError, states::{AggregatorConfig, ReserveWithdrawAccounts}};



fn get_deposit_discriminator() -> Vec<u8> {
    // discriminator = sha256("global:deposit")[0..8]
    vec![242, 35, 198, 137, 82, 225, 242, 182]
}

fn get_farm_initialize_user_discriminator() -> Vec<u8> {
    // discriminator = sha256("global:initializeUser")[0..8]
    vec![111, 17, 185, 250, 60, 122, 38, 254]
}

fn get_farm_stake_discriminator() -> Vec<u8> {
    // discriminator = sha256("global:stake")[0..8]
    vec![206, 176, 202, 18, 200, 209, 179, 108]
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
    ) -> Box<KaminoVault<'info>> {

        let mut account_iter = remaining_accounts.iter();
        
        // Skip first 13 accounts (JupLend accounts)
        for _ in 0..13 {
            account_iter.next();
        }

        // Vault accounts
        let kamino_vault_state = account_iter.next().unwrap();
        let kamino_token_vault = account_iter.next().unwrap();
        let kamino_base_vault_authority = account_iter.next().unwrap();
        let kamino_shares_mint = account_iter.next().unwrap();
        let kamino_user_shares_ata = account_iter.next().unwrap();
        let kamino_klend_program = account_iter.next().unwrap();
        let kamino_shares_token_program = account_iter.next().unwrap();
        let kamino_event_authority = account_iter.next().unwrap();
        let kamino_lending_vault_program = account_iter.next().unwrap();
        
        // Farm accounts
        let kamino_user_state = account_iter.next().unwrap();
        let kamino_farm_state = account_iter.next().unwrap();
        let kamino_farm_vault = account_iter.next().unwrap();
        let kamino_scope_prices = account_iter.next().unwrap();
        let kamino_farm_program = account_iter.next().unwrap();
        let kamino_vault_program = account_iter.next().unwrap();
        let kamino_farm_vault_authority = account_iter.next().unwrap();
        
        
        let instruction_sysvar = account_iter.next().unwrap();
        
        
        let mut reserve_accounts: Vec<ReserveWithdrawAccounts<'info>> = Vec::new();
        
        // Reserve 1 (7 accounts)
        let reserve_1 = ReserveWithdrawAccounts {
            reserve: account_iter.next().unwrap().to_account_info(),
            ctoken_vault: account_iter.next().unwrap().to_account_info(),
            lending_market: account_iter.next().unwrap().to_account_info(),
            lending_market_authority: account_iter.next().unwrap().to_account_info(),
            reserve_liquidity_supply: account_iter.next().unwrap().to_account_info(),
            reserve_collateral_mint: account_iter.next().unwrap().to_account_info(),
            reserve_collateral_token_program: account_iter.next().unwrap().to_account_info(),
        };
        reserve_accounts.push(reserve_1);
        
        // Reserve 2 (7 accounts)
        let reserve_2 = ReserveWithdrawAccounts {
            reserve: account_iter.next().unwrap().to_account_info(),
            ctoken_vault: account_iter.next().unwrap().to_account_info(),
            lending_market: account_iter.next().unwrap().to_account_info(),
            lending_market_authority: account_iter.next().unwrap().to_account_info(),
            reserve_liquidity_supply: account_iter.next().unwrap().to_account_info(),
            reserve_collateral_mint: account_iter.next().unwrap().to_account_info(),
            reserve_collateral_token_program: account_iter.next().unwrap().to_account_info(),
        };
        reserve_accounts.push(reserve_2);
    
        

        let kamino_accounts = Box::new(KaminoVault {
            signer: signer.to_account_info(),
            config: config.to_account_info(),
            vault_state: kamino_vault_state.to_account_info(),
            token_vault: kamino_token_vault.to_account_info(),
            token_mint: usdc_mint.to_account_info(),
            base_vault_authority: kamino_base_vault_authority.to_account_info(),
            shares_mint: kamino_shares_mint.to_account_info(),
            user_token_ata: vault_usdc.to_account_info(),
            user_shares_ata: kamino_user_shares_ata.to_account_info(),
            klend_program: kamino_klend_program.to_account_info(),
            token_program: token_program.to_account_info(),
            shares_token_program: kamino_shares_token_program.to_account_info(),
            event_authority: kamino_event_authority.to_account_info(),
            kamino_lending_vault_program: kamino_lending_vault_program.to_account_info(),
            associated_token_program: associated_token_program.to_account_info(),
            farm_vault_authority: kamino_farm_vault_authority.to_account_info(),
            system_program: system_program.to_account_info(),
            rent: rent.to_account_info(),
            vault_farm: kamino_farm_state.to_account_info(),
            farm_state: kamino_farm_state.to_account_info(),
            user_farm_state: kamino_user_state.to_account_info(),
            farm_vault: kamino_farm_vault.to_account_info(),
            scope_prices: kamino_scope_prices.to_account_info(),
            farm_program: kamino_farm_program.to_account_info(),
            reserve_accounts: reserve_accounts,
            instruction_sysvar: instruction_sysvar.to_account_info(),
            kamino_vault_program: kamino_vault_program.to_account_info(),
        });

        kamino_accounts
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
        .map_err(|_| AggregatorError::CpiToLendingProgramFailed)?;

        Ok(())
    }

    pub fn deposit_to_kamino(&self, amount: u64, config_bump: u8) -> Result<()> {
        let mut instruction_data = get_deposit_discriminator();
        instruction_data.extend_from_slice(&amount.to_le_bytes());

        let mut account_metas = vec![
            AccountMeta::new(*self.config.key, true),
            AccountMeta::new(*self.vault_state.key, false),
            AccountMeta::new(*self.token_vault.key, false),
            AccountMeta::new_readonly(*self.token_mint.key, false),
            AccountMeta::new_readonly(*self.base_vault_authority.key, false),
            AccountMeta::new(*self.shares_mint.key, false),
            AccountMeta::new(*self.user_token_ata.key, false),
            AccountMeta::new(*self.user_shares_ata.key, false),
            AccountMeta::new_readonly(*self.klend_program.key, false),
            AccountMeta::new_readonly(*self.token_program.key, false),
            AccountMeta::new_readonly(*self.shares_token_program.key, false),
            AccountMeta::new_readonly(*self.event_authority.key, false),
            AccountMeta::new_readonly(*self.kamino_lending_vault_program.key, false),
        ];

        // Append remaining accounts (reserves + lending markets)
        let num_reserves = self.reserve_accounts.len();
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

        let mut accounts_for_cpi = vec![
            self.config.clone(),
            self.vault_state.clone(),
            self.token_vault.clone(),
            self.token_mint.clone(),
            self.base_vault_authority.clone(),
            self.shares_mint.clone(),
            self.user_token_ata.clone(),
            self.user_shares_ata.clone(),
            self.klend_program.clone(),
            self.token_program.clone(),
            self.shares_token_program.clone(),
            self.event_authority.clone(),
            self.kamino_lending_vault_program.clone(),
        ];

        for account in &self.reserve_accounts {
            accounts_for_cpi.push(account.reserve.clone());
        }
        for account in &self.reserve_accounts {
            accounts_for_cpi.push(account.lending_market.clone());
        }

        invoke_signed(&instruction, &accounts_for_cpi, signer_seeds)
            .map_err(|_| AggregatorError::CpiToLendingProgramFailed)?;
    
        Ok(())
    }

    
    fn initialize_user_farm_state(&self, config_bump: u8) -> Result<()> {
        let instruction_data = get_farm_initialize_user_discriminator();

        
        let account_metas = vec![
            AccountMeta::new(*self.config.key, true),        
            AccountMeta::new(*self.config.key, true),                 
            AccountMeta::new_readonly(*self.config.key, false),       
            AccountMeta::new_readonly(*self.config.key, false),       
            AccountMeta::new(*self.user_farm_state.key, false),     
            AccountMeta::new(*self.farm_state.key, false),          
            AccountMeta::new_readonly(*self.system_program.key, false),
            AccountMeta::new_readonly(*self.rent.key, false),
        ];

        let instruction = Instruction {
            program_id: *self.farm_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &instruction,
            &[
                self.config.clone(),
                self.config.clone(),
                self.config.clone(),
                self.config.clone(),  
                self.user_farm_state.clone(),
                self.farm_state.clone(),
                self.system_program.clone(),
                self.rent.clone(),
            ],
            signer_seeds,
        )
        .map_err(|_| AggregatorError::CpiToLendingProgramFailed)?;

        Ok(())
    }


    fn stake_in_farm(&self, shares_amount: u64, config_bump: u8) -> Result<()> {
        let mut instruction_data = get_farm_stake_discriminator();
        instruction_data.extend_from_slice(&shares_amount.to_le_bytes());

        let account_metas = vec![
            AccountMeta::new(*self.config.key, true),        
            AccountMeta::new(*self.user_farm_state.key, false),     
            AccountMeta::new(*self.farm_state.key, false),          
            AccountMeta::new(*self.farm_vault.key, false),          
            AccountMeta::new(*self.user_shares_ata.key, false),     
            AccountMeta::new(*self.shares_mint.key, false), 
            AccountMeta::new_readonly(*self.scope_prices.key, false), 
            AccountMeta::new_readonly(*self.token_program.key, false),
        ];

        let instruction = Instruction {
            program_id: *self.farm_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        let seeds = &[b"config".as_ref(), &[config_bump]];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &instruction,
            &[
                self.config.clone(),
                self.user_farm_state.clone(),
                self.farm_state.clone(),
                self.farm_vault.clone(),
                self.user_shares_ata.clone(),
                self.shares_mint.clone(),
                self.scope_prices.clone(),
                self.token_program.clone(),
            ],
            signer_seeds,
        )
        .map_err(|_| AggregatorError::CpiToLendingProgramFailed)?;

        Ok(())
    }


    pub fn stake_shares_in_farm(&self, shares_amount: u64, config_bump: u8) -> Result<()> {
        if !self.has_farm() {
            msg!("Vault has no farm (vault_farm == default), skipping farm staking");
            return Ok(());
        }

        msg!("Vault has farm, staking shares...");

        let init_result = self.initialize_user_farm_state(config_bump);
        match init_result {
            Ok(_) => msg!("User farm state initialized"),
            Err(_) => msg!("User farm state already exists or init failed, continuing to stake"),
        }


        self.stake_in_farm(shares_amount, config_bump)?;

        Ok(())
    }

    
    pub fn execute_complete_deposit(&self, amount: u64, config_bump: u8) -> Result<()> {
        // Step 1: Create shares ATA if needed
        self.create_shares_ata(
            &self.shares_mint.to_account_info(),
            &self.user_shares_ata.to_account_info(),
            &self.config.to_account_info(),
        )?;
        
        // Step 2: Execute deposit
        self.deposit_to_kamino(amount, config_bump)?;
        
        // Step 3: Stake in farm if vault has one
        self.stake_shares_in_farm(u64::MAX, config_bump)?;
        
        Ok(())
    }

}
