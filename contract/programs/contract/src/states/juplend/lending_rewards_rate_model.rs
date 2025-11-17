use anchor_lang::prelude::*;

#[account]
pub struct LendingRewardsRateModel {
    /// @dev mint address
    pub mint: Pubkey,

    /// @dev tvl below which rewards rate is 0. If current TVL is below this value, triggering `update_rate()` on the fToken
    /// might bring the total TVL above this cut-off.
    pub start_tvl: u64,

    /// @dev for how long current rewards should run
    pub duration: u64,

    /// @dev when current rewards got started
    pub start_time: u64,

    /// @dev current annualized reward based on input params (duration, rewardAmount)
    pub yearly_reward: u64,

    /// @dev Duration for the next rewards phase
    pub next_duration: u64,

    /// @dev Amount of rewards for the next phase
    pub next_reward_amount: u64,

    pub bump: u8,
}
