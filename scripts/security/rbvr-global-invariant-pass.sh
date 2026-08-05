#!/usr/bin/env bash
set -euo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "GLOBAL INVARIANT PASS FAILED"
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

echo "===== ADD GLOBAL INVARIANT TEST ====="

PATCH_OUTPUT="$(python3 rbvr-add-release-permutation-test.py)"
printf '%s\n' "$PATCH_OUTPUT"

BACKUP="$(
    printf '%s\n' "$PATCH_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "Could not identify the backup directory."
    exit 1
fi

echo
echo "===== FORMAT ====="
cargo fmt --all -- --check || cargo fmt --all

echo
echo "===== VERIFY TEST INSERTION ====="
grep -n -A 25 \
  'all_release_orders_reach_the_same_final_treasury_state' \
  programs/treasury-router/src/engines/release.rs

echo
echo "===== RUST TESTS ====="
cargo test --workspace

echo
echo "===== TARGETED PERMUTATION TEST ====="
cargo test \
  --workspace \
  all_release_orders_reach_the_same_final_treasury_state \
  -- \
  --exact \
  --nocapture

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
echo "===== FINAL TEST COUNTS ====="
cargo test --workspace 2>&1 |
grep -E 'running [0-9]+ tests|test result:'

echo
echo "===== DIFF ====="
git diff --stat
git status --short

echo
echo "============================================================"
echo "RBVR GLOBAL RELEASE INVARIANTS VERIFIED"
echo "============================================================"
echo "Expected Rust result: 122 passed"
echo "Expected integration result: 17 passing"
echo "Release orderings verified: 120"
echo "============================================================"

trap - EXIT
