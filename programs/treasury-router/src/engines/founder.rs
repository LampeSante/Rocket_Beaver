use anchor_lang::prelude::*;

use crate::{errors::TreasuryRouterError, state::FounderState};

/// Result returned by the founder allocation engine.
pub struct FounderAllocation {
    /// Amount actually credited to the founder.
    pub founder_amount: u64,

    /// Amount redirected to Liquidity because of the cap
    /// or because founder rewards are disabled.
    pub liquidity_overflow_amount: u64,
}

///
/// Founder allocation engine.
///
/// Rules:
///
/// - If founder rewards are disabled:
///     100% goes to Liquidity.
///
/// - If the accounting period has expired:
///     automatically reset.
///
/// - If the requested amount is below the remaining cap:
///     founder receives everything.
///
/// - If the cap would be exceeded:
///     founder receives only the remaining allowance,
///     with the excess redirected to Liquidity.
///
pub fn allocate(founder: &mut FounderState, requested: u64, now: i64) -> Result<FounderAllocation> {
    //
    // Founder disabled
    //
    if !founder.enabled {
        return Ok(FounderAllocation {
            founder_amount: 0,
            liquidity_overflow_amount: requested,
        });
    }

    //
    // Automatic accounting-period reset
    //
    let period_end = founder
        .period_started_at
        .checked_add(founder.period_duration)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    if now >= period_end {
        founder.period_started_at = now;
        founder.earned_current_period = 0;
    }

    //
    // Remaining founder allowance
    //
    let remaining = founder
        .period_cap
        .saturating_sub(founder.earned_current_period);

    //
    // Entire allocation fits under the cap.
    //
    if requested <= remaining {
        founder.earned_current_period = founder
            .earned_current_period
            .checked_add(requested)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        founder.lifetime_earned = founder
            .lifetime_earned
            .checked_add(requested)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        return Ok(FounderAllocation {
            founder_amount: requested,
            liquidity_overflow_amount: 0,
        });
    }

    //
    // Founder reaches the cap.
    //
    founder.earned_current_period = founder.period_cap;

    founder.lifetime_earned = founder
        .lifetime_earned
        .checked_add(remaining)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let overflow = requested
        .checked_sub(remaining)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    Ok(FounderAllocation {
        founder_amount: remaining,
        liquidity_overflow_amount: overflow,
    })
}
