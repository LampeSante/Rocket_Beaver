use anchor_lang::prelude::*;

use crate::{errors::TreasuryRouterError, state::TreasuryState};

/// Survival Waterfall operating stages.
///
/// Lower numeric values represent healthier protocol conditions.
/// The stored u8 values are part of the on-chain protocol interface,
/// so their ordering must not be changed without a state migration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum WaterfallStage {
    /// Reserve represents at least 30% of pending treasury allocations.
    Normal = 0,

    /// Reserve represents between 20% and 29.99%.
    Caution = 1,

    /// Reserve represents between 12.5% and 19.99%.
    Defensive = 2,

    /// Reserve represents between 5% and 12.49%.
    Survival = 3,

    /// Reserve represents less than 5%.
    Emergency = 4,
}

impl WaterfallStage {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Caution => "CAUTION",
            Self::Defensive => "DEFENSIVE",
            Self::Survival => "SURVIVAL",
            Self::Emergency => "EMERGENCY",
        }
    }
}

/// Result returned after evaluating protocol treasury health.
pub struct WaterfallEvaluation {
    /// Stage selected by the Waterfall engine.
    pub stage: WaterfallStage,

    /// Reserve share of total pending allocations, measured in basis points.
    pub reserve_ratio_bps: u16,

    /// Combined amount currently pending across every treasury bucket.
    pub total_pending: u64,

    /// Current pending reserve balance used by the evaluation.
    pub pending_reserve: u64,
}

/// Normal operation requires reserve coverage of at least 30%.
pub const NORMAL_MIN_RESERVE_BPS: u16 = 3_000;

/// Caution operation requires reserve coverage of at least 20%.
pub const CAUTION_MIN_RESERVE_BPS: u16 = 2_000;

/// Defensive operation requires reserve coverage of at least 12.5%.
pub const DEFENSIVE_MIN_RESERVE_BPS: u16 = 1_250;

/// Survival operation requires reserve coverage of at least 5%.
pub const SURVIVAL_MIN_RESERVE_BPS: u16 = 500;

/// Evaluates the treasury's current reserve strength.
///
/// The first Devnet version uses the reserve share of all pending allocations:
///
/// pending_reserve / total_pending
///
/// When no pending allocations exist, the treasury is classified as Normal
/// because there are no current treasury obligations requiring protection.
pub fn evaluate(treasury: &TreasuryState) -> Result<WaterfallEvaluation> {
    let total_pending = treasury
        .pending_reserve
        .checked_add(treasury.pending_buyback_burn)
        .and_then(|value| value.checked_add(treasury.pending_liquidity))
        .and_then(|value| value.checked_add(treasury.pending_company))
        .and_then(|value| value.checked_add(treasury.pending_founder))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let reserve_ratio_bps = calculate_reserve_ratio_bps(treasury.pending_reserve, total_pending)?;

    let stage = classify(reserve_ratio_bps);

    Ok(WaterfallEvaluation {
        stage,
        reserve_ratio_bps,
        total_pending,
        pending_reserve: treasury.pending_reserve,
    })
}

/// Classifies a reserve ratio into a Survival Waterfall stage.
pub fn classify(reserve_ratio_bps: u16) -> WaterfallStage {
    if reserve_ratio_bps >= NORMAL_MIN_RESERVE_BPS {
        WaterfallStage::Normal
    } else if reserve_ratio_bps >= CAUTION_MIN_RESERVE_BPS {
        WaterfallStage::Caution
    } else if reserve_ratio_bps >= DEFENSIVE_MIN_RESERVE_BPS {
        WaterfallStage::Defensive
    } else if reserve_ratio_bps >= SURVIVAL_MIN_RESERVE_BPS {
        WaterfallStage::Survival
    } else {
        WaterfallStage::Emergency
    }
}

fn calculate_reserve_ratio_bps(pending_reserve: u64, total_pending: u64) -> Result<u16> {
    if total_pending == 0 {
        return Ok(10_000);
    }

    let numerator = u128::from(pending_reserve)
        .checked_mul(10_000)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let ratio = numerator
        .checked_div(u128::from(total_pending))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    u16::try_from(ratio).map_err(|_| TreasuryRouterError::ArithmeticOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_normal_stage() {
        assert_eq!(classify(3_000), WaterfallStage::Normal);
        assert_eq!(classify(10_000), WaterfallStage::Normal);
    }

    #[test]
    fn classifies_caution_stage() {
        assert_eq!(classify(2_000), WaterfallStage::Caution);
        assert_eq!(classify(2_999), WaterfallStage::Caution);
    }

    #[test]
    fn classifies_defensive_stage() {
        assert_eq!(classify(1_250), WaterfallStage::Defensive);
        assert_eq!(classify(1_999), WaterfallStage::Defensive);
    }

    #[test]
    fn classifies_survival_stage() {
        assert_eq!(classify(500), WaterfallStage::Survival);
        assert_eq!(classify(1_249), WaterfallStage::Survival);
    }

    #[test]
    fn classifies_emergency_stage() {
        assert_eq!(classify(0), WaterfallStage::Emergency);
        assert_eq!(classify(499), WaterfallStage::Emergency);
    }
}
