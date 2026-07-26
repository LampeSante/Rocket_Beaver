use anchor_lang::prelude::*;

use crate::{
    engines::{
        dam::{self, DamLevel},
        waterfall::{self, WaterfallStage},
    },
    errors::TreasuryRouterError,
    state::{ProtocolState, TreasuryState},
};

/// Treasury bucket requesting permission to release funds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseBucket {
    Reserve,
    BuybackBurn,
    Liquidity,
    Company,
    Founder,
}

/// Result returned by the central execution-security gate.
pub struct ExecutionAuthorization {
    pub bucket: ReleaseBucket,
    pub requested_amount: u64,
    pub pending_balance: u64,
    pub maximum_release: u64,
    pub waterfall_stage: WaterfallStage,
    pub dam_level: DamLevel,
    pub release_bps: u16,
    pub reserve_ratio_bps: u16,
}

/// Authorizes a treasury release through the mandatory RBVR security layer.
///
/// Account ownership, PDA seeds, signer authority, mint validation, and
/// protocol-to-treasury linkage must be enforced by the Anchor instruction
/// account constraints that call this function.
pub fn authorize_release(
    protocol: &ProtocolState,
    treasury: &TreasuryState,
    bucket: ReleaseBucket,
    requested_amount: u64,
) -> Result<ExecutionAuthorization> {
    require!(!protocol.paused, TreasuryRouterError::ProtocolPaused);

    require!(
        requested_amount > 0,
        TreasuryRouterError::InvalidReleaseAmount
    );

    validate_accounting_integrity(treasury)?;

    if bucket == ReleaseBucket::BuybackBurn {
        require!(
            !treasury.buybacks_paused,
            TreasuryRouterError::BuybacksPaused
        );
    }

    let waterfall_evaluation = waterfall::evaluate(treasury)?;
    let dam_evaluation = dam::evaluate(waterfall_evaluation.stage);

    require!(
        dam_evaluation.release_bps <= 10_000,
        TreasuryRouterError::InvalidDamReleaseRate
    );

    require!(
        dam_evaluation.level != DamLevel::Filling,
        TreasuryRouterError::DamClosed
    );

    let pending_balance = pending_balance(treasury, bucket);

    require!(
        requested_amount <= pending_balance,
        TreasuryRouterError::InsufficientPendingBalance
    );

    let maximum_release = calculate_maximum_release(pending_balance, dam_evaluation.release_bps)?;

    require!(
        requested_amount <= maximum_release,
        TreasuryRouterError::ReleaseLimitExceeded
    );

    Ok(ExecutionAuthorization {
        bucket,
        requested_amount,
        pending_balance,
        maximum_release,
        waterfall_stage: waterfall_evaluation.stage,
        dam_level: dam_evaluation.level,
        release_bps: dam_evaluation.release_bps,
        reserve_ratio_bps: waterfall_evaluation.reserve_ratio_bps,
    })
}

/// Ensures fee accounting has never allocated more than was received.
///
/// Exact equality is required before releases. This prevents funds from moving
/// while accounting is incomplete or inconsistent.
fn validate_accounting_integrity(treasury: &TreasuryState) -> Result<()> {
    require!(
        treasury.total_fees_allocated <= treasury.total_fees_received,
        TreasuryRouterError::AccountingInvariantViolation
    );

    require!(
        treasury.total_fees_allocated == treasury.total_fees_received,
        TreasuryRouterError::AccountingNotSettled
    );

    Ok(())
}

/// Returns the current pending balance for a treasury bucket.
pub fn pending_balance(treasury: &TreasuryState, bucket: ReleaseBucket) -> u64 {
    match bucket {
        ReleaseBucket::Reserve => treasury.pending_reserve,
        ReleaseBucket::BuybackBurn => treasury.pending_buyback_burn,
        ReleaseBucket::Liquidity => treasury.pending_liquidity,
        ReleaseBucket::Company => treasury.pending_company,
        ReleaseBucket::Founder => treasury.pending_founder,
    }
}

/// Calculates the maximum amount releasable under a Dam rate.
///
/// Integer division deliberately rounds down so the security limit can never
/// be exceeded because of rounding.
pub fn calculate_maximum_release(pending_balance: u64, release_bps: u16) -> Result<u64> {
    require!(
        release_bps <= 10_000,
        TreasuryRouterError::InvalidDamReleaseRate
    );

    let numerator = u128::from(pending_balance)
        .checked_mul(u128::from(release_bps))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let maximum_release = numerator
        .checked_div(10_000)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    u64::try_from(maximum_release).map_err(|_| TreasuryRouterError::ArithmeticOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_dam_allows_full_pending_balance() {
        assert_eq!(calculate_maximum_release(1_000, 10_000).unwrap(), 1_000);
    }

    #[test]
    fn normal_dam_allows_seventy_five_percent() {
        assert_eq!(calculate_maximum_release(1_000, 7_500).unwrap(), 750);
    }

    #[test]
    fn controlled_dam_allows_fifty_percent() {
        assert_eq!(calculate_maximum_release(1_000, 5_000).unwrap(), 500);
    }

    #[test]
    fn restricted_dam_allows_twenty_five_percent() {
        assert_eq!(calculate_maximum_release(1_000, 2_500).unwrap(), 250);
    }

    #[test]
    fn filling_dam_allows_nothing() {
        assert_eq!(calculate_maximum_release(1_000, 0).unwrap(), 0);
    }

    #[test]
    fn release_calculation_rounds_down() {
        assert_eq!(calculate_maximum_release(3, 2_500).unwrap(), 0);
        assert_eq!(calculate_maximum_release(7, 5_000).unwrap(), 3);
    }
}
