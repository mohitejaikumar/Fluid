
pub fn calculate_shares_to_mint(
    deposit_amount: u64,
    total_shares: u64,
    total_deposits: u64,
) -> u64 {
    if total_shares == 0 || total_deposits == 0 {
        deposit_amount   // 1:1 ratio
    } else {
        deposit_amount
            .checked_mul(total_shares)
            .and_then(|v| v.checked_div(total_deposits))
            .unwrap_or(deposit_amount)
    }
}