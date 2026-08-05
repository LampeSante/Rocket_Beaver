#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

EXPECTED_COMMIT_PREFIX="aa755ae"
TAG="rbvr-reserve-policy-v1"

FILES=(
  "programs/treasury-router/src/constants.rs"
  "programs/treasury-router/src/errors/mod.rs"
  "programs/treasury-router/src/lib.rs"
  "programs/treasury-router/src/state/mod.rs"
  "programs/treasury-router/src/state/reserve_policy.rs"
  "programs/treasury-router/src/instructions/mod.rs"
  "programs/treasury-router/src/instructions/initialize_reserve_policy.rs"
)

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "IMMUTABLE RESERVE POLICY PASS FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" && -d "$BACKUP" ]]; then
            for file in \
              programs/treasury-router/src/constants.rs \
              programs/treasury-router/src/errors/mod.rs \
              programs/treasury-router/src/lib.rs \
              programs/treasury-router/src/state/mod.rs \
              programs/treasury-router/src/instructions/mod.rs
            do
                if [[ -f "$BACKUP/$file" ]]; then
                    cp "$BACKUP/$file" "$file"
                fi
            done

            rm -f \
              programs/treasury-router/src/state/reserve_policy.rs \
              programs/treasury-router/src/instructions/initialize_reserve_policy.rs

            cargo fmt --all >/dev/null 2>&1 || true
            echo "Original files restored from: $BACKUP"
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "============================================================"
echo "RBVR IMMUTABLE RESERVE POLICY — PHASE 1"
echo "============================================================"

if [[ "$(git branch --show-current)" != "red-team" ]]; then
    echo "ERROR: Expected branch red-team."
    exit 1
fi

CURRENT_COMMIT="$(git rev-parse HEAD)"

if [[ "$CURRENT_COMMIT" != "$EXPECTED_COMMIT_PREFIX"* ]]; then
    echo "ERROR: Expected commit beginning with $EXPECTED_COMMIT_PREFIX."
    echo "Current commit: $CURRENT_COMMIT"
    exit 1
fi

echo
echo "===== ADD IMMUTABLE RESERVE POLICY ====="

PATCH_LOG="$(mktemp)"

set +e
python3 rbvr-add-immutable-reserve-policy.py 2>&1 | tee "$PATCH_LOG"
PATCH_STATUS="${PIPESTATUS[0]}"
set -e

BACKUP="$(
    sed -n 's/^Backup created: //p' "$PATCH_LOG" |
    head -n 1
)"

rm -f "$PATCH_LOG"

if [[ "$PATCH_STATUS" -ne 0 ]]; then
    exit "$PATCH_STATUS"
fi

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "ERROR: Could not capture backup directory."
    exit 1
fi

echo
echo "===== VERIFY NO UPDATE INSTRUCTION EXISTS ====="

if grep -RIn \
    'update_reserve_policy\|UpdateReservePolicy' \
    programs/treasury-router/src
then
    echo "ERROR: A Reserve Policy update path exists."
    exit 1
fi

echo "No Reserve Policy update instruction found."

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== ANCHOR BUILD ====="
anchor build

echo
echo "===== TYPESCRIPT CHECK ====="
npx tsc --noEmit

echo
echo "===== TARGETED RESERVE POLICY TESTS ====="

cargo test \
    --workspace \
    state::reserve_policy::tests \
    -- \
    --nocapture

echo
echo "===== COMPLETE RUST REGRESSION ====="

RUST_LOG="$(mktemp)"
cargo test --workspace 2>&1 | tee "$RUST_LOG"

if ! grep -q 'test result: ok. 145 passed; 0 failed' "$RUST_LOG"; then
    echo "ERROR: Expected 145 passing Rust tests."
    rm -f "$RUST_LOG"
    exit 1
fi

rm -f "$RUST_LOG"

echo
echo "===== STRICT CLIPPY ====="

cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- \
    -D warnings

echo
echo "===== INTEGRATION REGRESSION ====="

INTEGRATION_LOG="$(mktemp)"
anchor test --validator legacy 2>&1 | tee "$INTEGRATION_LOG"

if ! grep -q '37 passing' "$INTEGRATION_LOG"; then
    echo "ERROR: Expected the existing 37 integration tests to pass."
    rm -f "$INTEGRATION_LOG"
    exit 1
fi

rm -f "$INTEGRATION_LOG"

echo
echo "===== REVIEW CHANGES ====="

git diff --check
git diff --stat -- "${FILES[@]}"

echo
echo "===== STAGE VERIFIED POLICY FILES ====="

git add "${FILES[@]}"
git diff --cached --stat

if git diff --cached --quiet; then
    echo "ERROR: No Reserve Policy changes were staged."
    exit 1
fi

echo
echo "===== COMMIT IMMUTABLE RESERVE POLICY ====="

git commit -m \
    "protocol: add immutable reserve deployment policy"

echo
echo "===== TAG RESERVE POLICY CHECKPOINT ====="

if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Tag already exists: $TAG"
else
    git tag -a "$TAG" \
      -m "Immutable Reserve Policy state and initialization verified"
fi

echo
echo "===== FINAL CHECKPOINT ====="

git log --oneline --decorate -5

echo
git status --short

echo
echo "============================================================"
echo "RBVR IMMUTABLE RESERVE POLICY PASSED"
echo "============================================================"
echo "Policy account:          ReservePolicy PDA"
echo "Policy update path:      none"
echo "Minimum floor:           supplied once at initialization"
echo "Liquidity floor rate:   supplied once at initialization"
echo "Deployment rate:        supplied once at initialization"
echo "Cooldown:                supplied once at initialization"
echo "New Rust tests:          11"
echo "Complete Rust tests:     145 expected"
echo "Integration regression: 37 expected"
echo "Live token movement:     unchanged"
echo "Tag:                     $TAG"
echo
echo "Next:"
echo "Implement permissionless surplus-only Spillway execution."
echo "============================================================"

trap - EXIT
