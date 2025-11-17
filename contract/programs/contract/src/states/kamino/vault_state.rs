use std::fmt::{self, Display};

use anchor_lang::prelude::*;
pub use fixed::types::U68F60 as Fraction;
use fixed::traits::{FromFixed, ToFixed};
pub use fixed_macro::types::U68F60 as fraction;

use bytemuck::{Zeroable, Pod};

pub const MAX_RESERVES: usize = 25;
pub const VAULT_STATE_SIZE: usize = 62544;
pub const VAULT_ALLOCATION_SIZE: usize = 2160;


#[allow(clippy::assign_op_pattern)]
#[allow(clippy::reversed_empty_ranges)]
mod uint_types {
    use uint::construct_uint;
    construct_uint! {

        pub struct U256(4);
    }
    construct_uint! {

        pub struct U128(2);
    }
}

pub use uint_types::{U128, U256};

// Wrapper types for arrays to implement Pod and Zeroable
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

pub fn pow_fraction(fraction: Fraction, power: u32) -> Option<Fraction> {
    if power == 0 {
        return Some(Fraction::ONE);
    }

   
   
    let mut x = fraction;
    let mut y = Fraction::ONE;
    let mut n = power;

    while n > 1 {
        if n % 2 == 1 {
            y = x.checked_mul(y)?;
        }
        x = x.checked_mul(x)?;
        n /= 2;
    }

    x.checked_mul(y)
}



pub struct FractionDisplay<'a>(&'a Fraction);

impl Display for FractionDisplay<'_> {
    fn fmt(&self, formater: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sf = self.0.to_bits();

       
        const ROUND_COMP: u128 = (1 << Fraction::FRAC_NBITS) / (10_000 * 2);
        let sf = sf + ROUND_COMP;

       
        let i = sf >> Fraction::FRAC_NBITS;

       
        const FRAC_MASK: u128 = (1 << Fraction::FRAC_NBITS) - 1;
        let f_p = (sf & FRAC_MASK) as u64;
       
        let f_p = ((f_p >> 30) * 10_000) >> 30;
        write!(formater, "{i}.{f_p:0>4}")
    }
}

impl std::fmt::Debug for FractionDisplay<'_> {
    fn fmt(&self, formater: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formater, "{}", self)
    }
}

pub trait FractionExtra {
    fn to_percent<Dst: FromFixed>(&self) -> Option<Dst>;
    fn to_bps<Dst: FromFixed>(&self) -> Option<Dst>;
    fn from_percent<Src: ToFixed>(percent: Src) -> Self;
    fn from_bps<Src: ToFixed>(bps: Src) -> Self;
    fn checked_pow(&self, power: u32) -> Option<Self>
    where
        Self: std::marker::Sized;

    fn mul_int_ratio(&self, numerator: impl Into<u128>, denominator: impl Into<u128>) -> Self;
    fn full_mul_int_ratio(&self, numerator: impl Into<U256>, denominator: impl Into<U256>) -> Self;
    fn full_mul_int_ratio_ceil(
        &self,
        numerator: impl Into<U256>,
        denominator: impl Into<U256>,
    ) -> Self;

    fn div_ceil(&self, denominator: &Self) -> Self;

    fn to_floor<Dst: FromFixed>(&self) -> Dst;
    fn to_ceil<Dst: FromFixed>(&self) -> Dst;
    fn to_round<Dst: FromFixed>(&self) -> Dst;

    fn try_to_floor<Dst: FromFixed>(&self) -> Option<Dst>;
    fn try_to_ceil<Dst: FromFixed>(&self) -> Option<Dst>;
    fn try_to_round<Dst: FromFixed>(&self) -> Option<Dst>;

    fn to_sf(&self) -> u128;
    fn from_sf(sf: u128) -> Self;

    fn to_display(&self) -> FractionDisplay;
}

impl FractionExtra for Fraction {
    #[inline]
    fn to_percent<Dst: FromFixed>(&self) -> Option<Dst> {
        self.checked_mul(fraction!(100))?.round().checked_to_num()
    }

