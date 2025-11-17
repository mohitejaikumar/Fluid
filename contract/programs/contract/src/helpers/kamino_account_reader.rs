/// Manual byte-offset readers for Kamino accounts
/// This avoids risky zero-copy deserialization of large structs
use anchor_lang::prelude::*;

// ============================================================================
// VaultState Offsets (after 8-byte discriminator)
// ============================================================================
pub mod vault_offsets {
    // Skip vault_admin_authority (32) + base_vault_authority (32) + bump (8)
    // + token_mint (32) + decimals (8) + token_vault (32) + token_program (32)
    // + shares_mint (32) + shares_decimals (8)
    // = 216 bytes
    
    pub const TOKEN_AVAILABLE: usize = 216;                    // u64 at offset 216
    pub const SHARES_ISSUED: usize = TOKEN_AVAILABLE + 8;      // u64 at offset 224
    
    // Skip available_crank_funds (8) + unallocated_weight (8) + perf_fee_bps (8)
    // + mgmt_fee_bps (8) + last_fee_timestamp (8) + prev_aum_sf (16)
    // = 56 bytes
    
    pub const PENDING_FEES_SF: usize = SHARES_ISSUED + 8 + 56; // u128 at offset 288
    
    // vault_allocation_strategy starts after pending_fees_sf
    pub const VAULT_ALLOCATION_STRATEGY: usize = PENDING_FEES_SF + 16; // offset 304
    
    // Each VaultAllocation is 2160 bytes, max 25 allocations
    pub const VAULT_ALLOCATION_SIZE: usize = 2160;
    pub const MAX_RESERVES: usize = 25;
}

// ============================================================================
// VaultAllocation Offsets (within each allocation entry)
// ============================================================================
pub mod allocation_offsets {
    pub const RESERVE: usize = 0;                          // Pubkey at offset 0
    pub const CTOKEN_VAULT: usize = RESERVE + 32;          // Pubkey at offset 32
    pub const TARGET_ALLOCATION_WEIGHT: usize = CTOKEN_VAULT + 32; // u64 at 64
    pub const TOKEN_ALLOCATION_CAP: usize = TARGET_ALLOCATION_WEIGHT + 8; // u64 at 72
    pub const CTOKEN_VAULT_BUMP: usize = TOKEN_ALLOCATION_CAP + 8; // u64 at 80
    
    // Skip config_padding (127 * 8 = 1016 bytes)
    pub const CTOKEN_ALLOCATION: usize = CTOKEN_VAULT_BUMP + 8 + 1016; // u64 at 1104
}

// ============================================================================
// Reserve Offsets (after 8-byte discriminator)
// ============================================================================
pub mod reserve_offsets {
    // version (8) + last_update.slot (8)
    pub const LAST_UPDATE_SLOT: usize = 8;
    
    // Skip version (8) + last_update (16) + lending_market (32) 
    // + farm_collateral (32) + farm_debt (32) = 120
    
    // ReserveLiquidity starts at offset 120
    pub const LIQUIDITY_START: usize = 120;
    
    // Within ReserveLiquidity (relative to LIQUIDITY_START):
    // mint_pubkey (32) + supply_vault (32) + fee_vault (32) = 96
    pub const AVAILABLE_AMOUNT: usize = LIQUIDITY_START + 96;           // u64
    pub const BORROWED_AMOUNT_SF: usize = AVAILABLE_AMOUNT + 8;         // u128
    
    // Skip market_price_sf (16) + market_price_ts (8) + mint_decimals (8)
    // + deposit_limit_crossed (8) + borrow_limit_crossed (8)
    // + cumulative_borrow_rate_bsf (48) = 96
    pub const ACCUMULATED_PROTOCOL_FEES_SF: usize = BORROWED_AMOUNT_SF + 16 + 96; // u128
    pub const ACCUMULATED_REFERRER_FEES_SF: usize = ACCUMULATED_PROTOCOL_FEES_SF + 16; // u128
    pub const PENDING_REFERRER_FEES_SF: usize = ACCUMULATED_REFERRER_FEES_SF + 16; // u128
    
    // ReserveLiquidity is 1736 bytes total
    // Skip reserve_liquidity_padding (150 * 8 = 1200)
    // ReserveCollateral starts at: LIQUIDITY_START + 1736 + 1200 = 3056
    pub const COLLATERAL_START: usize = LIQUIDITY_START + 1736 + 1200;
    
    // Within ReserveCollateral:
    // mint_pubkey (32)
    pub const MINT_TOTAL_SUPPLY: usize = COLLATERAL_START + 32;  // u64
    
    // ReserveCollateral is 1096 bytes
    // Skip reserve_collateral_padding (150 * 8 = 1200)
    // ReserveConfig starts at: COLLATERAL_START + 1096 + 1200 = 5352
    pub const CONFIG_START: usize = COLLATERAL_START + 1096 + 1200;
    
    // Within ReserveConfig:
    // status (1) + asset_tier (1) + host_fixed_interest_rate_bps (2)
    // + min_deleveraging_bonus_bps (2) + block_ctoken_usage (1) + reserved_1 (6)
    pub const PROTOCOL_ORDER_EXECUTION_FEE_PCT: usize = CONFIG_START + 13;  // u8
    pub const PROTOCOL_TAKE_RATE_PCT: usize = CONFIG_START + 14;            // u8
    pub const PROTOCOL_LIQUIDATION_FEE_PCT: usize = CONFIG_START + 15;      // u8
    
    // For host_fixed_interest_rate_bps:
    pub const HOST_FIXED_INTEREST_RATE_BPS: usize = CONFIG_START + 2;       // u16
    
    // Skip to borrow_rate_curve:
    // After the above fields and other config fields (need to count precisely)
    // fees: ReserveFees is at some offset, then borrow_rate_curve
    // For now, we'll calculate when needed based on struct layout
}

// ============================================================================
// Reader Functions
// ============================================================================

/// Read VaultState fields needed for balance calculation
pub struct VaultStateFields {
    pub token_available: u64,
    pub shares_issued: u64,
    pub pending_fees_sf: u128,
}

pub fn read_vault_state_fields(data: &[u8]) -> Result<VaultStateFields> {
    // Skip 8-byte discriminator
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

/// Read a single VaultAllocation from the vault_allocation_strategy array
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

/// Read Reserve fields needed for exchange rate calculation
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

// ============================================================================
// Low-level byte readers
// ============================================================================

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

