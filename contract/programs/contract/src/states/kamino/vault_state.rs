use anchor_lang::prelude::*;

use bytemuck::{Zeroable, Pod};

pub const MAX_RESERVES: usize = 25;
pub const VAULT_STATE_SIZE: usize = 62544;
pub const VAULT_ALLOCATION_SIZE: usize = 2160;




#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Name([u8; 40]);

unsafe impl Zeroable for Name {}
unsafe impl Pod for Name {}

impl Default for Name {
    fn default() -> Self {
        Self([0u8; 40])
    }
}

impl AnchorDeserialize for Name {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut bytes = [0u8; 40];
        reader.read_exact(&mut bytes)?;
        Ok(Self(bytes))
    }
}

impl AnchorSerialize for Name {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.0)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConfigPadding127([u64; 127]);

unsafe impl Zeroable for ConfigPadding127 {}
unsafe impl Pod for ConfigPadding127 {}

impl Default for ConfigPadding127 {
    fn default() -> Self {
        Self([0u64; 127])
    }
}

impl AnchorDeserialize for ConfigPadding127 {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut array = [0u64; 127];
        for item in array.iter_mut() {
            let mut bytes = [0u8; 8];
            reader.read_exact(&mut bytes)?;
            *item = u64::from_le_bytes(bytes);
        }
        Ok(Self(array))
    }
}

impl AnchorSerialize for ConfigPadding127 {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for item in self.0.iter() {
            writer.write_all(&item.to_le_bytes())?;
        }
        Ok(())
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Padding242([u128; 242]);

unsafe impl Zeroable for Padding242 {}
unsafe impl Pod for Padding242 {}

impl Default for Padding242 {
    fn default() -> Self {
        Self([0u128; 242])
    }
}

impl AnchorDeserialize for Padding242 {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut array = [0u128; 242];
        for item in array.iter_mut() {
            let mut bytes = [0u8; 16];
            reader.read_exact(&mut bytes)?;
            *item = u128::from_le_bytes(bytes);
        }
        Ok(Self(array))
    }
}

impl AnchorSerialize for Padding242 {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for item in self.0.iter() {
            writer.write_all(&item.to_le_bytes())?;
        }
        Ok(())
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StatePadding128([u64; 128]);

unsafe impl Zeroable for StatePadding128 {}
unsafe impl Pod for StatePadding128 {}

impl Default for StatePadding128 {
    fn default() -> Self {
        Self([0u64; 128])
    }
}

impl AnchorDeserialize for StatePadding128 {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut array = [0u64; 128];
        for item in array.iter_mut() {
            let mut bytes = [0u8; 8];
            reader.read_exact(&mut bytes)?;
            *item = u64::from_le_bytes(bytes);
        }
        Ok(Self(array))
    }
}

impl AnchorSerialize for StatePadding128 {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for item in self.0.iter() {
            writer.write_all(&item.to_le_bytes())?;
        }
        Ok(())
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Padding256([u128; 256]);

unsafe impl Zeroable for Padding256 {}
unsafe impl Pod for Padding256 {}

impl Default for Padding256 {
    fn default() -> Self {
        Self([0u128; 256])
    }
}

impl AnchorDeserialize for Padding256 {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut array = [0u128; 256];
        for item in array.iter_mut() {
            let mut bytes = [0u8; 16];
            reader.read_exact(&mut bytes)?;
            *item = u128::from_le_bytes(bytes);
        }
        Ok(Self(array))
    }
}

impl AnchorSerialize for Padding256 {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for item in self.0.iter() {
            writer.write_all(&item.to_le_bytes())?;
        }
        Ok(())
    }
}



#[repr(C)]
#[derive(AnchorDeserialize, PartialEq, Eq)]
pub struct VaultState {
    // Admin
    pub vault_admin_authority: Pubkey,

    pub base_vault_authority: Pubkey,
    pub base_vault_authority_bump: u64,

    pub token_mint: Pubkey,
    pub token_mint_decimals: u64,
    pub token_vault: Pubkey,
    pub token_program: Pubkey,

    // shares
    pub shares_mint: Pubkey,
    pub shares_mint_decimals: u64,

    // accounting
    pub token_available: u64,
    pub shares_issued: u64,

    pub available_crank_funds: u64,
    pub unallocated_weight: u64,

    pub performance_fee_bps: u64,
    pub management_fee_bps: u64,
    pub last_fee_charge_timestamp: u64,
    pub prev_aum_sf: u128,
    // todo: should we split this into pending_mgmt_fee and pending_perf_fee?
    pub pending_fees_sf: u128,

    pub vault_allocation_strategy: [VaultAllocation; MAX_RESERVES],
    pub padding_1: Padding256,

    // General config
    pub min_deposit_amount: u64,
    pub min_withdraw_amount: u64,
    pub min_invest_amount: u64,
    pub min_invest_delay_slots: u64,
    pub crank_fund_fee_per_reserve: u64,

    pub pending_admin: Pubkey,

    pub cumulative_earned_interest_sf: u128, // this represents the raw total interest earned by the vault, including the fees
    pub cumulative_mgmt_fees_sf: u128,
    pub cumulative_perf_fees_sf: u128,

    pub name: Name,
    pub name_padding: [u8; 8],  // Explicit padding after Name to align next Pubkey
    pub vault_lookup_table: Pubkey,
    pub vault_farm: Pubkey,

    pub creation_timestamp: u64,

    // when computing the amounts to invest in each reserve and how much to leave unallocated we use this cap as the max value that can stay uninvested; if set to 0 (for backwards compatibility) it means the same thing as U64::MAX
    pub unallocated_tokens_cap: u64,
    pub allocation_admin: Pubkey,

    pub padding_3: Padding242,
}



#[repr(C)]
#[derive(AnchorDeserialize, Debug, PartialEq, Eq)]
pub struct VaultAllocation {
    pub reserve: Pubkey,
    pub ctoken_vault: Pubkey,
    pub target_allocation_weight: u64,
    /// Maximum token invested in this reserve
    pub token_allocation_cap: u64,
    pub ctoken_vault_bump: u64,

    // all the VaultAllocation config should be above this and use this padding
    pub config_padding: ConfigPadding127,

    pub ctoken_allocation: u64,
    pub last_invest_slot: u64,
    pub token_target_allocation_sf: u128,

    pub state_padding: StatePadding128,
}


  