    #[inline]
    fn to_bps<Dst: FromFixed>(&self) -> Option<Dst> {
        self.checked_mul(fraction!(10_000))?
            .round()
            .checked_to_num()
    }

    #[inline]
    fn from_percent<Src: ToFixed>(percent: Src) -> Self {
        let percent = Fraction::from_num(percent);
        percent / 100
    }

    #[inline]
    fn from_bps<Src: ToFixed>(bps: Src) -> Self {
        let bps = Fraction::from_num(bps);
        bps / 10_000
    }

    #[inline]
    fn checked_pow(&self, power: u32) -> Option<Self>
    where
        Self: Sized,
    {
        pow_fraction(*self, power)
    }

    #[inline]
    fn mul_int_ratio(&self, numerator: impl Into<u128>, denominator: impl Into<u128>) -> Self {
        let numerator = numerator.into();
        let denominator = denominator.into();
        *self * numerator / denominator
    }

    #[inline]
    fn full_mul_int_ratio(&self, numerator: impl Into<U256>, denominator: impl Into<U256>) -> Self {
        let numerator = numerator.into();
        let denominator = denominator.into();
        let big_sf = U256::from(self.to_bits());
        let big_sf_res = big_sf * numerator / denominator;
        let sf_res: u128 = big_sf_res
            .try_into()
            .expect("Denominator is not big enough, the result doesn't fit in a Fraction.");
        Fraction::from_bits(sf_res)
    }

    #[inline]
    fn full_mul_int_ratio_ceil(
        &self,
        numerator: impl Into<U256>,
        denominator: impl Into<U256>,
    ) -> Self {
        let numerator = numerator.into();
        let denominator = denominator.into();
        let big_sf = U256::from(self.to_bits());
        let big_sf_res = (big_sf * numerator + denominator - 1) / denominator;
        let sf_res: u128 = big_sf_res
            .try_into()
            .expect("Denominator is not big enough, the result doesn't fit in a Fraction.");
        Fraction::from_bits(sf_res)
    }

    #[inline]
    fn div_ceil(&self, denum: &Self) -> Self {
        let num_sf = self.to_bits();
        let denum_sf = denum.to_bits();
        let res_sf_u256 =
            ((U256::from(num_sf) << Self::FRAC_NBITS) + U256::from(denum_sf - 1)) / denum_sf;
        let res_sf = u128::try_from(res_sf_u256).expect("Overflow in div_ceil");
        Self::from_bits(res_sf)
    }

    #[inline]
    fn to_floor<Dst: FromFixed>(&self) -> Dst {
        self.floor().to_num()
    }

    #[inline]
    fn to_ceil<Dst: FromFixed>(&self) -> Dst {
        self.ceil().to_num()
    }

    #[inline]
    fn to_round<Dst: FromFixed>(&self) -> Dst {
        self.round().to_num()
    }

    fn try_to_floor<Dst: FromFixed>(&self) -> Option<Dst> {
        self.floor().checked_to_num()
    }

    fn try_to_ceil<Dst: FromFixed>(&self) -> Option<Dst> {
        self.ceil().checked_to_num()
    }

    fn try_to_round<Dst: FromFixed>(&self) -> Option<Dst> {
        self.round().checked_to_num()
    }

    #[inline]
    fn to_sf(&self) -> u128 {
        self.to_bits()
    }

    #[inline]
    fn from_sf(sf: u128) -> Self {
        Fraction::from_bits(sf)
    }

    #[inline]
    fn to_display(&self) -> FractionDisplay {
        FractionDisplay(self)
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





#[derive(Default, Clone)]
    pub struct InvestedReserve {
        pub reserve: Pubkey,
        pub liquidity_amount: Fraction,
        pub ctoken_amount: u64,
        pub target_weight: u64,
    }

    impl fmt::Debug for InvestedReserve {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("InvestedReserve")
                .field("reserve", &self.reserve)
                .field("liquidity_amount", &self.liquidity_amount.to_display())
                .field("ctoken_amount", &self.ctoken_amount)
                .field("target_weight", &self.target_weight)
                .finish()
        }
    }

#[derive(Default, Clone)]
    pub struct Invested {
        pub allocations: Box<[InvestedReserve; MAX_RESERVES]>,
        pub total: Fraction,
    }

