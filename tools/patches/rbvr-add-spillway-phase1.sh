#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

EXPECTED_COMMIT_PREFIX="f21896a"
TAG="rbvr-spillway-v1"

FILES=(
  "programs/treasury-router/src/errors/mod.rs"
  "programs/treasury-router/src/events/mod.rs"
  "programs/treasury-router/src/lib.rs"
  "programs/treasury-router/src/instructions/mod.rs"
  "programs/treasury-router/src/instructions/spillway_release.rs"
  "tests/rbvr_protocol.ts"
)

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RBVR SPILLWAY PHASE 1 FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" && -d "$BACKUP" ]]; then
            for file in \
              programs/treasury-router/src/errors/mod.rs \
              programs/treasury-router/src/events/mod.rs \
              programs/treasury-router/src/lib.rs \
              programs/treasury-router/src/instructions/mod.rs \
              tests/rbvr_protocol.ts
            do
                if [[ -f "$BACKUP/$file" ]]; then
                    cp "$BACKUP/$file" "$file"
                fi
            done

            rm -f \
              programs/treasury-router/src/instructions/spillway_release.rs

            cargo fmt --all >/dev/null 2>&1 || true

            echo "Original files restored from:"
            echo "$BACKUP"
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "============================================================"
echo "RBVR PERMISSIONLESS SPILLWAY — PHASE 1"
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
echo "===== ADD SPILLWAY INSTRUCTION ====="

PATCH_LOG="$(mktemp)"

set +e
python3 rbvr-add-spillway-phase1.py 2>&1 | tee "$PATCH_LOG"
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
echo "===== VERIFY PERMISSIONLESS DESIGN ====="

python3 <<'PYCHECK'
from pathlib import Path
import re

path = Path(
    "programs/treasury-router/src/instructions/spillway_release.rs"
)

if not path.exists():
    raise RuntimeError(f"Missing Spillway instruction: {path}")

source = path.read_text()

# Remove comments before checking for authority-bearing account declarations.
without_comments = re.sub(r"//.*?$|/\*.*?\*/", "", source, flags=re.M | re.S)

if re.search(r"\bSigner\s*<", without_comments):
    raise RuntimeError(
        "Spillway unexpectedly requires a caller signer"
    )

# Normalize every whitespace run so multiline Rust expressions can be
# verified without depending on cargo-fmt line wrapping.
normalized = " ".join(source.split())

required = {
    "Treasury-owned Reserve Vault":
        "reserve_vault.owner == treasury.key()",

    "Protected floor enforcement":
        "evaluation.remaining_reserve >= evaluation.reserve_floor",

    "Immutable liquidity destination linkage":
        "execution_config.liquidity_destination == liquidity_destination.key()",

    "Canonical Reserve Vault PDA":
        "RESERVE_VAULT_SEED, treasury.key().as_ref()",

    "Immutable Reserve Policy PDA":
        "RESERVE_POLICY_SEED, treasury.key().as_ref()",

    "Deterministic Reserve Engine":
        "evaluate_reserve_deployment(",

    "Cooldown enforcement":
        "cooldown_has_elapsed(clock.unix_timestamp)",

    "Deployment accounting":
        "record_deployment(",

    "Treasury PDA signing":
        "CpiContext::new_with_signer(",
}

missing = [
    label
    for label, fragment in required.items()
    if fragment not in normalized
]

if missing:
    raise RuntimeError(
        "Spillway source verification failed: "
        + ", ".join(missing)
    )

print("Caller signer: none")
print("Reserve Vault authority: TreasuryState")
print("Destination: immutable Liquidity Growth account")
print("Protected floor: enforced on-chain")
print("Cooldown: enforced on-chain")
print("Deployment accounting: enabled")
PYCHECK

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
echo "===== RESERVE ENGINE AND POLICY TESTS ====="

cargo test \
    --workspace \
    engines::reserve_deployment::tests \
    -- \
    --nocapture

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
echo "===== LIVE SPILLWAY INTEGRATION TEST ====="

INTEGRATION_LOG="$(mktemp)"
anchor test --validator legacy 2>&1 | tee "$INTEGRATION_LOG"

if ! grep -q '38 passing' "$INTEGRATION_LOG"; then
    echo "ERROR: Expected 38 passing integration tests."
    rm -f "$INTEGRATION_LOG"
    exit 1
fi

if ! grep -q \
    'deploys only Reserve surplus through the permissionless Spillway' \
    "$INTEGRATION_LOG"
then
    echo "ERROR: Spillway integration test did not execute."
    rm -f "$INTEGRATION_LOG"
    exit 1
fi

rm -f "$INTEGRATION_LOG"

echo
echo "===== REVIEW CHANGES ====="

git diff --check
git diff --stat -- "${FILES[@]}"

echo
echo "===== STAGE VERIFIED SPILLWAY FILES ====="

git add "${FILES[@]}"
git diff --cached --stat

if git diff --cached --quiet; then
    echo "ERROR: No Spillway changes were staged."
    exit 1
fi

echo
echo "===== COMMIT SPILLWAY PHASE 1 ====="

git commit -m \
    "protocol: add permissionless surplus-only reserve spillway"

echo
echo "===== TAG SPILLWAY CHECKPOINT ====="

if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Tag already exists: $TAG"
else
    git tag -a "$TAG" \
      -m "Permissionless surplus-only Reserve Spillway verified"
fi

echo
echo "===== FINAL CHECKPOINT ====="

git log --oneline --decorate -6

echo
git status --short

echo
echo "============================================================"
echo "RBVR PERMISSIONLESS SPILLWAY PHASE 1 PASSED"
echo "============================================================"
echo "Caller authority:        none"
echo "Reserve source:          Treasury-owned Reserve Vault"
echo "Destination:             immutable Liquidity Growth"
echo "Protected floor:         enforced"
echo "Surplus calculation:     Reserve Deployment Engine"
echo "Cooldown:                Reserve Policy"
echo "Lifetime accounting:     Reserve Policy"
echo "Rust tests:              145 expected"
echo "Integration tests:       38 expected"
echo "Tag:                     $TAG"
echo
echo "Next:"
echo "RT-005 hostile Spillway account-substitution, cooldown,"
echo "floor-breach and replay assault."
echo "============================================================"

trap - EXIT
