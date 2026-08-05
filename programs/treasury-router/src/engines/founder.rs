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
///   100% goes to Liquidity.
///
/// - If the accounting period has expired:
///   automatically reset.
///
/// - If the requested amount is below the remaining cap:
///   founder receives everything.
///
/// - If the cap would be exceeded:
///   founder receives only the remaining allowance,
///   with the excess redirected to Liquidity.
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


/// Calculates Founder compensation using a progressive marginal schedule.
///
/// The rate decreases as eligible protocol volume increases.
///
/// Tier schedule:
///
/// 0 - 1M units       : 10%
/// 1M - 10M units     : 7.5%
/// 10M - 100M units   : 5%
/// Above 100M         : 2.5%
///
/// This only calculates the requested founder amount.
/// The USD cap engine remains responsible for final enforcement.
pub fn calculate_progressive_request(
    founder: &mut FounderState,
    volume: u64,
    _founder_bps: u16,
) -> Result<u64> {

    let mut remaining = volume;
    let mut compensation = 0u64;

    let tiers = [
        (1_000_000u64, 1000u64),
        (9_000_000u64, 750u64),
        (90_000_000u64, 500u64),
    ];

    for (limit, rate) in tiers {

        let applied = remaining.min(limit);

        compensation = compensation
            .checked_add(
                applied
                    .checked_mul(rate)
                    .ok_or(TreasuryRouterError::ArithmeticOverflow)?
                    .checked_div(10_000)
                    .ok_or(TreasuryRouterError::ArithmeticOverflow)?
            )
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        remaining = remaining
            .checked_sub(applied)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        if remaining == 0 {
            break;
        }
    }

    if remaining > 0 {

        compensation = compensation
            .checked_add(
                remaining
                    .checked_mul(250)
                    .ok_or(TreasuryRouterError::ArithmeticOverflow)?
                    .checked_div(10_000)
                    .ok_or(TreasuryRouterError::ArithmeticOverflow)?
            )
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;
    }


    founder.current_tier =
        if volume > 100_000_000 {
            3
        } else if volume > 10_000_000 {
            2
        } else if volume > 1_000_000 {
            1
        } else {
            0
        };


    Ok(compensation)
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn progressive_schedule_reduces_rate_with_volume() {

        let mut founder = FounderState {
            version: 1,
            protocol: Pubkey::default(),
            recipient: Pubkey::default(),
            period_cap: u64::MAX,
            earned_current_period: 0,
            lifetime_earned: 0,
            period_started_at: 0,
            period_duration: 31_536_000,
            current_tier: 0,
            enabled: true,
            bump: 0,
            reserved: [0;64],
        };


        let small =
            calculate_progressive_request(
                &mut founder,
                1_000_000,
                1000,
            )
            .unwrap();


        let large =
            calculate_progressive_request(
                &mut founder,
                200_000_000,
                1000,
            )
            .unwrap();


        assert!(small > 0);
        assert!(large > small);


        // Effective rate must decline as volume grows.
        let small_rate =
            small as f64 / 1_000_000f64;

        let large_rate =
            large as f64 / 200_000_000f64;


        assert!(large_rate < small_rate);
    }
}
