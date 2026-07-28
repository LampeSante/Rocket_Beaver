use crate::engines::waterfall::WaterfallStage;

/// Dam operating levels.
///
/// The numeric values are stored in ProtocolState and therefore form part
/// of the on-chain interface. Do not reorder them without a state migration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DamLevel {
    /// The Dam is closed while protocol reserves are rebuilt.
    Filling = 0,

    /// Only 25% of normal capital release is permitted.
    Restricted = 1,

    /// Up to 50% of normal capital release is permitted.
    Controlled = 2,

    /// Up to 75% of normal capital release is permitted.
    Normal = 3,

    /// Full capital release is permitted.
    Overflow = 4,
}

impl DamLevel {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Filling => "FILLING",
            Self::Restricted => "RESTRICTED",
            Self::Controlled => "CONTROLLED",
            Self::Normal => "NORMAL",
            Self::Overflow => "OVERFLOW",
        }
    }

    /// Maximum capital-release rate associated with this Dam level.
    ///
    /// This is informational in Dam v1. It will be enforced by the
    /// adaptive allocation layer in a later milestone.
    pub fn release_bps(self) -> u16 {
        match self {
            Self::Filling => 0,
            Self::Restricted => 2_500,
            Self::Controlled => 5_000,
            Self::Normal => 7_500,
            Self::Overflow => 10_000,
        }
    }
}

/// Result returned by the Dam Engine.
pub struct DamEvaluation {
    /// Dam level selected from the current Waterfall condition.
    pub level: DamLevel,

    /// Maximum release rate associated with the selected level.
    pub release_bps: u16,

    /// Waterfall stage used to determine the Dam level.
    pub waterfall_stage: WaterfallStage,
}

/// Evaluates the Dam level from the current Survival Waterfall stage.
///
/// Dam v1 uses a deterministic one-to-one mapping:
///
/// NORMAL    -> OVERFLOW
/// CAUTION   -> NORMAL
/// DEFENSIVE -> CONTROLLED
/// SURVIVAL  -> RESTRICTED
/// EMERGENCY -> FILLING
pub fn evaluate(waterfall_stage: WaterfallStage) -> DamEvaluation {
    evaluate_adaptive(waterfall_stage, 750)
}

/// Evaluates the Dam using the Waterfall stage and the non-circular
/// Pre-Dam Health Score.
///
/// The Waterfall establishes the least-permissive baseline required by
/// reserve conditions. The health score may tighten that baseline, but may
/// never make the Dam more permissive than the Waterfall allows.
pub fn evaluate_adaptive(
    waterfall_stage: WaterfallStage,
    pre_dam_health_score: u16,
) -> DamEvaluation {
    let waterfall_level = match waterfall_stage {
        WaterfallStage::Normal => DamLevel::Overflow,
        WaterfallStage::Caution => DamLevel::Normal,
        WaterfallStage::Defensive => DamLevel::Controlled,
        WaterfallStage::Survival => DamLevel::Restricted,
        WaterfallStage::Emergency => DamLevel::Filling,
    };

    let health_level = match pre_dam_health_score {
        650..=u16::MAX => DamLevel::Overflow,
        500..=649 => DamLevel::Normal,
        350..=499 => DamLevel::Controlled,
        200..=349 => DamLevel::Restricted,
        _ => DamLevel::Filling,
    };

    // Lower numeric values are more restrictive. The Adaptive Dam therefore
    // selects the more restrictive result and can never weaken Waterfall
    // protection.
    let level = if waterfall_level.as_u8() <= health_level.as_u8() {
        waterfall_level
    } else {
        health_level
    };

    DamEvaluation {
        level,
        release_bps: level.release_bps(),
        waterfall_stage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_waterfall_opens_dam_fully() {
        let result = evaluate(WaterfallStage::Normal);

        assert_eq!(result.level, DamLevel::Overflow);
        assert_eq!(result.release_bps, 10_000);
    }

    #[test]
    fn caution_waterfall_uses_normal_dam_level() {
        let result = evaluate(WaterfallStage::Caution);

        assert_eq!(result.level, DamLevel::Normal);
        assert_eq!(result.release_bps, 7_500);
    }

    #[test]
    fn defensive_waterfall_uses_controlled_dam_level() {
        let result = evaluate(WaterfallStage::Defensive);

        assert_eq!(result.level, DamLevel::Controlled);
        assert_eq!(result.release_bps, 5_000);
    }

    #[test]
    fn survival_waterfall_restricts_dam() {
        let result = evaluate(WaterfallStage::Survival);

        assert_eq!(result.level, DamLevel::Restricted);
        assert_eq!(result.release_bps, 2_500);
    }

    #[test]
    fn emergency_waterfall_closes_dam() {
        let result = evaluate(WaterfallStage::Emergency);

        assert_eq!(result.level, DamLevel::Filling);
        assert_eq!(result.release_bps, 0);
    }
}

#[cfg(test)]
mod adaptive_dam_tests {
    use super::*;

    #[test]
    fn healthy_normal_protocol_allows_full_release() {
        let result = evaluate_adaptive(WaterfallStage::Normal, 750);

        assert_eq!(result.level, DamLevel::Overflow);
        assert_eq!(result.release_bps, 10_000);
    }

    #[test]
    fn weak_health_tightens_a_normal_waterfall() {
        let result = evaluate_adaptive(WaterfallStage::Normal, 420);

        assert_eq!(result.level, DamLevel::Controlled);
        assert_eq!(result.release_bps, 5_000);
    }

    #[test]
    fn health_score_can_never_weaken_waterfall_protection() {
        let result = evaluate_adaptive(WaterfallStage::Survival, 750);

        assert_eq!(result.level, DamLevel::Restricted);
        assert_eq!(result.release_bps, 2_500);
    }

    #[test]
    fn critically_low_health_closes_the_dam() {
        let result = evaluate_adaptive(WaterfallStage::Normal, 199);

        assert_eq!(result.level, DamLevel::Filling);
        assert_eq!(result.release_bps, 0);
    }

    #[test]
    fn emergency_waterfall_always_closes_the_dam() {
        let result = evaluate_adaptive(WaterfallStage::Emergency, 750);

        assert_eq!(result.level, DamLevel::Filling);
        assert_eq!(result.release_bps, 0);
    }
}
