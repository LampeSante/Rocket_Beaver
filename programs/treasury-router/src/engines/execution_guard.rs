use anchor_lang::prelude::*;

use crate::{
    engines::{
        beaver_score,
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

    require!(
        treasury.execution_accounting_is_valid(),
        TreasuryRouterError::AccountingInvariantViolation
    );

    if bucket == ReleaseBucket::BuybackBurn {
        require!(
            !treasury.buybacks_paused,
            TreasuryRouterError::BuybacksPaused
        );
    }

    let waterfall_evaluation = waterfall::evaluate(treasury)?;

    let pre_dam_health_score = beaver_score::pre_dam_health_score(
        treasury,
        waterfall_evaluation.reserve_ratio_bps,
        waterfall_evaluation.stage,
    )?;

    let dam_evaluation = dam::evaluate_adaptive(waterfall_evaluation.stage, pre_dam_health_score);

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

    fn valid_protocol() -> ProtocolState {
        ProtocolState {
            version: 1,
            authority: Pubkey::new_unique(),
            protocol_config: Pubkey::new_unique(),
            treasury_state: Pubkey::new_unique(),
            reserve_state: Pubkey::new_unique(),
            liquidity_state: Pubkey::new_unique(),
            founder_state: Pubkey::new_unique(),
            company_state: Pubkey::new_unique(),
            buyback_state: Pubkey::new_unique(),
            beaver_score: 10_000,
            dam_level: DamLevel::Overflow.as_u8(),
            paused: false,
            bump: 255,
            initialized_at: 1,
            reserved: [0; 64],
        }
    }

    /// Creates fully settled and internally consistent accounting.
    ///
    /// Pending balances:
    /// - Reserve: 1,000
    /// - Buyback: 100
    /// - Liquidity: 100
    /// - Company: 100
    /// - Founder: 100
    ///
    /// Reserve ratio:
    /// 1,000 / 1,400 = 71.42%, producing:
    /// - WaterfallStage::Normal
    /// - DamLevel::Overflow
    /// - 100% release limit
    fn valid_treasury() -> TreasuryState {
        TreasuryState {
            version: 1,
            protocol: Pubkey::new_unique(),
            settlement_mint: Pubkey::new_unique(),
            settlement_vault: Pubkey::new_unique(),

            total_fees_received: 1_400,
            total_fees_allocated: 1_400,

            pending_reserve: 1_000,
            pending_buyback_burn: 100,
            pending_liquidity: 100,
            pending_company: 100,
            pending_founder: 100,

            lifetime_reserve: 1_000,
            lifetime_buyback_burn: 100,
            lifetime_liquidity: 100,
            lifetime_company: 100,
            lifetime_founder: 100,

            released_reserve: 0,
            released_buyback_burn: 0,
            released_liquidity: 0,
            released_company: 0,
            released_founder: 0,

            last_processed_at: 1,
            processing_epoch: 1,
            waterfall_stage: WaterfallStage::Normal.as_u8(),
            buybacks_paused: false,
            bump: 254,
            reserved: [0; 24],
        }
    }

    fn assert_anchor_error(result: Result<ExecutionAuthorization>, expected_error_name: &str) {
        let error = match result {
            Ok(_) => {
                panic!("Expected Anchor error {expected_error_name}, but authorization succeeded.")
            }
            Err(error) => error,
        };

        match error {
            anchor_lang::error::Error::AnchorError(anchor_error) => {
                assert_eq!(
                    anchor_error.error_name, expected_error_name,
                    "Unexpected Anchor error: {anchor_error:?}"
                );
            }
            other => panic!("Expected AnchorError {expected_error_name}, received {other:?}."),
        }
    }

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

    #[test]
    fn rejects_invalid_dam_release_rate() {
        assert_anchor_error(
            calculate_maximum_release(1_000, 10_001).map(|maximum_release| {
                ExecutionAuthorization {
                    bucket: ReleaseBucket::Reserve,
                    requested_amount: 1,
                    pending_balance: 1_000,
                    maximum_release,
                    waterfall_stage: WaterfallStage::Normal,
                    dam_level: DamLevel::Overflow,
                    release_bps: 10_001,
                    reserve_ratio_bps: 10_000,
                }
            }),
            "InvalidDamReleaseRate",
        );
    }

    #[test]
    fn authorizes_valid_reserve_release() {
        let protocol = valid_protocol();
        let treasury = valid_treasury();

        let authorization =
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 750).unwrap();

        assert_eq!(authorization.bucket, ReleaseBucket::Reserve);
        assert_eq!(authorization.requested_amount, 750);
        assert_eq!(authorization.pending_balance, 1_000);
        assert_eq!(authorization.maximum_release, 1_000);
        assert_eq!(authorization.waterfall_stage, WaterfallStage::Normal);
        assert_eq!(authorization.dam_level, DamLevel::Overflow);
        assert_eq!(authorization.release_bps, 10_000);
        assert_eq!(authorization.reserve_ratio_bps, 7_142);
    }

    #[test]
    fn authorizes_valid_buyback_release() {
        let protocol = valid_protocol();
        let treasury = valid_treasury();

        let authorization =
            authorize_release(&protocol, &treasury, ReleaseBucket::BuybackBurn, 100).unwrap();

        assert_eq!(authorization.bucket, ReleaseBucket::BuybackBurn);
        assert_eq!(authorization.requested_amount, 100);
        assert_eq!(authorization.pending_balance, 100);
        assert_eq!(authorization.maximum_release, 100);
    }

    #[test]
    fn rejects_release_when_protocol_is_paused() {
        let mut protocol = valid_protocol();
        let treasury = valid_treasury();

        protocol.paused = true;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
            "ProtocolPaused",
        );
    }

    #[test]
    fn rejects_zero_release_amount() {
        let protocol = valid_protocol();
        let treasury = valid_treasury();

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 0),
            "InvalidReleaseAmount",
        );
    }

    #[test]
    fn rejects_buyback_when_buybacks_are_paused() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.buybacks_paused = true;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::BuybackBurn, 1),
            "BuybacksPaused",
        );
    }

    #[test]
    fn buyback_pause_does_not_block_other_buckets() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.buybacks_paused = true;

        let authorization =
            authorize_release(&protocol, &treasury, ReleaseBucket::Liquidity, 50).unwrap();

        assert_eq!(authorization.bucket, ReleaseBucket::Liquidity);
        assert_eq!(authorization.requested_amount, 50);
    }

    #[test]
    fn rejects_release_exceeding_pending_balance() {
        let protocol = valid_protocol();
        let treasury = valid_treasury();

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Company, 101),
            "InsufficientPendingBalance",
        );
    }

    #[test]
    fn rejects_release_when_dam_is_closed() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.pending_reserve = 40;
        treasury.pending_buyback_burn = 240;
        treasury.pending_liquidity = 240;
        treasury.pending_company = 240;
        treasury.pending_founder = 240;

        treasury.lifetime_reserve = 40;
        treasury.lifetime_buyback_burn = 240;
        treasury.lifetime_liquidity = 240;
        treasury.lifetime_company = 240;
        treasury.lifetime_founder = 240;

        treasury.total_fees_received = 1_000;
        treasury.total_fees_allocated = 1_000;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Company, 1),
            "DamClosed",
        );
    }

    #[test]
    fn rejects_release_exceeding_current_dam_limit() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        // 200 / 1,000 = 20%, which is Caution.
        // Caution maps to a 75% release limit.
        treasury.pending_reserve = 200;
        treasury.pending_buyback_burn = 0;
        treasury.pending_liquidity = 0;
        treasury.pending_company = 800;
        treasury.pending_founder = 0;

        treasury.lifetime_reserve = 200;
        treasury.lifetime_buyback_burn = 0;
        treasury.lifetime_liquidity = 0;
        treasury.lifetime_company = 800;
        treasury.lifetime_founder = 0;

        treasury.total_fees_received = 1_000;
        treasury.total_fees_allocated = 1_000;

        // Company pending balance is 800.
        // 75% of 800 is 600.
        // A request of 601 is within pending but above the Dam limit.
        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Company, 601),
            "ReleaseLimitExceeded",
        );
    }

    #[test]
    fn authorizes_release_at_exact_dam_limit() {
        // The Adaptive Dam tightens this Caution-stage fixture to
        // Controlled: 5,000 BPS of 800 pending units equals 400.
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.pending_reserve = 200;
        treasury.pending_buyback_burn = 0;
        treasury.pending_liquidity = 0;
        treasury.pending_company = 800;
        treasury.pending_founder = 0;

        treasury.lifetime_reserve = 200;
        treasury.lifetime_buyback_burn = 0;
        treasury.lifetime_liquidity = 0;
        treasury.lifetime_company = 800;
        treasury.lifetime_founder = 0;

        treasury.total_fees_received = 1_000;
        treasury.total_fees_allocated = 1_000;

        let authorization =
            authorize_release(&protocol, &treasury, ReleaseBucket::Company, 400).unwrap();

        assert_eq!(authorization.requested_amount, 400);
        assert_eq!(authorization.pending_balance, 800);
        assert_eq!(authorization.maximum_release, 400);
        assert_eq!(authorization.waterfall_stage, WaterfallStage::Caution);
        assert_eq!(authorization.dam_level, DamLevel::Controlled);
        assert_eq!(authorization.release_bps, 5_000);
        assert_eq!(authorization.reserve_ratio_bps, 2_000);
    }

    #[test]
    fn rejects_when_allocated_fees_exceed_received_fees() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.total_fees_received = 1_399;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
            "AccountingInvariantViolation",
        );
    }

    #[test]
    fn rejects_when_total_accounting_is_not_settled() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.total_fees_received = 1_401;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
            "AccountingNotSettled",
        );
    }

    #[test]
    fn rejects_invalid_reserve_bucket_accounting() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        // Lifetime must equal pending + released.
        treasury.lifetime_reserve = 999;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
            "AccountingInvariantViolation",
        );
    }

    #[test]
    fn rejects_invalid_non_requested_bucket_accounting() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        // A corrupted founder bucket must block a reserve release.
        // The guard protects the entire treasury, not only the requested bucket.
        treasury.lifetime_founder = 99;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
            "AccountingInvariantViolation",
        );
    }

    #[test]
    fn pending_balance_returns_correct_bucket_amounts() {
        let treasury = valid_treasury();

        assert_eq!(pending_balance(&treasury, ReleaseBucket::Reserve), 1_000);
        assert_eq!(pending_balance(&treasury, ReleaseBucket::BuybackBurn), 100);
        assert_eq!(pending_balance(&treasury, ReleaseBucket::Liquidity), 100);
        assert_eq!(pending_balance(&treasury, ReleaseBucket::Company), 100);
        assert_eq!(pending_balance(&treasury, ReleaseBucket::Founder), 100);
    }
}