    impl fmt::Debug for Invested {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let allocations_filtered: Vec<InvestedReserve> = self
                .allocations
                .iter()
                .filter(|i| i.reserve != Pubkey::default())
                .cloned()
                .collect();

            f.debug_struct("")
                .field("total", &self.total.to_display())
                .field("allocations", &allocations_filtered)
                .finish()
        }
    }

impl VaultState {
    pub fn get_pending_fees(&self) -> Fraction {
        Fraction::from_bits(self.pending_fees_sf)
    }

    pub fn set_pending_fees(&mut self, pending_fees: Fraction) {
        self.pending_fees_sf = pending_fees.to_bits();
    }

    pub fn get_prev_aum(&self) -> Fraction {
        Fraction::from_bits(self.prev_aum_sf)
    }

    pub fn set_prev_aum(&mut self, current_aum: Fraction) {
        self.prev_aum_sf = current_aum.to_bits();
    }

    pub fn get_reserves_count(&self) -> usize {
        self.vault_allocation_strategy
            .iter()
            .filter(|r| r.reserve != Pubkey::default())
            .count()
    }

    pub fn get_reserves_with_allocation_count(&self) -> usize {
        self.vault_allocation_strategy
            .iter()
            .filter(|r| {
                r.reserve != Pubkey::default()
                    && r.target_allocation_weight > 0
                    && r.token_allocation_cap > 0
            })
            .count()
    }

    pub fn get_cumulative_earned_interest(&self) -> Fraction {
        Fraction::from_bits(self.cumulative_earned_interest_sf)
    }

    pub fn set_cumulative_earned_interest(&mut self, cumulative_earned_interest: Fraction) {
        self.cumulative_earned_interest_sf = cumulative_earned_interest.to_bits();
    }

    pub fn get_cumulative_mgmt_fees(&self) -> Fraction {
        Fraction::from_bits(self.cumulative_mgmt_fees_sf)
    }

    pub fn set_cumulative_mgmt_fees(&mut self, cumulative_mgmt_fees: Fraction) {
        self.cumulative_mgmt_fees_sf = cumulative_mgmt_fees.to_bits();
    }

    pub fn get_cumulative_perf_fees(&self) -> Fraction {
        Fraction::from_bits(self.cumulative_perf_fees_sf)
    }

    pub fn set_cumulative_perf_fees(&mut self, cumulative_perf_fees: Fraction) {
        self.cumulative_perf_fees_sf = cumulative_perf_fees.to_bits();
    }

    pub fn compute_aum(&self, invested_total: &Fraction) -> Result<Fraction> {
        // if the vault only has pending fees, it should not be possible to withdraw
        let pending_fees = self.get_pending_fees();

        if Fraction::from(self.token_available) + invested_total < pending_fees {
            return err!(KaminoVaultError::AUMBelowPendingFees);
        }

        Ok(Fraction::from(self.token_available) + invested_total - pending_fees)
    }

    pub fn validate(&self) -> Result<()> {
        if self.vault_admin_authority == Pubkey::default() {
            return err!(KaminoVaultError::AdminAuthorityIncorrect);
        }

        if self.base_vault_authority == Pubkey::default() {
            return err!(KaminoVaultError::BaseVaultAuthorityIncorrect);
        }

        if self.base_vault_authority_bump > u8::MAX as u64 {
            return err!(KaminoVaultError::BaseVaultAuthorityBumpIncorrect);
        }

        if self.token_mint == Pubkey::default() {
            return err!(KaminoVaultError::TokenMintIncorrect);
        }

        if self.token_mint_decimals == 0 {
            return err!(KaminoVaultError::TokenMintDecimalsIncorrect);
        }

        if self.token_vault == Pubkey::default() {
            return err!(KaminoVaultError::TokenVaultIncorrect);
        }

        if self.shares_mint == Pubkey::default() {
            return err!(KaminoVaultError::SharesMintIncorrect);
        }

        if self.shares_mint_decimals == 0 {
            return err!(KaminoVaultError::SharesMintDecimalsIncorrect);
        }

        if self.token_available != 0
            || self.shares_issued != 0
            || self.performance_fee_bps != 0
            || self.management_fee_bps != 0
            || self.pending_fees_sf != 0
            || self.last_fee_charge_timestamp != 0
            || self.prev_aum_sf != 0
        {
            return err!(KaminoVaultError::InitialAccountingIncorrect);
        }

        Ok(())
    }

