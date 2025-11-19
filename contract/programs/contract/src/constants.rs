pub const BPS_BASE: u16 = 10000;
pub const MIN_OPERATE_AMOUNT: u64 = 1000;

/// Kamino VaultState discriminator
pub const VAULT_STATE_DISCRIMINATOR: [u8; 8] = [228, 196, 82, 165, 98, 210, 235, 152];
/// Slots per year for interest calculation (2 slots/sec * 60 * 60 * 24 * 365)
pub const SLOTS_PER_YEAR: u128 = 63_072_000;
