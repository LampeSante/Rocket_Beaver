#![no_main]

use anchor_lang::prelude::Pubkey;
use libfuzzer_sys::fuzz_target;
use treasury_router::state::TreasuryState;

const INPUT_LENGTH: usize = 120;

fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .expect("validated eight-byte range"),
    )
}

fn reference_bucket_is_valid(pending: u64, released: u64, lifetime: u64) -> bool {
    pending.checked_add(released) == Some(lifetime)
}

fuzz_target!(|data: &[u8]| {
    if data.len() < INPUT_LENGTH {
        return;
    }

    /*
     * Five independent accounting triples:
     *
     * reserve:   pending, released, lifetime
     * buyback:   pending, released, lifetime
     * liquidity: pending, released, lifetime
     * company:   pending, released, lifetime
     * founder:   pending, released, lifetime
     */

    let pending_reserve = read_u64(data, 0);
    let released_reserve = read_u64(data, 8);
    let lifetime_reserve = read_u64(data, 16);

    let pending_buyback_burn = read_u64(data, 24);
    let released_buyback_burn = read_u64(data, 32);
    let lifetime_buyback_burn = read_u64(data, 40);

    let pending_liquidity = read_u64(data, 48);
    let released_liquidity = read_u64(data, 56);
    let lifetime_liquidity = read_u64(data, 64);

    let pending_company = read_u64(data, 72);
    let released_company = read_u64(data, 80);
    let lifetime_company = read_u64(data, 88);

    let pending_founder = read_u64(data, 96);
    let released_founder = read_u64(data, 104);
    let lifetime_founder = read_u64(data, 112);

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

        lifetime_reserve,
        lifetime_buyback_burn,
        lifetime_liquidity,
        lifetime_company,
        lifetime_founder,

        released_reserve,
        released_buyback_burn,
        released_liquidity,
        released_company,
        released_founder,

        last_processed_at: 0,
        processing_epoch: 0,
        waterfall_stage: 0,
        buybacks_paused: false,
        bump: 0,
        reserved: [0; 24],
    };

    let reserve_valid =
        reference_bucket_is_valid(pending_reserve, released_reserve, lifetime_reserve);

    let buyback_valid = reference_bucket_is_valid(
        pending_buyback_burn,
        released_buyback_burn,
        lifetime_buyback_burn,
    );

    let liquidity_valid =
        reference_bucket_is_valid(pending_liquidity, released_liquidity, lifetime_liquidity);

    let company_valid =
        reference_bucket_is_valid(pending_company, released_company, lifetime_company);

    let founder_valid =
        reference_bucket_is_valid(pending_founder, released_founder, lifetime_founder);

    assert_eq!(
        treasury.reserve_accounting_is_valid(),
        reserve_valid,
        "Reserve accounting disagreed with checked reference model: \
         pending={pending_reserve}, released={released_reserve}, \
         lifetime={lifetime_reserve}"
    );

    assert_eq!(
        treasury.buyback_accounting_is_valid(),
        buyback_valid,
        "Buyback accounting disagreed with checked reference model: \
         pending={pending_buyback_burn}, \
         released={released_buyback_burn}, \
         lifetime={lifetime_buyback_burn}"
    );

    assert_eq!(
        treasury.liquidity_accounting_is_valid(),
        liquidity_valid,
        "Liquidity accounting disagreed with checked reference model: \
         pending={pending_liquidity}, released={released_liquidity}, \
         lifetime={lifetime_liquidity}"
    );

    assert_eq!(
        treasury.company_accounting_is_valid(),
        company_valid,
        "Company accounting disagreed with checked reference model: \
         pending={pending_company}, released={released_company}, \
         lifetime={lifetime_company}"
    );

    assert_eq!(
        treasury.founder_accounting_is_valid(),
        founder_valid,
        "Founder accounting disagreed with checked reference model: \
         pending={pending_founder}, released={released_founder}, \
         lifetime={lifetime_founder}"
    );

    let execution_valid =
        reserve_valid && buyback_valid && liquidity_valid && company_valid && founder_valid;

    assert_eq!(
        treasury.execution_accounting_is_valid(),
        execution_valid,
        "Combined execution-accounting result disagreed with \
         independent bucket validation"
    );

    // Combined validation must equal the conjunction of production validators.
    assert_eq!(
        treasury.execution_accounting_is_valid(),
        treasury.reserve_accounting_is_valid()
            && treasury.buyback_accounting_is_valid()
            && treasury.liquidity_accounting_is_valid()
            && treasury.company_accounting_is_valid()
            && treasury.founder_accounting_is_valid(),
        "Combined validator did not equal all five bucket validators"
    );

    /*
     * Construct a guaranteed-valid state from the same pending and released
     * values whenever all five additions can be represented by u64.
     */
    let valid_lifetimes = pending_reserve
        .checked_add(released_reserve)
        .zip(pending_buyback_burn.checked_add(released_buyback_burn))
        .zip(pending_liquidity.checked_add(released_liquidity))
        .zip(pending_company.checked_add(released_company))
        .zip(pending_founder.checked_add(released_founder));

    if let Some(((((reserve, buyback), liquidity), company), founder)) = valid_lifetimes {
        let mut valid = treasury;

        valid.lifetime_reserve = reserve;
        valid.lifetime_buyback_burn = buyback;
        valid.lifetime_liquidity = liquidity;
        valid.lifetime_company = company;
        valid.lifetime_founder = founder;

        assert!(valid.reserve_accounting_is_valid());
        assert!(valid.buyback_accounting_is_valid());
        assert!(valid.liquidity_accounting_is_valid());
        assert!(valid.company_accounting_is_valid());
        assert!(valid.founder_accounting_is_valid());
        assert!(valid.execution_accounting_is_valid());

        // Corrupt exactly one bucket without changing any other bucket.
        valid.lifetime_reserve = valid.lifetime_reserve.wrapping_add(1);

        assert!(
            !valid.reserve_accounting_is_valid(),
            "One-unit reserve corruption was not detected"
        );

        assert!(
            !valid.execution_accounting_is_valid(),
            "Combined accounting accepted a corrupted reserve bucket"
        );
    }
});
