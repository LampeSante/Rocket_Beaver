#![no_main]

use treasury_router::state::{
    FounderPriceState,
    FounderUsdCapState,
    FOUNDER_ANNUAL_CAP_USD_E6,
    FOUNDER_ANNUAL_PERIOD_SECONDS,
    FOUNDER_PRICE_VERSION,
    FOUNDER_USD_CAP_VERSION,
};


use anchor_lang::prelude::Pubkey;
use libfuzzer_sys::fuzz_target;
use treasury_router::{
    engines::beaver_score,
    instructions::process_fees::process_fee_cycle_usd_cap,
    state::{CompanyState, ProtocolState, TreasuryState},
};

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

    fn bool(&mut self) -> bool {
        self.u8() & 1 == 1
    }

    fn u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];

        for byte in &mut bytes {
            *byte = self.u8();
        }

        u64::from_le_bytes(bytes)
    }

    fn i64(&mut self) -> i64 {
        self.u64() as i64
    }
}

fn checked_sum(values: &[u64]) -> Option<u64> {
    values
        .iter()
        .try_fold(0u64, |total, value| total.checked_add(*value))
}

fuzz_target!(|data: &[u8]| {
    let mut cursor = Cursor::new(data);

    let protocol_key = Pubkey::new_from_array([7u8; 32]);

    let pending_reserve = cursor.u64();
    let pending_buyback = cursor.u64();
    let pending_liquidity = cursor.u64();
    let pending_company = cursor.u64();
    let pending_founder = cursor.u64();

    let released_reserve = cursor.u64();
    let released_buyback = cursor.u64();
    let released_liquidity = cursor.u64();
    let released_company = cursor.u64();
    let released_founder = cursor.u64();

    let lifetime_reserve = match pending_reserve.checked_add(released_reserve) {
        Some(value) => value,
        None => return,
    };

    let lifetime_buyback = match pending_buyback.checked_add(released_buyback) {
        Some(value) => value,
        None => return,
    };

    let lifetime_liquidity = match pending_liquidity.checked_add(released_liquidity) {
        Some(value) => value,
        None => return,
    };

    let lifetime_company = match pending_company.checked_add(released_company) {
        Some(value) => value,
        None => return,
    };

    let lifetime_founder = match pending_founder.checked_add(released_founder) {
        Some(value) => value,
        None => return,
    };

    let total_allocated = match checked_sum(&[
        lifetime_reserve,
        lifetime_buyback,
        lifetime_liquidity,
        lifetime_company,
        lifetime_founder,
    ]) {
        Some(value) => value,
        None => return,
    };

    let amount = cursor.u64();

    if amount == 0 {
        return;
    }

    let company_cap = cursor.u64();
    let company_spent = cursor.u64().min(company_cap);
    let company_lifetime = cursor.u64();

    let founder_cap = cursor.u64();
    let _founder_earned = cursor.u64().min(founder_cap);
    let _founder_lifetime = cursor.u64();

    let now = cursor.i64();
    let company_started = cursor.i64();
    let _founder_started = cursor.i64();

    // Keep duration non-negative while still exercising zero, tiny, and very
    // large period boundaries.
    let company_duration = (cursor.u64() >> 1) as i64;
    let _founder_duration = (cursor.u64() >> 1) as i64;

    let mut protocol = ProtocolState {
        version: 1,
        authority: Pubkey::new_unique(),
        protocol_config: Pubkey::new_unique(),
        treasury_state: Pubkey::new_unique(),
        reserve_state: Pubkey::default(),
        liquidity_state: Pubkey::default(),
        founder_state: Pubkey::new_unique(),
        company_state: Pubkey::new_unique(),
        buyback_state: Pubkey::default(),
        beaver_score: cursor.u64() as u16,
        dam_level: cursor.u8() % 5,
        paused: false,
        bump: 255,
        initialized_at: 0,
        reserved: [0; 64],
    };

    let mut treasury = TreasuryState {
        version: 1,
        protocol: protocol_key,
        settlement_mint: Pubkey::new_unique(),
        settlement_vault: Pubkey::new_unique(),

        total_fees_received: total_allocated,
        total_fees_allocated: total_allocated,

        pending_reserve,
        pending_buyback_burn: pending_buyback,
        pending_liquidity,
        pending_company,
        pending_founder,

        lifetime_reserve,
        lifetime_buyback_burn: lifetime_buyback,
        lifetime_liquidity,
        lifetime_company,
        lifetime_founder,

        released_reserve,
        released_buyback_burn: released_buyback,
        released_liquidity,
        released_company,
        released_founder,

        last_processed_at: cursor.i64(),
        processing_epoch: cursor.u64(),
        waterfall_stage: cursor.u8() % 5,
        buybacks_paused: cursor.bool(),
        bump: 254,
        reserved: [0; 24],
    };

    let mut company = CompanyState {
        version: 1,
        protocol: protocol_key,
        recipient: Pubkey::new_unique(),
        period_cap: company_cap,
        spent_current_period: company_spent,
        lifetime_spent: company_lifetime,
        period_started_at: company_started,
        period_duration: company_duration,
        enabled: cursor.bool(),
        bump: 253,
        reserved: [0; 64],
    };



    assert!(treasury.execution_accounting_is_valid());

    let old_total_received = treasury.total_fees_received;
    let old_total_allocated = treasury.total_fees_allocated;
    let old_epoch = treasury.processing_epoch;

    let old_lifetime_reserve = treasury.lifetime_reserve;
    let old_lifetime_buyback = treasury.lifetime_buyback_burn;
    let old_lifetime_liquidity = treasury.lifetime_liquidity;
    let old_lifetime_company = treasury.lifetime_company;
    let old_lifetime_founder = treasury.lifetime_founder;


    let fuzz_now = (now).max(1);

    let mut founder_usd_cap = FounderUsdCapState {
        version: FOUNDER_USD_CAP_VERSION,
        protocol: Pubkey::new_unique(),
        settlement_mint: Pubkey::new_unique(),
        price_feed_id: [7u8; 32],
        annual_cap_usd_e6: FOUNDER_ANNUAL_CAP_USD_E6,
        earned_current_period_usd_e6: 0,
        lifetime_earned_usd_e6: 0,
        period_started_at: fuzz_now,
        period_duration: FOUNDER_ANNUAL_PERIOD_SECONDS,
        max_price_age_seconds: 300,
        max_confidence_bps: 100,
        enabled: true,
        bump: 255,
        reserved: [0u8; 64],
    };

    let founder_price = FounderPriceState {
        version: FOUNDER_PRICE_VERSION,
        protocol: Pubkey::new_unique(),
        settlement_mint: Pubkey::new_unique(),
        price_feed_id: [7u8; 32],
        oracle_adapter_authority: Pubkey::new_unique(),
        price: 100_000_000,
        exponent: -8,
        confidence: 100_000,
        publish_time: fuzz_now,
        received_at: fuzz_now,
        sequence: 1,
        max_price_age_seconds: 300,
        max_confidence_bps: 100,
        enabled: true,
        bump: 255,
        reserved: [0u8; 64],
    };

    let result = process_fee_cycle_usd_cap(
        &mut protocol,
        &mut treasury,
        &mut founder_usd_cap,
        &founder_price,
        6,
        &mut company,
        amount,
        fuzz_now,
    );

    let outcome = match result {
        Ok(value) => value,

        // Checked arithmetic and invalid boundary combinations are expected
        // to fail closed. Solana rolls instruction mutations back atomically.
        Err(_) => return,
    };

    assert_eq!(
        treasury.total_fees_received,
        old_total_received.checked_add(amount).unwrap()
    );

    assert_eq!(
        treasury.total_fees_allocated,
        old_total_allocated.checked_add(amount).unwrap()
    );

    assert_eq!(treasury.processing_epoch, old_epoch.checked_add(1).unwrap());

    assert_eq!(treasury.last_processed_at, fuzz_now);
    assert!(treasury.execution_accounting_is_valid());

    let reserve_delta = treasury
        .lifetime_reserve
        .checked_sub(old_lifetime_reserve)
        .unwrap();

    let buyback_delta = treasury
        .lifetime_buyback_burn
        .checked_sub(old_lifetime_buyback)
        .unwrap();

    let liquidity_delta = treasury
        .lifetime_liquidity
        .checked_sub(old_lifetime_liquidity)
        .unwrap();

    let company_delta = treasury
        .lifetime_company
        .checked_sub(old_lifetime_company)
        .unwrap();

    let founder_delta = treasury
        .lifetime_founder
        .checked_sub(old_lifetime_founder)
        .unwrap();

    let conserved = checked_sum(&[
        reserve_delta,
        buyback_delta,
        liquidity_delta,
        company_delta,
        founder_delta,
    ])
    .unwrap();

    assert_eq!(conserved, amount);

    assert_eq!(reserve_delta, outcome.reserve_deposit.total_amount);
    assert_eq!(buyback_delta, outcome.buyback_deposit.amount);
    assert_eq!(liquidity_delta, outcome.liquidity_deposit.amount);
    assert_eq!(company_delta, outcome.company_amount);
    assert_eq!(founder_delta, outcome.founder_amount);

    assert_eq!(outcome.adaptive_allocation.total_bps(), 10_000);

    assert_eq!(
        outcome
            .company_amount
            .checked_add(
                outcome
                    .requested_company_amount
                    .checked_sub(outcome.company_amount)
                    .unwrap()
            )
            .unwrap(),
        outcome.requested_company_amount
    );

    assert_eq!(
        outcome
            .founder_amount
            .checked_add(
                outcome
                    .requested_founder_amount
                    .checked_sub(outcome.founder_amount)
                    .unwrap()
            )
            .unwrap(),
        outcome.requested_founder_amount
    );

    assert!(company.spent_current_period <= company.period_cap);
    assert!(
        founder_usd_cap.earned_current_period_usd_e6
            <= founder_usd_cap.annual_cap_usd_e6
    );

    assert!(treasury.waterfall_stage <= 4);
    assert!(protocol.dam_level <= 4);
    assert!(protocol.beaver_score <= beaver_score::MAX_BEAVER_SCORE);

    assert_eq!(
        protocol.beaver_score,
        outcome.beaver_score_evaluation.total_score
    );

    assert_eq!(protocol.dam_level, outcome.dam_evaluation.level.as_u8());

    assert_eq!(
        treasury.waterfall_stage,
        outcome.waterfall_evaluation.stage.as_u8()
    );
});