    pub fn is_allocated_to_reserve(&self, reserve: Pubkey) -> bool {
        // TODO: make this more sophisticated
        self.vault_allocation_strategy
            .iter()
            .any(|r| r.reserve == reserve)
    }

    pub fn allocation_for_reserve(&self, reserve: &Pubkey) -> Result<&VaultAllocation> {
        let allocation = self
            .vault_allocation_strategy
            .iter()
            .find(|a| a.reserve == *reserve)
            .ok_or_else(|| error!(KaminoVaultError::ReserveNotPartOfAllocations))?;

        Ok(allocation)
    }

    pub fn get_reserve_idx_in_allocation(&self, reserve: &Pubkey) -> Option<usize> {
        self.vault_allocation_strategy
            .iter()
            .position(|r| r.reserve.eq(reserve))
    }

    pub fn get_reserve_allocation_mut(&mut self, idx: usize) -> Result<&mut VaultAllocation> {
        self.vault_allocation_strategy
            .get_mut(idx)
            .ok_or(error!(KaminoVaultError::OutOfRangeOfReserveIndex))
    }

    

    

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

impl VaultAllocation {
    pub fn get_token_target_allocation(&self) -> Fraction {
        Fraction::from_bits(self.token_target_allocation_sf)
    }

    pub fn set_token_target_allocation(&mut self, token_target_allocation: Fraction) {
        self.token_target_allocation_sf = token_target_allocation.to_bits();
    }

    pub fn can_be_removed(&self) -> bool {
        // If 0 target allocation and no c token allocation, remove the reserve
        self.ctoken_allocation == 0 && self.target_allocation_weight == 0
    }

    pub fn set_last_invest_slot(&mut self, slot: u64) {
        self.last_invest_slot = slot;
    }
}

impl Default for VaultAllocation {
    fn default() -> Self {
        Self {
            reserve: Pubkey::default(),
            ctoken_vault: Pubkey::default(),
            target_allocation_weight: 0,
            ctoken_allocation: 0,
            token_target_allocation_sf: 0,
            token_allocation_cap: u64::MAX,
            last_invest_slot: 0,
            ctoken_vault_bump: 0,
            config_padding: ConfigPadding127::default(),
            state_padding: StatePadding128::default(),
        }
    }
}

#[error_code]
#[derive(PartialEq, Eq)]
pub enum KaminoVaultError {
    #[msg("Cannot deposit zero tokens")]
    DepositAmountsZero = 1000,

    #[msg("Post check failed on share issued")]
    SharesIssuedAmountDoesNotMatch,

    #[msg("Math operation overflowed")]
    MathOverflow,

    #[msg("Integer conversion overflowed")]
    IntegerOverflow,

    #[msg("Withdrawn amount is below minimum")]
    WithdrawAmountBelowMinimum,

    #[msg("TooMuchLiquidityToWithdraw")]
    TooMuchLiquidityToWithdraw,

    #[msg("ReserveAlreadyExists")]
    ReserveAlreadyExists,

    #[msg("ReserveNotPartOfAllocations")]
    ReserveNotPartOfAllocations,

    #[msg("CouldNotDeserializeAccountAsReserve")]
    CouldNotDeserializeAccountAsReserve,

