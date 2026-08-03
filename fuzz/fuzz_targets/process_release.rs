#![no_main]

use anchor_lang::prelude::Pubkey;
use libfuzzer_sys::fuzz_target;
use treasury_router::{
    engines::{
        execution_guard::ReleaseBucket,
        release::{balances_for_bucket, process_release},
    },
    state::TreasuryState,
};

const BUCKET_COUNT: usize = 5;

struct Cursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn u8(&mut self) -> u8 {
        let value = self.data.get(self.offset).copied().unwrap_or(0);
        self.offset = self.offset.saturating_add(1);
        value
    }

    fn u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];

        for byte in &mut bytes {
            *byte = self.u8();
        }

        u64::from_le_bytes(bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TreasurySnapshot {
    total_fees_received: u64,
    total_fees_allocated: u64,

    pending: [u64; BUCKET_COUNT],
    lifetime: [u64; BUCKET_COUNT],
    released: [u64; BUCKET_COUNT],

    last_processed_at: i64,
    processing_epoch: u64,
    waterfall_stage: u8,
    buybacks_paused: bool,
    bump: u8,
}

fn bucket_from_byte(value: u8) -> ReleaseBucket {
    match value % BUCKET_COUNT as u8 {
        0 => ReleaseBucket::Reserve,
        1 => ReleaseBucket::BuybackBurn,
        2 => ReleaseBucket::Liquidity,
        3 => ReleaseBucket::Company,
        _ => ReleaseBucket::Founder,
    }
}

fn bucket_index(bucket: ReleaseBucket) -> usize {
    match bucket {
        ReleaseBucket::Reserve => 0,
        ReleaseBucket::BuybackBurn => 1,
        ReleaseBucket::Liquidity => 2,
        ReleaseBucket::Company => 3,
        ReleaseBucket::Founder => 4,
    }
}

fn checked_sum(values: &[u64]) -> Option<u64> {
    values
        .iter()
        .try_fold(0u64, |total, value| total.checked_add(*value))
}

fn split_lifetime(lifetime: u64, selector: u64) -> (u64, u64) {
    let pending = if lifetime == u64::MAX {
        selector
    } else {
        selector % (lifetime + 1)
    };

    let released = lifetime - pending;

    (pending, released)
}

fn make_treasury(
    pending: [u64; BUCKET_COUNT],
    released: [u64; BUCKET_COUNT],
    lifetime: [u64; BUCKET_COUNT],
    total_allocated: u64,
    last_processed_at: i64,
    processing_epoch: u64,
    waterfall_stage: u8,
    buybacks_paused: bool,
    bump: u8,
) -> TreasuryState {
    TreasuryState {
        version: 1,
        protocol: Pubkey::new_from_array([1u8; 32]),
        settlement_mint: Pubkey::new_from_array([2u8; 32]),
        settlement_vault: Pubkey::new_from_array([3u8; 32]),

        total_fees_received: total_allocated,
        total_fees_allocated: total_allocated,

        pending_reserve: pending[0],
        pending_buyback_burn: pending[1],
        pending_liquidity: pending[2],
        pending_company: pending[3],
        pending_founder: pending[4],

        lifetime_reserve: lifetime[0],
        lifetime_buyback_burn: lifetime[1],
        lifetime_liquidity: lifetime[2],
        lifetime_company: lifetime[3],
        lifetime_founder: lifetime[4],

        released_reserve: released[0],
        released_buyback_burn: released[1],
        released_liquidity: released[2],
        released_company: released[3],
        released_founder: released[4],

        last_processed_at,
        processing_epoch,
        waterfall_stage,
        buybacks_paused,
        bump,
        reserved: [0; 24],
    }
}

fn snapshot(treasury: &TreasuryState) -> TreasurySnapshot {
    TreasurySnapshot {
        total_fees_received: treasury.total_fees_received,
        total_fees_allocated: treasury.total_fees_allocated,

        pending: [
            treasury.pending_reserve,
            treasury.pending_buyback_burn,
            treasury.pending_liquidity,
            treasury.pending_company,
            treasury.pending_founder,
        ],

        lifetime: [
            treasury.lifetime_reserve,
            treasury.lifetime_buyback_burn,
            treasury.lifetime_liquidity,
            treasury.lifetime_company,
            treasury.lifetime_founder,
        ],

        released: [
            treasury.released_reserve,
            treasury.released_buyback_burn,
            treasury.released_liquidity,
            treasury.released_company,
            treasury.released_founder,
        ],

        last_processed_at: treasury.last_processed_at,
        processing_epoch: treasury.processing_epoch,
        waterfall_stage: treasury.waterfall_stage,
        buybacks_paused: treasury.buybacks_paused,
        bump: treasury.bump,
    }
}

fn corrupt_lifetime(treasury: &mut TreasuryState, bucket: ReleaseBucket) {
    match bucket {
        ReleaseBucket::Reserve => {
            treasury.lifetime_reserve = treasury.lifetime_reserve.wrapping_add(1);
        }
        ReleaseBucket::BuybackBurn => {
            treasury.lifetime_buyback_burn = treasury.lifetime_buyback_burn.wrapping_add(1);
        }
        ReleaseBucket::Liquidity => {
            treasury.lifetime_liquidity = treasury.lifetime_liquidity.wrapping_add(1);
        }
        ReleaseBucket::Company => {
            treasury.lifetime_company = treasury.lifetime_company.wrapping_add(1);
        }
        ReleaseBucket::Founder => {
            treasury.lifetime_founder = treasury.lifetime_founder.wrapping_add(1);
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let mut cursor = Cursor::new(data);

    let bucket = bucket_from_byte(cursor.u8());
    let selected_index = bucket_index(bucket);

    let mut lifetime = [0u64; BUCKET_COUNT];
    let mut pending = [0u64; BUCKET_COUNT];
    let mut released = [0u64; BUCKET_COUNT];

    for index in 0..BUCKET_COUNT {
        lifetime[index] = cursor.u64();

        let split_selector = cursor.u64();
        let split = split_lifetime(lifetime[index], split_selector);

        pending[index] = split.0;
        released[index] = split.1;
    }

    let total_allocated = match checked_sum(&lifetime) {
        Some(value) => value,
        None => return,
    };

    let amount = cursor.u64();
    let corruption_mode = cursor.u8() % 4;

    let corrupt_bucket = match corruption_mode {
        1 => Some(bucket),
        2 => {
            let unrelated = (selected_index + 1 + usize::from(cursor.u8()) % 4) % BUCKET_COUNT;

            Some(bucket_from_byte(unrelated as u8))
        }
        3 => Some(bucket_from_byte(cursor.u8())),
        _ => None,
    };

    let last_processed_at = cursor.u64() as i64;
    let processing_epoch = cursor.u64();
    let waterfall_stage = cursor.u8() % 5;
    let buybacks_paused = cursor.u8() & 1 == 1;
    let bump = cursor.u8();

    let mut treasury = make_treasury(
        pending,
        released,
        lifetime,
        total_allocated,
        last_processed_at,
        processing_epoch,
        waterfall_stage,
        buybacks_paused,
        bump,
    );

    let mut repeated = make_treasury(
        pending,
        released,
        lifetime,
        total_allocated,
        last_processed_at,
        processing_epoch,
        waterfall_stage,
        buybacks_paused,
        bump,
    );

    if let Some(corrupted_bucket) = corrupt_bucket {
        corrupt_lifetime(&mut treasury, corrupted_bucket);
        corrupt_lifetime(&mut repeated, corrupted_bucket);
    }

    let before = snapshot(&treasury);
    let repeated_before = snapshot(&repeated);

    assert_eq!(
        before, repeated_before,
        "identically constructed treasury states differed"
    );

    let selected_before = balances_for_bucket(&treasury, bucket);

    let result = process_release(&mut treasury, bucket, amount);
    let repeated_result = process_release(&mut repeated, bucket, amount);

    assert_eq!(
        result.is_ok(),
        repeated_result.is_ok(),
        "identical release inputs produced different success results"
    );

    assert_eq!(
        snapshot(&treasury),
        snapshot(&repeated),
        "identical release inputs produced different treasury states"
    );

    let should_succeed = corrupt_bucket.is_none() && amount > 0 && amount <= selected_before.0;

    assert_eq!(
        result.is_ok(),
        should_succeed,
        "release success disagreed with the independent reference model: \
         bucket={bucket:?}, amount={amount}, \
         pending={}, corrupted={}",
        selected_before.0,
        corrupt_bucket.is_some()
    );

    match result {
        Ok(transition) => {
            let after = snapshot(&treasury);
            let selected_after = balances_for_bucket(&treasury, bucket);

            assert_eq!(transition.bucket, bucket);
            assert_eq!(transition.amount, amount);

            assert_eq!(transition.previous_pending_balance, selected_before.0);

            assert_eq!(transition.previous_released_balance, selected_before.1);

            let expected_pending = selected_before.0.checked_sub(amount).unwrap();

            let expected_released = selected_before.1.checked_add(amount).unwrap();

            assert_eq!(transition.remaining_pending_balance, expected_pending);

            assert_eq!(transition.total_released_balance, expected_released);

            assert_eq!(selected_after, (expected_pending, expected_released));

            assert_eq!(
                selected_before.0.checked_add(selected_before.1),
                selected_after.0.checked_add(selected_after.1),
                "selected bucket did not conserve pending plus released"
            );

            assert_eq!(
                after.lifetime[selected_index], before.lifetime[selected_index],
                "release changed selected bucket lifetime accounting"
            );

            for index in 0..BUCKET_COUNT {
                if index == selected_index {
                    continue;
                }

                assert_eq!(
                    after.pending[index], before.pending[index],
                    "release modified an unrelated pending bucket"
                );

                assert_eq!(
                    after.released[index], before.released[index],
                    "release modified an unrelated released bucket"
                );

                assert_eq!(
                    after.lifetime[index], before.lifetime[index],
                    "release modified an unrelated lifetime bucket"
                );
            }

            assert_eq!(after.total_fees_received, before.total_fees_received);

            assert_eq!(after.total_fees_allocated, before.total_fees_allocated);

            assert_eq!(after.last_processed_at, before.last_processed_at);
            assert_eq!(after.processing_epoch, before.processing_epoch);
            assert_eq!(after.waterfall_stage, before.waterfall_stage);
            assert_eq!(after.buybacks_paused, before.buybacks_paused);
            assert_eq!(after.bump, before.bump);

            assert!(
                treasury.execution_accounting_is_valid(),
                "successful release produced invalid accounting"
            );
        }

        Err(_) => {
            assert_eq!(
                snapshot(&treasury),
                before,
                "rejected release partially mutated treasury state"
            );
        }
    }
});
