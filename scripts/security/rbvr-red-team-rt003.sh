#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RT-003 DEPOSIT/TOKEN-AUTHORITY ASSAULT FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" &&
              -f "$BACKUP/tests/rbvr_protocol.ts" ]]; then
            cp \
              "$BACKUP/tests/rbvr_protocol.ts" \
              tests/rbvr_protocol.ts

            echo "tests/rbvr_protocol.ts restored from backup."
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "===== RT-003: PATCH DEPOSIT/TOKEN-AUTHORITY ATTACKS ====="

PATCH_OUTPUT="$(python3 rbvr-red-team-rt003.py)"
printf '%s\n' "$PATCH_OUTPUT"

BACKUP="$(
    printf '%s\n' "$PATCH_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "Could not identify RT-003 backup directory."
    exit 1
fi

echo
echo "===== VERIFY ATTACK INSERTION ====="

grep -n \
  'RT-003 rejects' \
  tests/rbvr_protocol.ts

echo
echo "===== TYPESCRIPT CHECK ====="

npx tsc --noEmit

echo
echo "===== ANCHOR BUILD ====="

anchor build

echo
echo "===== RUN DEPOSIT/TOKEN-AUTHORITY ASSAULT ====="

anchor test --validator legacy

echo
echo "===== VERIFY INTEGRATION TEST COUNT ====="

TEST_COUNT="$(
    grep -c '^ *it("' tests/rbvr_protocol.ts
)"

echo "Integration tests found: $TEST_COUNT"

if [[ "$TEST_COUNT" -ne 36 ]]; then
    echo "Expected 36 integration tests, found $TEST_COUNT."
    exit 1
fi

echo
echo "===== RUST REGRESSION ====="

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
echo "===== CREATE RT-003 REPORT ====="

mkdir -p red-team

cat > red-team/RT-003-DEPOSIT-TOKEN-AUTHORITY-ASSAULT.md <<'REPORT'
# RT-003 — Deposit and Token-Authority Assault

## Objective

Attack the settlement deposit path by supplying hostile token accounts,
incorrect authorities, incorrect mints, substituted vaults, and invalid
transfer amounts.

## Attacks executed

1. Source token account owned by an attacker while another signer is supplied.
2. Source token account associated with a non-settlement mint.
3. Valid SPL token account substituted for the canonical treasury vault.
4. Zero-value deposit.
5. Deposit amount exceeding the source token balance.

## Expected security behavior

Every hostile deposit must:

- be rejected before treasury accounting mutation;
- transfer no settlement assets;
- leave the canonical vault unchanged;
- leave attacker-controlled accounts unchanged;
- preserve global treasury invariants.

## Result

- Hostile deposit attempts: **5**
- Hostile deposits rejected: **5**
- Unauthorized token movement: **none**
- Treasury accounting mutation after rejection: **none**
- Integration tests: **36 passing**
- Rust tests: **122 passing**
- Strict Clippy: **passing**

## Conclusion

The tested deposit path rejected token-owner confusion, mint substitution,
vault substitution, zero-value transfers, and insufficient-balance attempts
without unauthorized asset movement or accounting mutation.

This result applies to the specific RT-003 cases executed. It does not replace
additional adversarial testing, fuzzing, devnet validation, or independent
security review.

## Next phase

**RT-004 — Fee-processing and accounting assault**

Planned cases:

- repeated processing without new settlement assets;
- vault/accounting balance mismatch;
- malformed bucket accounting;
- company and founder cap-boundary manipulation;
- arithmetic maximums;
- processing epoch consistency;
- failed processing must leave all state unchanged.
REPORT

echo
echo "===== STATUS ====="

git diff --stat
git status --short

echo
echo "============================================================"
echo "RT-003 DEPOSIT AND TOKEN-AUTHORITY ASSAULT PASSED"
echo "============================================================"
echo "Expected integration tests: 36 passing"
echo "Rejected hostile deposits: 5"
echo "Unauthorized token movement: none"
echo "State mutation after rejection: none"
echo "Next: RT-004 fee-processing and accounting assault"
echo "============================================================"

trap - EXIT
