use anchor_lang::prelude::*;

use crate::{
    engines::{dam::DamLevel, waterfall::WaterfallStage},
    errors::TreasuryRouterError,
    state::TreasuryState,
};

/// Maximum possible Beaver Score.
pub const MAX_BEAVER_SCORE: u16 = 1_000;

/// Maximum score contributed by the Survival Waterfall.
pub const WATERFALL_MAX_POINTS: u16 = 350;

/// Maximum score contributed by the Dam.
pub const DAM_MAX_POINTS: u16 = 250;

/// Maximum score contributed by reserve coverage.
pub const RESERVE_MAX_POINTS: u16 = 250;

/// Maximum score contributed by treasury accounting health.
pub const ACCOUNTING_MAX_POINTS: u16 = 150;

/// Result returned by the Beaver Score Engine.
pub struct BeaverScoreEvaluation {
    /// Final protocol health score from 0 to 1,000.
    pub total_score: u16,

    /// Score contributed by the Survival Waterfall.
    pub waterfall_points: u16,

    /// Score contributed by the Dam level.
    pub dam_points: u16,

    /// Score contributed by reserve coverage.
    pub reserve_points: u16,

    /// Score contributed by treasury accounting health.
    pub accounting_points: u16,

    /// Reserve ratio supplied to the score calculation.
    pub reserve_ratio_bps: u16,
}

/// Calculates the current Beaver Score.
///
/// Beaver Score v1 is deterministic and uses only internal protocol state.
/// External market, liquidity, volume, oracle, volatility, and holder data
/// will be introduced in later versions.
pub fn evaluate(
    treasury: &TreasuryState,
    reserve_ratio_bps: u16,
    waterfall_stage: WaterfallStage,
    dam_level: DamLevel,
) -> Result<BeaverScoreEvaluation> {
    let waterfall_points = score_waterfall(waterfall_stage);
    let dam_points = score_dam(dam_level);
    let reserve_points = score_reserve(reserve_ratio_bps)?;
    let accounting_points = score_accounting(treasury);

    let total_score = waterfall_points
        .checked_add(dam_points)
        .and_then(|value| value.checked_add(reserve_points))
        .and_then(|value| value.checked_add(accounting_points))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        total_score <= MAX_BEAVER_SCORE,
        TreasuryRouterError::ArithmeticOverflow
    );

    Ok(BeaverScoreEvaluation {
        total_score,
        waterfall_points,
        dam_points,
        reserve_points,
        accounting_points,
        reserve_ratio_bps,
    })
}

/// Scores the current Survival Waterfall condition.
fn score_waterfall(stage: WaterfallStage) -> u16 {
    match stage {
        WaterfallStage::Normal => WATERFALL_MAX_POINTS,
        WaterfallStage::Caution => 280,
        WaterfallStage::Defensive => 210,
        WaterfallStage::Survival => 105,
        WaterfallStage::Emergency => 0,
    }
}

/// Scores the current Dam level.
fn score_dam(level: DamLevel) -> u16 {
    match level {
        DamLevel::Overflow => DAM_MAX_POINTS,
        DamLevel::Normal => 188,
        DamLevel::Controlled => 125,
        DamLevel::Restricted => 63,
        DamLevel::Filling => 0,
    }
}

/// Scores reserve coverage proportionally.
///
/// A reserve ratio of 10,000 basis points receives the full 250 points.
/// Ratios above 10,000 basis points are capped at the maximum score.
fn score_reserve(reserve_ratio_bps: u16) -> Result<u16> {
    let capped_ratio = reserve_ratio_bps.min(10_000);

    let numerator = u32::from(capped_ratio)
        .checked_mul(u32::from(RESERVE_MAX_POINTS))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let points = numerator
        .checked_div(10_000)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    u16::try_from(points).map_err(|_| TreasuryRouterError::ArithmeticOverflow.into())
}

/// Scores treasury accounting health.
///
/// 100 points are awarded when all received fees have been allocated.
/// 50 points are awarded once at least one processing epoch has completed.
fn score_accounting(treasury: &TreasuryState) -> u16 {
    let integrity_points = if treasury.total_fees_received == treasury.total_fees_allocated {
        100
    } else {
        0
    };

    let activity_points = if treasury.processing_epoch > 0 { 50 } else { 0 };

    integrity_points + activity_points
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_treasury() -> TreasuryState {
        TreasuryState {
            version: 1,
            protocol: Pubkey::default(),
            settlement_mint: Pubkey::default(),
            settlement_vault: Pubkey::default(),
            total_fees_received: 1_000,
            total_fees_allocated: 1_000,
            pending_reserve: 300,
            pending_buyback_burn: 200,
            pending_liquidity: 200,
            pending_company: 200,
            pending_founder: 100,
            lifetime_reserve: 300,
            lifetime_buyback_burn: 200,
            lifetime_liquidity: 200,
            lifetime_company: 200,
            lifetime_founder: 100,
            released_reserve: 0,
            released_buyback_burn: 0,
            released_liquidity: 0,
            released_company: 0,
            released_founder: 0,
            last_processed_at: 1,
            processing_epoch: 1,
            waterfall_stage: 0,
            buybacks_paused: false,
            bump: 255,
            reserved: [0; 24],
        }
    }

    #[test]
    fn ideal_protocol_scores_one_thousand() {
        let treasury = test_treasury();

        let result = evaluate(
            &treasury,
            10_000,
            WaterfallStage::Normal,
            DamLevel::Overflow,
        )
        .unwrap();

        assert_eq!(result.total_score, 1_000);
        assert_eq!(result.waterfall_points, 350);
        assert_eq!(result.dam_points, 250);
        assert_eq!(result.reserve_points, 250);
        assert_eq!(result.accounting_points, 150);
    }

    #[test]
    fn current_initial_allocation_scores_correctly() {
        let treasury = test_treasury();

        let result =
            evaluate(&treasury, 3_000, WaterfallStage::Normal, DamLevel::Overflow).unwrap();

        assert_eq!(result.reserve_points, 75);
        assert_eq!(result.total_score, 825);
    }

    #[test]
    fn emergency_protocol_scores_only_accounting_points() {
        let treasury = test_treasury();

        let result = evaluate(&treasury, 0, WaterfallStage::Emergency, DamLevel::Filling).unwrap();

        assert_eq!(result.waterfall_points, 0);
        assert_eq!(result.dam_points, 0);
        assert_eq!(result.reserve_points, 0);
        assert_eq!(result.accounting_points, 150);
        assert_eq!(result.total_score, 150);
    }

    #[test]
    fn inconsistent_accounting_loses_integrity_points() {
        let mut treasury = test_treasury();
        treasury.total_fees_allocated = 900;

        let result =
            evaluate(&treasury, 3_000, WaterfallStage::Normal, DamLevel::Overflow).unwrap();

        assert_eq!(result.accounting_points, 50);
        assert_eq!(result.total_score, 725);
    }

    #[test]
    fn reserve_score_is_capped() {
        assert_eq!(score_reserve(10_000).unwrap(), 250);
        assert_eq!(score_reserve(u16::MAX).unwrap(), 250);
    }
}
