use anchor_lang::prelude::*;

use crate::{errors::TreasuryRouterError, state::CompanyState};

/// Result returned by the company allocation engine.
pub struct CompanyAllocation {
    /// Amount actually credited to the company.
    pub company_amount: u64,

    /// Amount redirected to Liquidity because of the cap
    /// or because company allocations are disabled.
    pub liquidity_overflow_amount: u64,
}

///
/// Company allocation engine.
///
/// Rules:
///
/// - If company allocations are disabled:
///     100% goes to Liquidity.
///
/// - If the accounting period has expired:
///     automatically reset.
///
/// - If the requested amount is below the remaining cap:
///     company receives everything.
///
/// - If the cap would be exceeded:
///     company receives only the remaining allowance,
///     with the excess redirected to Liquidity.
///
pub fn allocate(company: &mut CompanyState, requested: u64, now: i64) -> Result<CompanyAllocation> {
    //
    // Company disabled
    //
    if !company.enabled {
        return Ok(CompanyAllocation {
            company_amount: 0,
            liquidity_overflow_amount: requested,
        });
    }

    //
    // Automatic accounting-period reset
    //
    let period_end = company
        .period_started_at
        .checked_add(company.period_duration)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    if now >= period_end {
        company.period_started_at = now;
        company.spent_current_period = 0;
    }

    //
    // Remaining company allowance
    //
    let remaining = company
        .period_cap
        .saturating_sub(company.spent_current_period);

    //
    // Entire allocation fits under the cap.
    //
    if requested <= remaining {
        company.spent_current_period = company
            .spent_current_period
            .checked_add(requested)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        company.lifetime_spent = company
            .lifetime_spent
            .checked_add(requested)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        return Ok(CompanyAllocation {
            company_amount: requested,
            liquidity_overflow_amount: 0,
        });
    }

    //
    // Company reaches the cap.
    //
    company.spent_current_period = company.period_cap;

    company.lifetime_spent = company
        .lifetime_spent
        .checked_add(remaining)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let overflow = requested
        .checked_sub(remaining)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    Ok(CompanyAllocation {
        company_amount: remaining,
        liquidity_overflow_amount: overflow,
    })
}
