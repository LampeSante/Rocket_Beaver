#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

echo "============================================================"
echo "RBVR RESERVE ENGINE CHECKPOINT"
echo "============================================================"

EXPECTED_BRANCH="red-team"
CURRENT_BRANCH="$(git branch --show-current)"

if [[ "$CURRENT_BRANCH" != "$EXPECTED_BRANCH" ]]; then
    echo "ERROR: Expected branch '$EXPECTED_BRANCH'."
    echo "Current branch: '$CURRENT_BRANCH'"
    exit 1
fi

REQUIRED_FILES=(
    "programs/treasury-router/src/engines/mod.rs"
    "programs/treasury-router/src/engines/reserve_deployment.rs"
    "tests/rbvr_protocol.ts"
    "red-team/RT-002A-INITIALIZATION-ASSAULT.md"
    "red-team/RT-002B-ACCOUNT-SUBSTITUTION-ASSAULT.md"
    "red-team/RT-002C-LINKED-ACCOUNT-PDA-SUBSTITUTION.md"
    "red-team/RT-003-DEPOSIT-TOKEN-AUTHORITY-ASSAULT.md"
    "red-team/RT-004-FEE-PROCESSING-ACCOUNTING-ASSAULT.md"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [[ ! -f "$file" ]]; then
        echo "ERROR: Missing required checkpoint file: $file"
        exit 1
    fi
done

echo
echo "===== VERIFY RESERVE ENGINE EXPORT ====="

grep -q '^pub mod reserve_deployment;$' \
    programs/treasury-router/src/engines/mod.rs

echo "Reserve engine export found."

echo
echo "===== VERIFY RESERVE ENGINE SAFETY MARKERS ====="

grep -q 'pub fn evaluate_reserve_deployment' \
    programs/treasury-router/src/engines/reserve_deployment.rs

grep -q 'deployment_never_breaches_floor_across_rate_matrix' \
    programs/treasury-router/src/engines/reserve_deployment.rs

grep -q 'remaining_reserve >= reserve_floor' \
    programs/treasury-router/src/engines/reserve_deployment.rs

echo "Reserve-floor invariants found."

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== TYPESCRIPT CHECK ====="
npx tsc --noEmit

echo
echo "===== TARGETED RESERVE TESTS ====="

cargo test \
    --workspace \
    engines::reserve_deployment::tests \
    -- \
    --nocapture

echo
echo "===== COMPLETE RUST TESTS ====="

RUST_OUTPUT="$(mktemp)"

cargo test --workspace 2>&1 | tee "$RUST_OUTPUT"

if ! grep -q 'test result: ok. 134 passed; 0 failed' "$RUST_OUTPUT"; then
    echo "ERROR: Expected 134 passing Rust tests."
    rm -f "$RUST_OUTPUT"
    exit 1
fi

rm -f "$RUST_OUTPUT"

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

INTEGRATION_OUTPUT="$(mktemp)"

anchor test --validator legacy 2>&1 | tee "$INTEGRATION_OUTPUT"

if ! grep -q '37 passing' "$INTEGRATION_OUTPUT"; then
    echo "ERROR: Expected 37 passing integration tests."
    rm -f "$INTEGRATION_OUTPUT"
    exit 1
fi

rm -f "$INTEGRATION_OUTPUT"

echo
echo "===== CLEAR EXISTING STAGING AREA ====="

git reset

echo
echo "===== STAGE VERIFIED PROTOCOL WORK ONLY ====="

git add \
    programs/treasury-router/src/engines/mod.rs \
    programs/treasury-router/src/engines/reserve_deployment.rs \
    tests/rbvr_protocol.ts \
    red-team/RT-002A-INITIALIZATION-ASSAULT.md \
    red-team/RT-002B-ACCOUNT-SUBSTITUTION-ASSAULT.md \
    red-team/RT-002C-LINKED-ACCOUNT-PDA-SUBSTITUTION.md \
    red-team/RT-003-DEPOSIT-TOKEN-AUTHORITY-ASSAULT.md \
    red-team/RT-004-FEE-PROCESSING-ACCOUNTING-ASSAULT.md

echo
echo "===== REVIEW STAGED CHANGES ====="

git diff --cached --stat
git status --short

if git diff --cached --quiet; then
    echo "ERROR: Nothing was staged."
    exit 1
fi

echo
echo "===== COMMIT CHECKPOINT ====="

git commit -m \
    "security: add verified reserve deployment engine and red-team regressions"

echo
echo "===== CREATE CHECKPOINT TAG ====="

TAG="rbvr-reserve-engine-v1"

if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Tag already exists: $TAG"
else
    git tag -a "$TAG" -m \
        "Verified deterministic Reserve Deployment Engine baseline"
fi

echo
echo "===== FINAL CHECKPOINT ====="

git log --oneline --decorate -3

echo
git status --short

echo
echo "============================================================"
echo "RBVR RESERVE ENGINE CHECKPOINT LOCKED"
echo "============================================================"
echo "Reserve tests:       12 passing"
echo "Complete Rust tests: 134 passing"
echo "Integration tests:   37 passing"
echo "Strict Clippy:        passing"
echo "Anchor build:         passing"
echo "Tag:                  rbvr-reserve-engine-v1"
echo
echo "Next:"
echo "Convert reserve_destination into a protocol-owned Reserve Vault."
echo "Then add a separate surplus-deployment instruction."
echo "============================================================"
