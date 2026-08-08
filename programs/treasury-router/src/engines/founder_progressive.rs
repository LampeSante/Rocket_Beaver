use anchor_lang::prelude::*;

use crate::{
    errors::TreasuryRouterError,
    state::founder::FounderState,
};

/// Progressive Founder compensation result.
pub struct ProgressiveFounderAllocation {
    /// Founder amount after progressive rate calculation.
    pub founder_amount: u64,

    /// Excess redirected to liquidity growth.
    pub liquidity_overflow_amount: u64,

    /// Applied marginal rate.
    pub applied_rate_bps: u64,

    /// Current progressive tier.
    pub current_tier: u8,
}


/// Progressive founder compensation engine.
///
/// The rate decreases as cumulative founder volume increases.
/// Absolute founder compensation can continue increasing.
pub fn calculate(
    founder_state: &FounderState,
    requested_amount: u64,
    _now: i64,
) -> Result<ProgressiveFounderAllocation> {

    let lifetime_volume = founder_state.lifetime_earned;

    let (tier, rate_bps) = if lifetime_volume < 1_000_000_000 {
        (0u8, 1000u64)      // 10%
    } else if lifetime_volume < 10_000_000_000 {
        (1u8, 750u64)       // 7.5%
    } else if lifetime_volume < 100_000_000_000 {
        (2u8, 500u64)       // 5%
    } else {
        (3u8, 250u64)       // 2.5%
    };

    let founder_amount = requested_amount
        .checked_mul(rate_bps)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?
        .checked_div(10_000)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let liquidity_overflow = requested_amount
        .checked_sub(founder_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    Ok(ProgressiveFounderAllocation {
        founder_amount,
        liquidity_overflow_amount: liquidity_overflow,
        applied_rate_bps: rate_bps,
        current_tier: tier,
    })
}
