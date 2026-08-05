from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path.cwd()
TARGET = ROOT / "programs/treasury-router/src/engines/release.rs"

TEST_MARKER = "all_release_orders_reach_the_same_final_treasury_state"

if not TARGET.exists():
    raise RuntimeError(f"Missing file: {TARGET}")

original = TARGET.read_text()

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup_root = ROOT / f".global-invariant-v2-backup-{timestamp}"
backup_file = backup_root / TARGET.relative_to(ROOT)

backup_file.parent.mkdir(parents=True, exist_ok=False)
shutil.copy2(TARGET, backup_file)

def restore_and_fail(message: str) -> None:
    shutil.copy2(backup_file, TARGET)
    raise RuntimeError(f"{message}\nOriginal release.rs restored.")

try:
    text = original

    # Correct only the release-engine test fixture. Its five lifetime buckets
    # total 1,800, while the old global total incorrectly stated 1,500.
    tests_start = text.find("#[cfg(test)]\nmod tests {")

    if tests_start == -1:
        restore_and_fail("Could not locate release.rs test module.")

    fixture_start = text.find(
        "    fn valid_treasury() -> TreasuryState {",
        tests_start,
    )

    if fixture_start == -1:
        restore_and_fail("Could not locate release.rs valid_treasury fixture.")

    fixture_end = text.find(
        "\n    }\n\n    fn assert_release",
        fixture_start,
    )

    if fixture_end == -1:
        restore_and_fail("Could not isolate release.rs valid_treasury fixture.")

    fixture = text[fixture_start:fixture_end]

    old_totals = """            total_fees_received: 1_500,
            total_fees_allocated: 1_500,"""

    new_totals = """            total_fees_received: 1_800,
            total_fees_allocated: 1_800,"""

    if old_totals not in fixture:
        restore_and_fail(
            "Expected inconsistent 1,500 fixture totals were not found."
        )

    fixture = fixture.replace(old_totals, new_totals, 1)
    text = text[:fixture_start] + fixture + text[fixture_end:]

    if TEST_MARKER in text:
        restore_and_fail(
            "Permutation test already exists unexpectedly; refusing duplicate insertion."
        )

    module_end = text.rfind("\n}")

    if module_end == -1 or module_end <= tests_start:
        restore_and_fail("Could not locate end of release.rs test module.")

    patch = r'''
    #[derive(Debug, PartialEq, Eq)]
    struct AccountingSnapshot {
        total_fees_received: u64,
        total_fees_allocated: u64,

        pending_reserve: u64,
        pending_buyback_burn: u64,
        pending_liquidity: u64,
        pending_company: u64,
        pending_founder: u64,

        lifetime_reserve: u64,
        lifetime_buyback_burn: u64,
        lifetime_liquidity: u64,
        lifetime_company: u64,
        lifetime_founder: u64,

        released_reserve: u64,
        released_buyback_burn: u64,
        released_liquidity: u64,
        released_company: u64,
        released_founder: u64,
    }

    fn accounting_snapshot(treasury: &TreasuryState) -> AccountingSnapshot {
        AccountingSnapshot {
            total_fees_received: treasury.total_fees_received,
            total_fees_allocated: treasury.total_fees_allocated,

            pending_reserve: treasury.pending_reserve,
            pending_buyback_burn: treasury.pending_buyback_burn,
            pending_liquidity: treasury.pending_liquidity,
            pending_company: treasury.pending_company,
            pending_founder: treasury.pending_founder,

            lifetime_reserve: treasury.lifetime_reserve,
            lifetime_buyback_burn: treasury.lifetime_buyback_burn,
            lifetime_liquidity: treasury.lifetime_liquidity,
            lifetime_company: treasury.lifetime_company,
            lifetime_founder: treasury.lifetime_founder,

            released_reserve: treasury.released_reserve,
            released_buyback_burn: treasury.released_buyback_burn,
            released_liquidity: treasury.released_liquidity,
            released_company: treasury.released_company,
            released_founder: treasury.released_founder,
        }
    }

    fn checked_bucket_lifetime_total(treasury: &TreasuryState) -> u64 {
        treasury
            .lifetime_reserve
            .checked_add(treasury.lifetime_buyback_burn)
            .and_then(|value| value.checked_add(treasury.lifetime_liquidity))
            .and_then(|value| value.checked_add(treasury.lifetime_company))
            .and_then(|value| value.checked_add(treasury.lifetime_founder))
            .expect("valid fixture lifetime totals must not overflow")
    }

    fn checked_pending_total(treasury: &TreasuryState) -> u64 {
        treasury
            .pending_reserve
            .checked_add(treasury.pending_buyback_burn)
            .and_then(|value| value.checked_add(treasury.pending_liquidity))
            .and_then(|value| value.checked_add(treasury.pending_company))
            .and_then(|value| value.checked_add(treasury.pending_founder))
            .expect("valid fixture pending totals must not overflow")
    }

    fn checked_released_total(treasury: &TreasuryState) -> u64 {
        treasury
            .released_reserve
            .checked_add(treasury.released_buyback_burn)
            .and_then(|value| value.checked_add(treasury.released_liquidity))
            .and_then(|value| value.checked_add(treasury.released_company))
            .and_then(|value| value.checked_add(treasury.released_founder))
            .expect("valid fixture released totals must not overflow")
    }

    fn assert_global_accounting_invariants(treasury: &TreasuryState) {
        assert!(
            treasury.execution_accounting_is_valid(),
            "every bucket must satisfy lifetime = pending + released",
        );

        let lifetime_total = checked_bucket_lifetime_total(treasury);
        let pending_total = checked_pending_total(treasury);
        let released_total = checked_released_total(treasury);

        assert_eq!(
            lifetime_total,
            treasury.total_fees_allocated,
            "sum of bucket lifetime allocations must equal total allocated fees",
        );

        assert!(
            treasury.total_fees_allocated <= treasury.total_fees_received,
            "allocated fees may never exceed received fees",
        );

        assert_eq!(
            pending_total
                .checked_add(released_total)
                .expect("valid global totals must not overflow"),
            lifetime_total,
            "global pending plus released must equal global lifetime allocation",
        );
    }

    fn collect_release_permutations(
        buckets: &mut [ReleaseBucket; 5],
        index: usize,
        permutations: &mut Vec<[ReleaseBucket; 5]>,
    ) {
        if index == buckets.len() {
            permutations.push(*buckets);
            return;
        }

        for swap_index in index..buckets.len() {
            buckets.swap(index, swap_index);
            collect_release_permutations(buckets, index + 1, permutations);
            buckets.swap(index, swap_index);
        }
    }

    #[test]
    fn all_release_orders_reach_the_same_final_treasury_state() {
        let mut buckets = [
            ReleaseBucket::Reserve,
            ReleaseBucket::BuybackBurn,
            ReleaseBucket::Liquidity,
            ReleaseBucket::Company,
            ReleaseBucket::Founder,
        ];

        let mut permutations = Vec::new();
        collect_release_permutations(&mut buckets, 0, &mut permutations);

        assert_eq!(
            permutations.len(),
            120,
            "five buckets must produce exactly 5! release orders",
        );

        let mut expected_final_snapshot: Option<AccountingSnapshot> = None;

        for (permutation_index, permutation) in permutations.iter().enumerate() {
            let mut treasury = valid_treasury();

            let initial_lifetime_total = checked_bucket_lifetime_total(&treasury);
            let initial_received = treasury.total_fees_received;
            let initial_allocated = treasury.total_fees_allocated;

            assert_global_accounting_invariants(&treasury);

            for bucket in permutation {
                let before = accounting_snapshot(&treasury);
                let before_lifetime_total =
                    checked_bucket_lifetime_total(&treasury);

                let amount = balances_for_bucket(&treasury, *bucket).0;

                assert!(
                    amount > 0,
                    "every fixture bucket must begin with a pending balance",
                );

                let transition =
                    process_release(&mut treasury, *bucket, amount).unwrap();

                assert_eq!(transition.bucket, *bucket);
                assert_eq!(transition.amount, amount);
                assert_eq!(transition.remaining_pending_balance, 0);

                assert_global_accounting_invariants(&treasury);

                assert_eq!(
                    checked_bucket_lifetime_total(&treasury),
                    before_lifetime_total,
                    "release must not alter lifetime allocations",
                );

                assert_eq!(
                    treasury.total_fees_received,
                    initial_received,
                    "release must not alter received-fee accounting",
                );

                assert_eq!(
                    treasury.total_fees_allocated,
                    initial_allocated,
                    "release must not alter allocated-fee accounting",
                );

                match bucket {
                    ReleaseBucket::Reserve => {
                        assert_eq!(
                            treasury.pending_buyback_burn,
                            before.pending_buyback_burn
                        );
                        assert_eq!(
                            treasury.pending_liquidity,
                            before.pending_liquidity
                        );
                        assert_eq!(
                            treasury.pending_company,
                            before.pending_company
                        );
                        assert_eq!(
                            treasury.pending_founder,
                            before.pending_founder
                        );
                    }
                    ReleaseBucket::BuybackBurn => {
                        assert_eq!(
                            treasury.pending_reserve,
                            before.pending_reserve
                        );
                        assert_eq!(
                            treasury.pending_liquidity,
                            before.pending_liquidity
                        );
                        assert_eq!(
                            treasury.pending_company,
                            before.pending_company
                        );
                        assert_eq!(
                            treasury.pending_founder,
                            before.pending_founder
                        );
                    }
                    ReleaseBucket::Liquidity => {
                        assert_eq!(
                            treasury.pending_reserve,
                            before.pending_reserve
                        );
                        assert_eq!(
                            treasury.pending_buyback_burn,
                            before.pending_buyback_burn
                        );
                        assert_eq!(
                            treasury.pending_company,
                            before.pending_company
                        );
                        assert_eq!(
                            treasury.pending_founder,
                            before.pending_founder
                        );
                    }
                    ReleaseBucket::Company => {
                        assert_eq!(
                            treasury.pending_reserve,
                            before.pending_reserve
                        );
                        assert_eq!(
                            treasury.pending_buyback_burn,
                            before.pending_buyback_burn
                        );
                        assert_eq!(
                            treasury.pending_liquidity,
                            before.pending_liquidity
                        );
                        assert_eq!(
                            treasury.pending_founder,
                            before.pending_founder
                        );
                    }
                    ReleaseBucket::Founder => {
                        assert_eq!(
                            treasury.pending_reserve,
                            before.pending_reserve
                        );
                        assert_eq!(
                            treasury.pending_buyback_burn,
                            before.pending_buyback_burn
                        );
                        assert_eq!(
                            treasury.pending_liquidity,
                            before.pending_liquidity
                        );
                        assert_eq!(
                            treasury.pending_company,
                            before.pending_company
                        );
                    }
                }
            }

            assert_eq!(
                checked_pending_total(&treasury),
                0,
                "all pending balances must reach zero",
            );

            assert_eq!(
                checked_released_total(&treasury),
                initial_lifetime_total,
                "fully released treasury must release its lifetime allocation",
            );

            assert_eq!(
                treasury.released_reserve,
                treasury.lifetime_reserve
            );
            assert_eq!(
                treasury.released_buyback_burn,
                treasury.lifetime_buyback_burn
            );
            assert_eq!(
                treasury.released_liquidity,
                treasury.lifetime_liquidity
            );
            assert_eq!(
                treasury.released_company,
                treasury.lifetime_company
            );
            assert_eq!(
                treasury.released_founder,
                treasury.lifetime_founder
            );

            let final_snapshot = accounting_snapshot(&treasury);

            if let Some(expected) = &expected_final_snapshot {
                assert_eq!(
                    &final_snapshot,
                    expected,
                    "release permutation {permutation_index} reached a different final state",
                );
            } else {
                expected_final_snapshot = Some(final_snapshot);
            }
        }
    }
'''

    updated = (
        text[:module_end]
        + "\n"
        + patch.rstrip()
        + "\n"
        + text[module_end:]
    )

    TARGET.write_text(updated)

    verification = TARGET.read_text()

    required = [
        "total_fees_received: 1_800",
        "total_fees_allocated: 1_800",
        TEST_MARKER,
        "collect_release_permutations",
        "assert_global_accounting_invariants",
        "permutations.len(),",
        "120,",
    ]

    for fragment in required:
        if fragment not in verification:
            restore_and_fail(
                f"Patch verification failed. Missing fragment: {fragment}"
            )

    print(f"Backup created: {backup_root}")
    print("Corrected release test fixture totals: 1,500 -> 1,800")
    print("Inserted 120-order global invariant test.")

except Exception:
    if TARGET.read_text() != original:
        shutil.copy2(backup_file, TARGET)
    raise
