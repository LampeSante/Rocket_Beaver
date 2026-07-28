#![no_main]

use libfuzzer_sys::fuzz_target;
use treasury_router::engines::{
    dam::{self, DamLevel},
    waterfall::WaterfallStage,
};

fn stage_from_byte(value: u8) -> WaterfallStage {
    match value % 5 {
        0 => WaterfallStage::Normal,
        1 => WaterfallStage::Caution,
        2 => WaterfallStage::Defensive,
        3 => WaterfallStage::Survival,
        _ => WaterfallStage::Emergency,
    }
}

fn waterfall_baseline(stage: WaterfallStage) -> DamLevel {
    match stage {
        WaterfallStage::Normal => DamLevel::Overflow,
        WaterfallStage::Caution => DamLevel::Normal,
        WaterfallStage::Defensive => DamLevel::Controlled,
        WaterfallStage::Survival => DamLevel::Restricted,
        WaterfallStage::Emergency => DamLevel::Filling,
    }
}

fn health_level(score: u16) -> DamLevel {
    match score {
        650..=u16::MAX => DamLevel::Overflow,
        500..=649 => DamLevel::Normal,
        350..=499 => DamLevel::Controlled,
        200..=349 => DamLevel::Restricted,
        _ => DamLevel::Filling,
    }
}

fn expected_release_bps(level: DamLevel) -> u16 {
    match level {
        DamLevel::Filling => 0,
        DamLevel::Restricted => 2_500,
        DamLevel::Controlled => 5_000,
        DamLevel::Normal => 7_500,
        DamLevel::Overflow => 10_000,
    }
}

fn more_restrictive(left: DamLevel, right: DamLevel) -> DamLevel {
    if left.as_u8() <= right.as_u8() {
        left
    } else {
        right
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 3 {
        return;
    }

    let stage = stage_from_byte(data[0]);
    let score = u16::from_le_bytes([data[1], data[2]]);

    let result = dam::evaluate_adaptive(stage, score);

    let baseline = waterfall_baseline(stage);
    let score_level = health_level(score);
    let expected_level = more_restrictive(baseline, score_level);

    // Exact agreement with an independent reference model.
    assert_eq!(
        result.level, expected_level,
        "Adaptive Dam disagreed with reference model: stage={stage:?}, score={score}"
    );

    assert_eq!(
        result.waterfall_stage, stage,
        "Adaptive Dam returned the wrong Waterfall stage"
    );

    assert_eq!(
        result.release_bps,
        expected_release_bps(expected_level),
        "Adaptive Dam returned the wrong release rate"
    );

    assert_eq!(
        result.release_bps,
        result.level.release_bps(),
        "Dam evaluation release rate disagreed with DamLevel"
    );

    assert!(
        result.release_bps <= 10_000,
        "Adaptive Dam release exceeded 100%"
    );

    // Lower numeric Dam levels are more restrictive. The health score may
    // tighten the Waterfall baseline, but it may never weaken it.
    assert!(
        result.level.as_u8() <= baseline.as_u8(),
        "Health score weakened Waterfall protection: \
         stage={stage:?}, score={score}, baseline={baseline:?}, result={:?}",
        result.level
    );

    assert!(
        result.release_bps <= baseline.release_bps(),
        "Adaptive Dam allowed a higher release rate than the Waterfall baseline"
    );

    // Determinism: identical inputs must always produce identical output.
    let repeated = dam::evaluate_adaptive(stage, score);

    assert_eq!(repeated.level, result.level);
    assert_eq!(repeated.release_bps, result.release_bps);
    assert_eq!(repeated.waterfall_stage, result.waterfall_stage);

    // A healthier score must never make the Dam more restrictive.
    let healthier_score = score.saturating_add(1);
    let healthier_result = dam::evaluate_adaptive(stage, healthier_score);

    assert!(
        healthier_result.level.as_u8() >= result.level.as_u8(),
        "Increasing health made the Dam more restrictive: \
         stage={stage:?}, score={score}, old={:?}, new={:?}",
        result.level,
        healthier_result.level
    );

    assert!(
        healthier_result.release_bps >= result.release_bps,
        "Increasing health reduced the permitted release rate"
    );

    // Even maximum health cannot override the Waterfall baseline.
    let maximum_health = dam::evaluate_adaptive(stage, u16::MAX);

    assert_eq!(
        maximum_health.level, baseline,
        "Maximum health incorrectly weakened the Waterfall baseline"
    );

    // Zero health must always close the Dam.
    let zero_health = dam::evaluate_adaptive(stage, 0);

    assert_eq!(zero_health.level, DamLevel::Filling);
    assert_eq!(zero_health.release_bps, 0);

    // The legacy deterministic evaluator uses a healthy score of 750.
    let legacy = dam::evaluate(stage);
    let adaptive_healthy = dam::evaluate_adaptive(stage, 750);

    assert_eq!(legacy.level, adaptive_healthy.level);
    assert_eq!(legacy.release_bps, adaptive_healthy.release_bps);
    assert_eq!(legacy.waterfall_stage, adaptive_healthy.waterfall_stage);
});
