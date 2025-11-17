
pub fn calculate_usdc_for_shares(
    shares: u64,
    total_shares: u64,
    total_deposits: u64,
) -> u64 {
    if total_shares == 0 {
        return 0;
    }
    shares
        .checked_mul(total_deposits)
        .and_then(|v| v.checked_div(total_shares))
        .unwrap_or(0)
}