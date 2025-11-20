pub const BPS_BASE: u16 = 10000;
pub const MIN_OPERATE_AMOUNT: u64 = 1000;

/// Kamino VaultState discriminator
pub const VAULT_STATE_DISCRIMINATOR: [u8; 8] = [228, 196, 82, 165, 98, 210, 235, 152];
/// Slots per year for interest calculation (2 slots/sec * 60 * 60 * 24 * 365)
pub const SLOTS_PER_YEAR: u128 = 63_072_000;

pub const MAX_REWARDS_TOKENS: usize = 10;

/// WAD scale = 10^18
pub const WAD: u128 = 1_000_000_000_000_000_000;

pub const EXCHANGE_PRICES_PRECISION: u128 = 1000000000000;
pub const SECONDS_PER_YEAR: u128 = 31536000;
pub const MAX_REWARDS_RATE: u128 = 50000000000000;