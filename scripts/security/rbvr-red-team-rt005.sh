#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

EXPECTED_COMMIT_PREFIX="9f76f50"
TAG="rbvr-red-team-rt005"
BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RT-005 SPILLWAY ASSAULT FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" &&
              -f "$BACKUP/tests/rbvr_protocol.ts" ]]; then
            cp "$BACKUP/tests/rbvr_protocol.ts" tests/rbvr_protocol.ts
            echo "Integration tests restored from: $BACKUP"
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "============================================================"
echo "RT-005 — HOSTILE SPILLWAY ASSAULT"
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
echo "===== INSERT RT-005 ATTACKS ====="

PATCH_LOG="$(mktemp)"

set +e
python3 rbvr-red-team-rt005.py 2>&1 | tee "$PATCH_LOG"
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
    echo "ERROR: Could not capture RT-005 backup directory."
    exit 1
fi

echo
echo "===== VERIFY ATTACK INSERTION ====="

grep -nE \
  'RT-005[A-D] rejects' \
  tests/rbvr_protocol.ts

TEST_COUNT="$(
    grep -cE '^[[:space:]]*it\("' tests/rbvr_protocol.ts
)"

echo "Integration tests found: $TEST_COUNT"

if [[ "$TEST_COUNT" -ne 42 ]]; then
    echo "ERROR: Expected exactly 42 integration tests."
    exit 1
fi

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
echo "===== RUN LIVE RT-005 ASSAULT ====="

INTEGRATION_LOG="$(mktemp)"
anchor test --validator legacy 2>&1 | tee "$INTEGRATION_LOG"

if ! grep -q '42 passing' "$INTEGRATION_LOG"; then
    echo "ERROR: Expected 42 passing integration tests."
    rm -f "$INTEGRATION_LOG"
    exit 1
fi

for attack in \
  "RT-005A rejects a substituted Reserve Policy without mutation" \
  "RT-005B rejects a substituted Reserve Vault without mutation" \
  "RT-005C rejects a substituted Spillway destination without mutation" \
  "RT-005D rejects an immediate Spillway replay without mutation"
do
    if ! grep -q "$attack" "$INTEGRATION_LOG"; then
        echo "ERROR: Attack did not execute: $attack"
        rm -f "$INTEGRATION_LOG"
        exit 1
    fi
done

rm -f "$INTEGRATION_LOG"

echo
echo "===== CREATE RT-005 REPORT ====="

mkdir -p red-team

cat > red-team/RT-005-SPILLWAY-ASSAULT.md <<EOF
# RT-005 — Hostile Spillway Assault

Generated: $(date -u '+%Y-%m-%d %H:%M:%S UTC')

Baseline commit: \`$(git rev-parse HEAD)\`

## Defensive objective

Prove that the permissionless Spillway rejects hostile account substitution
and immediate replay without transferring tokens or mutating Reserve Policy
state.

## Live attacks

1. Substituted Reserve Policy PDA.
2. Substituted Reserve Vault.
3. Substituted Liquidity Growth destination.
4. Immediate replay during the immutable deployment cooldown.

## Required rejection invariants

Every rejected transaction preserved:

- canonical Reserve Vault token balance;
- immutable Liquidity Growth destination token balance;
- Reserve Policy \`last_deployed_at\`;
- Reserve Policy \`lifetime_deployed\`;
- substituted destination balances where applicable.

## Supporting deterministic coverage

Existing Reserve Engine and Reserve Policy tests cover:

- reserve below floor;
- reserve exactly at floor;
- zero deployable surplus;
- maximum-value arithmetic;
- basis-point validation;
- floor preservation across the rate matrix;
- cooldown arithmetic;
- lifetime deployment overflow;
- failed state transition immutability.

## Results

- Rust tests: **145 passing**
- Integration tests: **42 passing**
- Live hostile Spillway attacks: **4 passing**
- Unauthorized Reserve transfer: **none**
- Unauthorized destination transfer: **none**
- Rejected-call policy mutation: **none**

## Verdict

**PASS — Spillway rejects hostile account substitution and cooldown replay
without token movement or policy mutation.**
EOF

echo
echo "===== REVIEW CHANGES ====="

git diff --check
git diff --stat -- \
  tests/rbvr_protocol.ts \
  red-team/RT-005-SPILLWAY-ASSAULT.md

echo
echo "===== STAGE VERIFIED RT-005 WORK ====="

git add \
  tests/rbvr_protocol.ts \
  red-team/RT-005-SPILLWAY-ASSAULT.md

git diff --cached --stat

if git diff --cached --quiet; then
    echo "ERROR: No RT-005 changes were staged."
    exit 1
fi

echo
echo "===== COMMIT RT-005 ====="

git commit -m \
  "security: red-team permissionless reserve spillway"

echo
echo "===== TAG RT-005 CHECKPOINT ====="

if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Tag already exists: $TAG"
else
    git tag -a "$TAG" \
      -m "RT-005 hostile Spillway assault verified"
fi

echo
echo "===== FINAL CHECKPOINT ====="

git log --oneline --decorate -7

echo
git status --short

echo
echo "============================================================"
echo "RT-005 HOSTILE SPILLWAY ASSAULT PASSED"
echo "============================================================"
echo "Substituted policy:       rejected"
echo "Substituted vault:        rejected"
echo "Substituted destination:  rejected"
echo "Immediate replay:         rejected"
echo "Unauthorized movement:    none"
echo "Policy mutation:          none"
echo "Rust tests:               145 passing"
echo "Integration tests:        42 passing"
echo "Tag:                      $TAG"
echo
echo "Next:"
echo "Add Spillway-specific fuzz target and run it in the background."
echo "============================================================"

trap - EXIT
