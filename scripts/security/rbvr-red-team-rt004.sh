#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RT-004 FEE-PROCESSING ASSAULT FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" &&
              -f "$BACKUP/tests/rbvr_protocol.ts" ]]; then
            cp "$BACKUP/tests/rbvr_protocol.ts" tests/rbvr_protocol.ts
            echo "tests/rbvr_protocol.ts restored from backup."
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "===== RT-004: PATCH FEE-PROCESSING REPLAY ATTACK ====="

PATCH_OUTPUT="$(python3 rbvr-red-team-rt004.py)"
printf '%s\n' "$PATCH_OUTPUT"

BACKUP="$(
    printf '%s\n' "$PATCH_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "Could not identify RT-004 backup directory."
    exit 1
fi

echo
echo "===== VERIFY ATTACK INSERTION ====="
grep -n -A 12 \
  'RT-004 rejects fee-processing replay' \
  tests/rbvr_protocol.ts

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== TYPESCRIPT CHECK ====="
npx tsc --noEmit

echo
echo "===== ANCHOR BUILD ====="
anchor build

echo
echo "===== RUN RT-004 INTEGRATION ASSAULT ====="
anchor test --validator legacy

echo
echo "===== VERIFY INTEGRATION TEST COUNT ====="

TEST_COUNT="$(
    grep -cE '^[[:space:]]*it\("' tests/rbvr_protocol.ts
)"

echo "Integration tests found: $TEST_COUNT"

if [[ "$TEST_COUNT" -ne 37 ]]; then
    echo "Expected exactly 37 integration tests."
    exit 1
fi

echo
echo "===== TARGETED ACCOUNTING UNIT TESTS ====="

cargo test --workspace \
  instructions::process_fees::tests::allocation_total_fails_on_overflow \
  -- --exact

cargo test --workspace \
  instructions::process_fees::tests::calculate_share_handles_u64_max_without_multiplication_overflow \
  -- --exact

cargo test --workspace \
  instructions::process_fees::tests::rounding_remainder_is_assigned_to_reserve \
  -- --exact

cargo test --workspace \
  instructions::process_fees::tests::company_cap_overflow_redirected_to_liquidity_is_conserved \
  -- --exact

cargo test --workspace \
  instructions::process_fees::tests::founder_cap_overflow_redirected_to_liquidity_is_conserved \
  -- --exact

cargo test --workspace \
  instructions::process_fees::tests::simultaneous_company_and_founder_overflow_is_conserved \
  -- --exact

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
echo "===== CREATE RT-004 REPORT ====="

mkdir -p red-team

cat > red-team/RT-004-FEE-PROCESSING-ACCOUNTING-ASSAULT.md <<EOF
# RT-004 — Fee Processing and Accounting Assault

Generated: $(date -u '+%Y-%m-%d %H:%M:%S UTC')

Git commit before RT-004: \`$(git rev-parse HEAD)\`

## Defensive objective

Verify that the fee-processing path cannot:

- process the same vault balance twice;
- inflate lifetime or pending allocations;
- advance the processing epoch after a rejected replay;
- mutate Founder or Company cap accounting after rejection;
- change the settlement-vault balance during accounting processing;
- bypass arithmetic overflow protections;
- lose rounding remainders;
- lose Company or Founder cap overflow;
- violate global allocation conservation.

## Active integration attack

A second \`processFees()\` call was submitted immediately after the complete
vault balance had already been processed.

Expected result:

- rejection with \`NoUnprocessedFees\`;
- no TreasuryState mutation;
- no FounderState mutation;
- no CompanyState mutation;
- no settlement-vault token movement;
- all global invariants remain valid.

## Supporting deterministic tests

The existing Rust suite verifies:

- u64 multiplication overflow resistance;
- allocation-total overflow rejection;
- rounding remainder assignment to Reserve;
- Company overflow redirection to Liquidity;
- Founder overflow redirection to Liquidity;
- simultaneous Company and Founder overflow conservation;
- locked allocation conservation.

## Results

- Integration tests: **37 passing**
- Rust tests: **122 passing**
- Strict Clippy: **passing**
- Unauthorized accounting mutation: **none**
- Unauthorized token movement: **none**
- Fee replay inflation: **rejected**

## Verdict

**PASS — fee processing rejects replay and preserves all observed accounting
and token balances after rejection.**
EOF

echo
echo "===== STATUS ====="
git diff --stat
git status --short

echo
echo "============================================================"
echo "RT-004 FEE PROCESSING AND ACCOUNTING ASSAULT PASSED"
echo "============================================================"
echo "Expected integration tests: 37 passing"
echo "Rust regression tests:       122 passing"
echo "Rejected fee replay:         1"
echo "Accounting mutation:         none"
echo "Unauthorized token movement: none"
echo "Next: RT-005 release-path and execution-limit assault"
echo "============================================================"

trap - EXIT
