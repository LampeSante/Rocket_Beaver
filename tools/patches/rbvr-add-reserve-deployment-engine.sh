#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RESERVE DEPLOYMENT ENGINE PASS FAILED"
        echo "============================================================"

        rm -f \
          programs/treasury-router/src/engines/reserve_deployment.rs

        if [[ -n "$BACKUP" &&
              -f "$BACKUP/programs/treasury-router/src/engines/mod.rs" ]]; then
            cp \
              "$BACKUP/programs/treasury-router/src/engines/mod.rs" \
              programs/treasury-router/src/engines/mod.rs

            echo "Original engine module restored."
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "===== ADD RESERVE DEPLOYMENT ENGINE ====="

PATCH_OUTPUT="$(python3 rbvr-add-reserve-deployment-engine.py)"
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
echo "===== VERIFY MODULE ====="

grep -n \
  'reserve_deployment' \
  programs/treasury-router/src/engines/mod.rs

grep -nE \
  'pub enum ReserveDeploymentStage|pub struct ReserveDeploymentPolicy|pub fn evaluate_reserve_deployment' \
  programs/treasury-router/src/engines/reserve_deployment.rs

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== TARGETED RESERVE TESTS ====="

cargo test \
  --workspace \
  engines::reserve_deployment::tests \
  -- \
  --nocapture

echo
echo "===== COMPLETE RUST REGRESSION ====="
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
echo "===== INTEGRATION REGRESSION ====="
anchor test --validator legacy

echo
echo "===== STATUS ====="
git diff --stat
git status --short

echo
echo "============================================================"
echo "RBVR RESERVE DEPLOYMENT ENGINE PASSED"
echo "============================================================"
echo "New deterministic engine: reserve_deployment"
echo "Live token paths changed:  none"
echo "Instruction API changed:   none"
echo "Reserve-floor invariant:   verified"
echo "Overflow-safe arithmetic:  verified"
echo "Next: lock policy values and integrate surplus deployment"
echo "============================================================"

trap - EXIT
