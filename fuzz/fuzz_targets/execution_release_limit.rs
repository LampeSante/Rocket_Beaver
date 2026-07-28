#![no_main]

use libfuzzer_sys::fuzz_target;
use treasury_router::engines::execution_guard::calculate_maximum_release;

fuzz_target!(|data: &[u8]| {
    if data.len() < 10 {
        return;
    }

    let pending_balance = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]);

    let release_bps = u16::from_le_bytes([data[8], data[9]]);

    let result = calculate_maximum_release(pending_balance, release_bps);

    if release_bps > 10_000 {
        assert!(
            result.is_err(),
            "release rate above 100% was accepted: \
             pending={pending_balance}, release_bps={release_bps}"
        );
        return;
    }

    let maximum_release = result.unwrap_or_else(|error| {
        panic!(
            "valid release calculation failed: \
             pending={pending_balance}, release_bps={release_bps}, error={error:?}"
        )
    });

    assert!(
        maximum_release <= pending_balance,
        "maximum release exceeded pending balance: \
         pending={pending_balance}, release_bps={release_bps}, \
         maximum={maximum_release}"
    );

    let expected = ((u128::from(pending_balance) * u128::from(release_bps)) / 10_000) as u64;

    assert_eq!(
        maximum_release, expected,
        "production calculation disagreed with reference model: \
         pending={pending_balance}, release_bps={release_bps}"
    );

    if release_bps == 0 {
        assert_eq!(maximum_release, 0);
    }

    if release_bps == 10_000 {
        assert_eq!(maximum_release, pending_balance);
    }

    if pending_balance == 0 {
        assert_eq!(maximum_release, 0);
    }
});
