#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

GENERATOR="rbvr-reserve-vault-phase1.py"
EXPECTED_COMMIT_PREFIX="ae6e68b"
TAG="rbvr-reserve-vault-v1"

FILES=(
  "programs/treasury-router/src/constants.rs"
  "programs/treasury-router/src/instructions/initialize_execution_config.rs"
  "programs/treasury-router/src/instructions/reserve.rs"
  "tests/rbvr_protocol.ts"
)

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RESERVE VAULT PHASE 1 FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" && -d "$BACKUP" ]]; then
            for file in "${FILES[@]}"; do
                if [[ -f "$BACKUP/$file" ]]; then
                    cp "$BACKUP/$file" "$file"
                fi
            done

            cargo fmt --all >/dev/null 2>&1 || true
            echo "Original protocol files restored from:"
            echo "$BACKUP"
        else
            echo "No backup directory was captured."
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "============================================================"
echo "RBVR RESERVE VAULT — FINAL PHASE 1 PASS"
echo "============================================================"

if [[ "$(git branch --show-current)" != "red-team" ]]; then
    echo "ERROR: Expected branch red-team."
    echo "Current branch: $(git branch --show-current)"
    exit 1
fi

CURRENT_COMMIT="$(git rev-parse HEAD)"

if [[ "$CURRENT_COMMIT" != "$EXPECTED_COMMIT_PREFIX"* ]]; then
    echo "ERROR: Expected commit beginning with $EXPECTED_COMMIT_PREFIX."
    echo "Current commit: $CURRENT_COMMIT"
    exit 1
fi

if [[ ! -f "$GENERATOR" ]]; then
    echo "ERROR: Missing generator: $GENERATOR"
    exit 1
fi

if grep -q 'RESERVE_VAULT_SEED' \
    programs/treasury-router/src/constants.rs; then
    echo "ERROR: Reserve Vault Phase 1 already appears to be applied."
    exit 1
fi

echo
echo "===== APPLY RESERVE VAULT PATCH ====="

PATCH_LOG="$(mktemp)"

set +e
python3 "$GENERATOR" 2>&1 | tee "$PATCH_LOG"
PATCH_STATUS="${PIPESTATUS[0]}"
set -e

BACKUP="$(
    sed -n 's/^Backup created: //p' "$PATCH_LOG" |
    head -n 1
)"

rm -f "$PATCH_LOG"

if [[ "$PATCH_STATUS" -ne 0 ]]; then
    echo "ERROR: Protocol patch failed."
    exit "$PATCH_STATUS"
fi

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "ERROR: Could not capture the patch backup directory."
    exit 1
fi

echo
echo "===== VERIFY SOURCE ARCHITECTURE ====="

grep -q \
  'pub const RESERVE_VAULT_SEED:.*b"reserve-vault"' \
  programs/treasury-router/src/constants.rs

grep -q \
  'token::authority = treasury' \
  programs/treasury-router/src/instructions/initialize_execution_config.rs

grep -q \
  'reserve_destination.owner == treasury.key()' \
  programs/treasury-router/src/instructions/reserve.rs

grep -q \
  'Buffer.from("reserve-vault")' \
  tests/rbvr_protocol.ts

grep -q \
  'TreasuryState must be the Reserve Vault token authority' \
  tests/rbvr_protocol.ts

if grep -q 'reserveRecipientOwner' tests/rbvr_protocol.ts; then
    echo "ERROR: External reserve owner still exists in integration tests."
    exit 1
fi

echo "Canonical Reserve Vault seed: verified"
echo "TreasuryState token authority: verified"
echo "External reserve owner removed: verified"

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== ANCHOR BUILD AND CLIENT GENERATION ====="
anchor build

echo
echo "===== TYPESCRIPT CHECK ====="
npx tsc --noEmit

echo
echo "===== TARGETED RESERVE ENGINE TESTS ====="

cargo test \
  --workspace \
  engines::reserve_deployment::tests \
  -- \
  --nocapture

echo
echo "===== COMPLETE RUST REGRESSION ====="

RUST_LOG="$(mktemp)"

cargo test --workspace 2>&1 | tee "$RUST_LOG"

if ! grep -q 'test result: ok. 134 passed; 0 failed' "$RUST_LOG"; then
    echo "ERROR: Expected 134 passing Rust tests."
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
echo "===== FULL INTEGRATION REGRESSION ====="

INTEGRATION_LOG="$(mktemp)"

anchor test --validator legacy 2>&1 | tee "$INTEGRATION_LOG"

if ! grep -q '37 passing' "$INTEGRATION_LOG"; then
    echo "ERROR: Expected 37 passing integration tests."
    rm -f "$INTEGRATION_LOG"
    exit 1
fi

rm -f "$INTEGRATION_LOG"

echo
echo "===== VERIFY RESERVE VAULT TEST ASSERTIONS ====="

grep -n -A 10 \
  'TreasuryState must be the Reserve Vault token authority' \
  tests/rbvr_protocol.ts

grep -n -A 8 \
  'Reserve Vault must use the canonical PDA' \
  tests/rbvr_protocol.ts

echo
echo "===== REVIEW PROTOCOL CHANGES ====="

git diff --stat -- \
  programs/treasury-router/src/constants.rs \
  programs/treasury-router/src/instructions/initialize_execution_config.rs \
  programs/treasury-router/src/instructions/reserve.rs \
  tests/rbvr_protocol.ts

git diff --check

echo
echo "===== STAGE VERIFIED PHASE 1 FILES ====="

git add \
  programs/treasury-router/src/constants.rs \
  programs/treasury-router/src/instructions/initialize_execution_config.rs \
  programs/treasury-router/src/instructions/reserve.rs \
  tests/rbvr_protocol.ts

git diff --cached --stat

if git diff --cached --quiet; then
    echo "ERROR: No protocol changes were staged."
    exit 1
fi

echo
echo "===== COMMIT RESERVE VAULT PHASE 1 ====="

git commit -m \
  "protocol: convert reserve destination into treasury-owned vault"

echo
echo "===== TAG VERIFIED RESERVE VAULT ====="

if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Tag already exists: $TAG"
else
    git tag -a "$TAG" \
      -m "Protocol-controlled Reserve Vault funding path verified"
fi

echo
echo "===== FINAL CHECKPOINT ====="

git log --oneline --decorate -4

echo
git status --short

echo
echo "============================================================"
echo "RBVR RESERVE VAULT PHASE 1 PASSED"
echo "============================================================"
echo "Reserve Vault PDA:       [reserve-vault, TreasuryState]"
echo "Vault token authority:   TreasuryState PDA"
echo "External reserve wallet: removed"
echo "Reserve Engine tests:    12 passing"
echo "Complete Rust tests:     134 passing"
echo "Integration tests:       37 passing"
echo "Strict Clippy:            passing"
echo "Commit created:          yes"
echo "Tag:                     $TAG"
echo
echo "Next:"
echo "Add immutable Reserve Policy and surplus-only Spillway execution."
echo "============================================================"

trap - EXIT
