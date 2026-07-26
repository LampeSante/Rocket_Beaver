use anchor_lang::prelude::*;

use crate::{errors::TreasuryRouterError, state::TreasuryState};

/// Breakdown of a reserve deposit processed by the Reserve Engine.
pub struct ReserveDeposit {
    /// Normal reserve allocation (includes any rounding remainder).
    pub normal_amount: u64,

    /// Total amount credited to reserve.
    pub total_amount: u64,
}

/// Credits the immutable Reserve allocation.
///
/// Reserve ONLY receives:
///
/// - 30% Beavernomics allocation
/// - rounding remainder
///
/// Founder and Company overflow are handled entirely by the
/// Liquidity Engine.
pub fn deposit_fee_allocation(
    treasury: &mut TreasuryState,
    normal_amount: u64,
) -> Result<ReserveDeposit> {
    treasury.pending_reserve = treasury
        .pending_reserve
        .checked_add(normal_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.lifetime_reserve = treasury
        .lifetime_reserve
        .checked_add(normal_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    Ok(ReserveDeposit {
        normal_amount,
        total_amount: normal_amount,
    })
}
