#![no_main]

use anchor_lang::prelude::Pubkey;
use libfuzzer_sys::fuzz_target;
use treasury_router::{
    engines::waterfall::{
        allocation_for_stage, classify, WaterfallStage, CAUTION_MIN_RESERVE_BPS,
        DEFENSIVE_MIN_RESERVE_BPS, NORMAL_MIN_RESERVE_BPS, SURVIVAL_MIN_RESERVE_BPS,
    },
    state::TreasuryState,
};

fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ])
}

fn expected_stage(reserve_ratio_bps: u16) -> WaterfallStage {
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

fuzz_target!(|data: &[u8]| {
    // Five independent u64 pending treasury balances.
    if data.len() < 40 {
        return;
    }

    let pending_reserve = read_u64(data, 0);
    let pending_buyback_burn = read_u64(data, 8);
    let pending_liquidity = read_u64(data, 16);
    let pending_company = read_u64(data, 24);
    let pending_founder = read_u64(data, 32);

    let treasury = TreasuryState {
        version: 1,
        protocol: Pubkey::default(),
        settlement_mint: Pubkey::default(),
        settlement_vault: Pubkey::default(),

        total_fees_received: 0,
        total_fees_allocated: 0,

        pending_reserve,
        pending_buyback_burn,
        pending_liquidity,
        pending_company,
        pending_founder,

        lifetime_reserve: pending_reserve,
        lifetime_buyback_burn: pending_buyback_burn,
        lifetime_liquidity: pending_liquidity,
        lifetime_company: pending_company,
        lifetime_founder: pending_founder,

        released_reserve: 0,
        released_buyback_burn: 0,
        released_liquidity: 0,
        released_company: 0,
        released_founder: 0,

        last_processed_at: 0,
        processing_epoch: 0,
        waterfall_stage: 0,
        buybacks_paused: false,
        bump: 0,
        reserved: [0; 24],
    };

    let expected_total = pending_reserve
        .checked_add(pending_buyback_burn)
        .and_then(|value| value.checked_add(pending_liquidity))
        .and_then(|value| value.checked_add(pending_company))
        .and_then(|value| value.checked_add(pending_founder));

    let evaluation = treasury_router::engines::waterfall::evaluate(&treasury);

    let total_pending = match expected_total {
        Some(total) => total,
        None => {
            assert!(
                evaluation.is_err(),
                "Waterfall accepted overflowing pending balances: \
                 reserve={pending_reserve}, buyback={pending_buyback_burn}, \
                 liquidity={pending_liquidity}, company={pending_company}, \
                 founder={pending_founder}"
            );
            return;
        }
    };

    let evaluation = evaluation.unwrap_or_else(|error| {
        panic!(
            "Waterfall rejected non-overflowing treasury state: \
             reserve={pending_reserve}, buyback={pending_buyback_burn}, \
             liquidity={pending_liquidity}, company={pending_company}, \
             founder={pending_founder}, error={error:?}"
        )
    });

    assert_eq!(
        evaluation.total_pending, total_pending,
        "Waterfall total pending disagreed with checked reference sum"
    );

    assert_eq!(
        evaluation.pending_reserve, pending_reserve,
        "Waterfall reported the wrong pending reserve"
    );

    let expected_ratio = if total_pending == 0 {
        10_000u16
    } else {
        let ratio = (u128::from(pending_reserve) * 10_000u128) / u128::from(total_pending);

        u16::try_from(ratio).expect("reference reserve ratio exceeded u16")
    };

    assert!(
        evaluation.reserve_ratio_bps <= 10_000,
        "reserve ratio exceeded 100%: {}",
        evaluation.reserve_ratio_bps
    );

    assert_eq!(
        evaluation.reserve_ratio_bps, expected_ratio,
        "production reserve-ratio math disagreed with reference model"
    );

    let independently_classified = expected_stage(expected_ratio);

    assert_eq!(
        evaluation.stage, independently_classified,
        "Waterfall selected the wrong treasury-health stage"
    );

    assert_eq!(
        classify(expected_ratio),
        independently_classified,
        "classify() disagreed with the independent threshold model"
    );

    // Determinism: identical state must always produce identical results.
    let repeated = treasury_router::engines::waterfall::evaluate(&treasury).unwrap();

    assert_eq!(repeated.stage, evaluation.stage);
    assert_eq!(repeated.reserve_ratio_bps, evaluation.reserve_ratio_bps);
    assert_eq!(repeated.total_pending, evaluation.total_pending);
    assert_eq!(repeated.pending_reserve, evaluation.pending_reserve);

    // Every adaptive Bevernomics profile must conserve exactly 100%.
    let allocation = allocation_for_stage(evaluation.stage);

    assert_eq!(
        allocation.total_bps(),
        10_000,
        "adaptive allocation failed to conserve 10,000 basis points"
    );

    for amount in [
        allocation.reserve_bps,
        allocation.buyback_burn_bps,
        allocation.liquidity_bps,
        allocation.company_bps,
        allocation.founder_bps,
    ] {
        assert!(
            amount <= 10_000,
            "individual adaptive allocation exceeded 100%"
        );
    }

    match evaluation.stage {
        WaterfallStage::Normal => {
            assert_eq!(allocation.reserve_bps, 3_000);
            assert_eq!(allocation.buyback_burn_bps, 2_000);
            assert_eq!(allocation.liquidity_bps, 2_000);
            assert_eq!(allocation.company_bps, 2_000);
            assert_eq!(allocation.founder_bps, 1_000);
        }

        WaterfallStage::Survival | WaterfallStage::Emergency => {
            assert_eq!(allocation.buyback_burn_bps, 0);
            assert_eq!(allocation.company_bps, 0);
            assert_eq!(allocation.founder_bps, 0);

            assert_eq!(
                u32::from(allocation.reserve_bps) + u32::from(allocation.liquidity_bps),
                10_000
            );
        }

        WaterfallStage::Caution | WaterfallStage::Defensive => {}
    }
});
