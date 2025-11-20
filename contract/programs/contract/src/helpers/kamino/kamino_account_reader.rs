
use anchor_lang::prelude::*;

pub mod vault_offsets {
    pub const TOKEN_AVAILABLE: usize = 216;                    
    pub const SHARES_ISSUED: usize = TOKEN_AVAILABLE + 8;      
    pub const PENDING_FEES_SF: usize = SHARES_ISSUED + 8 + 56; 
    
    pub const VAULT_ALLOCATION_STRATEGY: usize = PENDING_FEES_SF + 16; 
    
    pub const VAULT_ALLOCATION_SIZE: usize = 2160;
    pub const MAX_RESERVES: usize = 25;
}


pub mod allocation_offsets {
    pub const RESERVE: usize = 0;                          
    pub const CTOKEN_VAULT: usize = RESERVE + 32;          
    pub const TARGET_ALLOCATION_WEIGHT: usize = CTOKEN_VAULT + 32; 
    pub const TOKEN_ALLOCATION_CAP: usize = TARGET_ALLOCATION_WEIGHT + 8; 
    pub const CTOKEN_VAULT_BUMP: usize = TOKEN_ALLOCATION_CAP + 8; 
    
    pub const CTOKEN_ALLOCATION: usize = CTOKEN_VAULT_BUMP + 8 + 1016; 
}

pub mod reserve_offsets {
    pub const LAST_UPDATE_SLOT: usize = 8;
    
    pub const LIQUIDITY_START: usize = 120;
    
    pub const AVAILABLE_AMOUNT: usize = LIQUIDITY_START + 96;           
    pub const BORROWED_AMOUNT_SF: usize = AVAILABLE_AMOUNT + 8;         
    pub const ACCUMULATED_PROTOCOL_FEES_SF: usize = BORROWED_AMOUNT_SF + 16 + 96; 
    pub const ACCUMULATED_REFERRER_FEES_SF: usize = ACCUMULATED_PROTOCOL_FEES_SF + 16; 
    pub const PENDING_REFERRER_FEES_SF: usize = ACCUMULATED_REFERRER_FEES_SF + 16; 
    
    pub const COLLATERAL_START: usize = LIQUIDITY_START + 1232 + 1200;
    pub const MINT_TOTAL_SUPPLY: usize = COLLATERAL_START + 32;  
    pub const CONFIG_START: usize = COLLATERAL_START + 1096 + 1200;
    
    pub const PROTOCOL_ORDER_EXECUTION_FEE_PCT: usize = CONFIG_START + 13;  
    pub const PROTOCOL_TAKE_RATE_PCT: usize = CONFIG_START + 14;            
    pub const PROTOCOL_LIQUIDATION_FEE_PCT: usize = CONFIG_START + 15;      
    pub const HOST_FIXED_INTEREST_RATE_BPS: usize = CONFIG_START + 2;       
}

#[derive(Debug)]
pub struct VaultStateFields {
    pub token_available: u64,
    pub shares_issued: u64,
    pub pending_fees_sf: u128,
}

pub fn read_vault_state_fields(data: &[u8]) -> Result<VaultStateFields> {
    let data = &data[8..];
    
    if data.len() < vault_offsets::PENDING_FEES_SF + 16 {
        return Err(ProgramError::InvalidAccountData.into());
    }
    
    Ok(VaultStateFields {
        token_available: read_u64(data, vault_offsets::TOKEN_AVAILABLE),
        shares_issued: read_u64(data, vault_offsets::SHARES_ISSUED),
        pending_fees_sf: read_u128(data, vault_offsets::PENDING_FEES_SF),
    })
}

pub struct VaultAllocationFields {
    pub reserve: Pubkey,
    pub ctoken_allocation: u64,
}

pub fn read_vault_allocation(
    data: &[u8],
    allocation_index: usize,
) -> Result<VaultAllocationFields> {
    // Skip 8-byte discriminator
    let data = &data[8..];
    
    let base_offset = vault_offsets::VAULT_ALLOCATION_STRATEGY 
        + (allocation_index * vault_offsets::VAULT_ALLOCATION_SIZE);
    
    if data.len() < base_offset + allocation_offsets::CTOKEN_ALLOCATION + 8 {
        return Err(ProgramError::InvalidAccountData.into());
    }
    
    Ok(VaultAllocationFields {
        reserve: read_pubkey(data, base_offset + allocation_offsets::RESERVE),
        ctoken_allocation: read_u64(data, base_offset + allocation_offsets::CTOKEN_ALLOCATION),
    })
}

pub struct ReserveFields {
    pub last_update_slot: u64,
    pub available_amount: u64,
    pub borrowed_amount_sf: u128,
    pub accumulated_protocol_fees_sf: u128,
    pub accumulated_referrer_fees_sf: u128,
    pub pending_referrer_fees_sf: u128,
    pub mint_total_supply: u64,
    pub protocol_take_rate_pct: u8,
    pub host_fixed_interest_rate_bps: u16,
}

pub fn read_reserve_fields(data: &[u8]) -> Result<ReserveFields> {
    // Skip 8-byte discriminator
    let data = &data[8..];
    
    if data.len() < reserve_offsets::MINT_TOTAL_SUPPLY + 8 {
        return Err(ProgramError::InvalidAccountData.into());
    }
    
    Ok(ReserveFields {
        last_update_slot: read_u64(data, reserve_offsets::LAST_UPDATE_SLOT),
        available_amount: read_u64(data, reserve_offsets::AVAILABLE_AMOUNT),
        borrowed_amount_sf: read_u128(data, reserve_offsets::BORROWED_AMOUNT_SF),
        accumulated_protocol_fees_sf: read_u128(data, reserve_offsets::ACCUMULATED_PROTOCOL_FEES_SF),
        accumulated_referrer_fees_sf: read_u128(data, reserve_offsets::ACCUMULATED_REFERRER_FEES_SF),
        pending_referrer_fees_sf: read_u128(data, reserve_offsets::PENDING_REFERRER_FEES_SF),
        mint_total_supply: read_u64(data, reserve_offsets::MINT_TOTAL_SUPPLY),
        protocol_take_rate_pct: read_u8(data, reserve_offsets::PROTOCOL_TAKE_RATE_PCT),
        host_fixed_interest_rate_bps: read_u16(data, reserve_offsets::HOST_FIXED_INTEREST_RATE_BPS),
    })
}


#[inline]
fn read_u8(data: &[u8], offset: usize) -> u8 {
    data[offset]
}

#[inline]
fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

#[inline]
fn read_u64(data: &[u8], offset: usize) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[offset..offset + 8]);
    u64::from_le_bytes(bytes)
}

#[inline]
fn read_u128(data: &[u8], offset: usize) -> u128 {
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&data[offset..offset + 16]);
    u128::from_le_bytes(bytes)
}

#[inline]
fn read_pubkey(data: &[u8], offset: usize) -> Pubkey {
    Pubkey::new_from_array(
        data[offset..offset + 32]
            .try_into()
            .expect("slice with incorrect length")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vault_offsets() {
        // Verify key offset calculations
        assert_eq!(vault_offsets::TOKEN_AVAILABLE, 216);
        assert_eq!(vault_offsets::SHARES_ISSUED, 224);
        assert_eq!(vault_offsets::PENDING_FEES_SF, 288);
    }
    
    #[test]
    fn test_reserve_offsets() {
        // Verify reserve offset calculations
        assert_eq!(reserve_offsets::LAST_UPDATE_SLOT, 8);
        assert_eq!(reserve_offsets::LIQUIDITY_START, 120);
    }
}