    #[msg("ReserveNotProvidedInTheAccounts")]
    ReserveNotProvidedInTheAccounts,

    #[msg("ReserveAccountAndKeyMismatch")]
    ReserveAccountAndKeyMismatch,

    #[msg("OutOfRangeOfReserveIndex")]
    OutOfRangeOfReserveIndex,

    #[msg("OutOfRangeOfReserveIndex")]
    CannotFindReserveInAllocations,

    #[msg("Invested amount is below minimum")]
    InvestAmountBelowMinimum,

    #[msg("AdminAuthorityIncorrect")]
    AdminAuthorityIncorrect,

    #[msg("BaseVaultAuthorityIncorrect")]
    BaseVaultAuthorityIncorrect,

    #[msg("BaseVaultAuthorityBumpIncorrect")]
    BaseVaultAuthorityBumpIncorrect,

    #[msg("TokenMintIncorrect")]
    TokenMintIncorrect,

    #[msg("TokenMintDecimalsIncorrect")]
    TokenMintDecimalsIncorrect,

    #[msg("TokenVaultIncorrect")]
    TokenVaultIncorrect,

    #[msg("SharesMintDecimalsIncorrect")]
    SharesMintDecimalsIncorrect,

    #[msg("SharesMintIncorrect")]
    SharesMintIncorrect,

    #[msg("InitialAccountingIncorrect")]
    InitialAccountingIncorrect,

    #[msg("Reserve is stale and must be refreshed before any operation")]
    ReserveIsStale,

    #[msg("Not enough liquidity disinvested to send to user")]
    NotEnoughLiquidityDisinvestedToSendToUser,

    #[msg("BPS value is greater than 10000")]
    BPSValueTooBig,

    #[msg("Deposited amount is below minimum")]
    DepositAmountBelowMinimum,

    #[msg("Vault have no space for new reserves")]
    ReserveSpaceExhausted,

    #[msg("Cannot withdraw from empty vault")]
    CannotWithdrawFromEmptyVault,

    #[msg("TokensDepositedAmountDoesNotMatch")]
    TokensDepositedAmountDoesNotMatch,

    #[msg("Amount to withdraw does not match")]
    AmountToWithdrawDoesNotMatch,

    #[msg("Liquidity to withdraw does not match")]
    LiquidityToWithdrawDoesNotMatch,

    #[msg("User received amount does not match")]
    UserReceivedAmountDoesNotMatch,

    #[msg("Shares burned amount does not match")]
    SharesBurnedAmountDoesNotMatch,

    #[msg("Disinvested liquidity amount does not match")]
    DisinvestedLiquidityAmountDoesNotMatch,

    #[msg("SharesMintedAmountDoesNotMatch")]
    SharesMintedAmountDoesNotMatch,

    #[msg("AUM decreased after invest")]
    AUMDecreasedAfterInvest,

    #[msg("AUM is below pending fees")]
    AUMBelowPendingFees,

    #[msg("Deposit amount results in 0 shares")]
    DepositAmountsZeroShares,

    #[msg("Withdraw amount results in 0 shares")]
    WithdrawResultsInZeroShares,

    #[msg("Cannot withdraw zero shares")]
    CannotWithdrawZeroShares,

    #[msg("Management fee is greater than maximum allowed")]
    ManagementFeeGreaterThanMaxAllowed,

    #[msg("Vault assets under management are empty")]
    VaultAUMZero,

    #[msg("Missing reserve for batch refresh")]
    MissingReserveForBatchRefresh,

    #[msg("Min withdraw amount is too big")]
    MinWithdrawAmountTooBig,

    #[msg("Invest is called too soon after last invest")]
    InvestTooSoon,

    #[msg("Wrong admin or allocation admin")]
    WrongAdminOrAllocationAdmin,

    #[msg("Reserve has non-zero allocation or ctokens so cannot be removed")]
    ReserveHasNonZeroAllocationOrCTokens,

    #[msg("Deposit amount is greater than requested amount")]
    DepositAmountGreaterThanRequestedAmount,
}