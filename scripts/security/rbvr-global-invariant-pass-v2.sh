#!/usr/bin/env bash
set -euo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "GLOBAL INVARIANT V2 FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" &&
              -f "$BACKUP/programs/treasury-router/src/engines/release.rs" ]]; then
            cp \
              "$BACKUP/programs/treasury-router/src/engines/release.rs" \
              programs/treasury-router/src/engines/release.rs

            echo "release.rs restored from backup."
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "===== PATCH FIXTURE AND ADD INVARIANT TEST ====="

PATCH_OUTPUT="$(python3 rbvr-global-invariant-pass-v2.py)"
printf '%s\n' "$PATCH_OUTPUT"

BACKUP="$(
    printf '%s\n' "$PATCH_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "Could not identify backup directory."
    exit 1
fi

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== VERIFY FIXTURE ====="
grep -n -A 12 \
  'fn valid_treasury() -> TreasuryState' \
  programs/treasury-router/src/engines/release.rs |
head -n 20

echo
echo "===== VERIFY PERMUTATION TEST ====="
grep -n -A 20 \
  'all_release_orders_reach_the_same_final_treasury_state' \
  programs/treasury-router/src/engines/release.rs

echo
echo "===== TARGETED TEST ====="
cargo test \
  --workspace \
  all_release_orders_reach_the_same_final_treasury_state \
  -- \
  --exact \
  --nocapture

echo
echo "===== ALL RUST TESTS ====="
cargo test --workspace

echo
echo "===== STRICT CLIPPY ====="
cargo clippy \
  --workspace \
  --all-targets \
  --all-features \
  -- \
  -D warnings

echo
echo "===== ANCHOR BUILD ====="
anchor build

echo
echo "===== INTEGRATION TESTS ====="
anchor test --validator legacy

echo
echo "===== FINAL STATUS ====="
git diff --stat
git status --short

echo
echo "============================================================"
echo "RBVR GLOBAL RELEASE INVARIANTS VERIFIED"
echo "============================================================"
echo "Rust tests expected: 122 passed"
echo "Integration tests expected: 17 passing"
echo "Release permutations verified: 120"
echo "Fixture global allocation: 1,800"
echo "============================================================"

trap - EXIT
