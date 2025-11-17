use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount};

use crate::{errors::AggregatorError, helpers::token_reserve_helper::get_supply_exchange_price, states::juplend::{lending::Lending, lending_rewards_rate_model::LendingRewardsRateModel}};

const EXCHANGE_PRICES_PRECISION: u128 = 1000000000000;
const SECONDS_PER_YEAR: u128 = 31536000;
const MAX_REWARDS_RATE: u128 = 50000000000000;


pub fn get_juplend_balance<'info>(
    token_reserve: &AccountInfo<'info>,
    lending: &Account<'info, Lending>,
    rewards_rate_model: &Account<'info, LendingRewardsRateModel>,
    fusdc_token_account: &InterfaceAccount<'info, TokenAccount>,
) -> Result<u64> {
    let supply_exchange_price = get_supply_exchange_price(&token_reserve.data.borrow());
    let program_ftoken = fusdc_token_account.amount;
    let juplend_balance = convert_to_asset(
        program_ftoken,
        supply_exchange_price,
        lending,
        rewards_rate_model
    )?;
    
    Ok(juplend_balance as u64)
}


pub struct RewardsRate {
    pub rate: u128,
    pub rewards_ended: bool,
    pub rewards_start_time: u64,
}

pub fn get_rewards_rate<'info>(
    total_assets: u64,
    rewards_rate_model: &Account<'info, LendingRewardsRateModel>,
) -> Result<RewardsRate> {
    if total_assets > rewards_rate_model.start_tvl {
        return Ok(RewardsRate {
            rate: 0,
            rewards_ended: false,
            rewards_start_time: rewards_rate_model.start_time,
        })
    }
    let rate = (rewards_rate_model.yearly_reward as u128)
                     .checked_mul(EXCHANGE_PRICES_PRECISION)
                     .ok_or(AggregatorError::MathOverflow)?
                     .checked_div(total_assets as u128)
                     .ok_or(AggregatorError::MathOverflow)?;
    
    if rate > MAX_REWARDS_RATE {
        return Ok(RewardsRate {
            rate: MAX_REWARDS_RATE,
            rewards_ended: false,
            rewards_start_time: rewards_rate_model.start_time,
        })
    }
    Ok(RewardsRate {
        rate,
        rewards_ended: false,
        rewards_start_time: rewards_rate_model.start_time,
    })
}


pub fn get_new_exchange_price<'info>(
    supply_exchange_price: u64,
    total_supply_ctoken: u64,
    lending: &Account<'info, Lending>,
    rewards_rate_model: &Account<'info, LendingRewardsRateModel>,
) -> Result<u128> {
    let old_token_exchange_price = lending.token_exchange_price as u128;

    let old_liquidity_exchange_price = lending.liquidity_exchange_price as u128;

    let mut total_return_percent = 0u128;

    let total_assets = old_token_exchange_price
                            .checked_mul(total_supply_ctoken as u128)
                            .ok_or(AggregatorError::MathOverflow)?
                            .checked_div(EXCHANGE_PRICES_PRECISION)
                            .ok_or(AggregatorError::MathOverflow)?;

    let mut rewards_rate = get_rewards_rate(total_assets as u64, rewards_rate_model)?;

    if rewards_rate.rate > MAX_REWARDS_RATE {
        rewards_rate.rate = 0u128;
    }

    let mut last_update_time = lending.last_update_timestamp;
    if last_update_time < rewards_rate.rewards_start_time {
        last_update_time = rewards_rate.rewards_start_time;
    }

    let current_timestamp = Clock::get()?.unix_timestamp as u128;
    
    total_return_percent = rewards_rate.rate
                                .checked_mul(
                                    current_timestamp
                                    .checked_sub(last_update_time as u128)
                                    .ok_or(AggregatorError::MathOverflow)?
                                )
                                .ok_or(AggregatorError::MathOverflow)?
                                .checked_div(SECONDS_PER_YEAR)
                                .ok_or(AggregatorError::MathOverflow)?;

    let delta = (supply_exchange_price as u128)
                       .checked_sub(old_liquidity_exchange_price)
                       .ok_or(AggregatorError::MathOverflow)?;

    total_return_percent = total_return_percent.checked_add(
        delta.checked_mul(100000000000000)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_div(old_liquidity_exchange_price)
        .ok_or(AggregatorError::MathOverflow)?
    ).ok_or(AggregatorError::MathOverflow)?;

    let new_token_exchange_price = old_token_exchange_price.checked_add(
        old_token_exchange_price.checked_mul(total_return_percent).ok_or(AggregatorError::MathOverflow)?
        .checked_div(100000000000000)
        .ok_or(AggregatorError::MathOverflow)?
    ).ok_or(AggregatorError::MathOverflow)?;

    Ok(new_token_exchange_price)

}


pub fn convert_to_asset<'info>(
    fusdc_amount: u64,
    supply_exchange_price: u64,
    lending: &Account<'info, Lending>,
    rewards_rate_model: &Account<'info, LendingRewardsRateModel>,
) -> Result<u64> {
    let new_exchange_price = get_new_exchange_price(supply_exchange_price, fusdc_amount, lending, rewards_rate_model)?;

    let usdc_assets = (fusdc_amount as u128).checked_mul(new_exchange_price)
        .ok_or(AggregatorError::MathOverflow)?
        .checked_div(EXCHANGE_PRICES_PRECISION)
        .ok_or(AggregatorError::MathOverflow)? as u64;

    Ok(usdc_assets)
}


