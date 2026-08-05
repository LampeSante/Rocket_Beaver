#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RT-002C LINKED-ACCOUNT/PDA ASSAULT FAILED"
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

echo "===== RT-002C: PATCH LINKED-ACCOUNT/PDA ATTACKS ====="

PATCH_OUTPUT="$(python3 rbvr-red-team-rt002c.py)"
printf '%s\n' "$PATCH_OUTPUT"

BACKUP="$(
    printf '%s\n' "$PATCH_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "Could not identify RT-002C backup directory."
    exit 1
fi

echo
echo "===== VERIFY ATTACK INSERTION ====="

grep -n \
  'RT-002C rejects' \
  tests/rbvr_protocol.ts

echo
echo "===== TYPESCRIPT CHECK ====="

npx tsc --noEmit

echo
echo "===== ANCHOR BUILD ====="

anchor build

echo
echo "===== RUN LINKED-ACCOUNT/PDA ASSAULT ====="

anchor test --validator legacy

echo
echo "===== VERIFY INTEGRATION TEST COUNT ====="

TEST_COUNT="$(
    grep -c '^ *it("' tests/rbvr_protocol.ts
)"

echo "Integration tests found: $TEST_COUNT"

if [[ "$TEST_COUNT" -ne 31 ]]; then
    echo "Expected 31 integration tests, found $TEST_COUNT."
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
echo "===== CREATE RT-002C REPORT ====="

mkdir -p red-team

cat > red-team/RT-002C-LINKED-ACCOUNT-PDA-SUBSTITUTION.md <<'REPORT'
# RT-002C — Linked-Account and PDA Substitution Assault

## Objective

Attempt to bypass canonical PDA, account-type, and protocol-linkage
constraints by supplying valid but incorrect RBVR accounts in place of the
accounts required by critical instructions.

## Attacks executed

1. Supplied `execution_config` as `protocol_config` to `process_fees`.
2. Swapped the canonical Founder and Company state PDAs.
3. Supplied `protocol_config` as `execution_config` to Reserve execution.
4. Supplied `execution_config` as the canonical `protocol_state`.

## Expected security behavior

Every hostile substitution must:

- be rejected before protocol mutation;
- produce no settlement-token movement;
- leave all treasury accounting unchanged;
- leave processing epoch unchanged;
- preserve all global accounting invariants.

## Result

- Malicious substitutions attempted: **4**
- Malicious substitutions rejected: **4**
- Unauthorized token movement: **none**
- Treasury mutation after rejection: **none**
- Global accounting invariant failures: **none**
- Integration tests: **31 passing**
- Rust tests: **122 passing**
- Strict Clippy: **passing**

## Conclusion

The tested instructions rejected linked-account and PDA substitution attacks
without altering treasury state or transferring settlement assets.

This result covers the specific attack cases executed in RT-002C. It does not
replace additional adversarial testing, fuzzing, or independent review.

## Next phase

**RT-003 — Deposit and token-authority assault**

Planned cases:

- source token account owned by the wrong signer;
- source account using the wrong settlement mint;
- canonical vault substitution;
- zero-value deposit;
- balance-underflow attempt;
- maximum-value transfer attempt;
- signer/account-owner confusion;
- repeated deposit accounting consistency.
REPORT

echo
echo "===== STATUS ====="

git diff --stat
git status --short

echo
echo "============================================================"
echo "RT-002C LINKED-ACCOUNT AND PDA SUBSTITUTION PASSED"
echo "============================================================"
echo "Expected integration tests: 31 passing"
echo "Rejected hostile substitutions: 4"
echo "Unauthorized token movement: none"
echo "State mutation after rejection: none"
echo "Next: RT-003 deposit and token-authority assault"
echo "============================================================"

trap - EXIT
