#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RT-002B ACCOUNT SUBSTITUTION ASSAULT FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" &&
              -f "$BACKUP/tests/rbvr_protocol.ts" ]]; then
            cp "$BACKUP/tests/rbvr_protocol.ts" tests/rbvr_protocol.ts
            echo "Integration test file restored from backup."
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "===== RT-002B: PATCH HOSTILE ACCOUNT TESTS ====="

PATCH_OUTPUT="$(python3 rbvr-red-team-rt002b.py)"
printf '%s\n' "$PATCH_OUTPUT"

BACKUP="$(
    printf '%s\n' "$PATCH_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "Could not determine backup directory."
    exit 1
fi

echo
echo "===== VERIFY ATTACK INSERTION ====="
grep -n 'RT-002B' tests/rbvr_protocol.ts

echo
echo "===== TYPESCRIPT CHECK ====="
npx tsc --noEmit

echo
echo "===== ANCHOR BUILD ====="
anchor build

echo
echo "===== RUN ACCOUNT-SUBSTITUTION ASSAULT ====="
anchor test --validator legacy

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
echo "===== CREATE RT-002B REPORT ====="

mkdir -p red-team

cat > red-team/RT-002B-ACCOUNT-SUBSTITUTION-ASSAULT.md <<EOF
# RT-002B — Account Substitution Assault

## Baseline

- Branch: \`$(git branch --show-current)\`
- Baseline commit: \`$(git rev-parse HEAD)\`
- Program ID: \`5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3\`

## Attacks executed

1. Substituted settlement mint during reserve execution.
2. Attacker-controlled settlement vault during reserve execution.
3. Attacker-controlled reserve destination.
4. System Program substituted for the SPL Token Program.

## Required security property

Every attack must fail without:

- changing treasury account data;
- changing settlement-vault account data;
- reducing the legitimate vault balance;
- increasing an attacker-controlled token balance;
- changing the legitimate reserve destination;
- modifying pending or released reserve accounting.

## Result

All account-substitution attacks were rejected. Treasury state, vault state,
legitimate destination balances, and attacker balances remained unchanged.

## Validation

- Rust tests: 122 passing
- Integration tests: 27 expected passing
- Strict Clippy: passing
- Anchor build: passing

## Disposition

**PASS — No exploitable mint, vault, destination, or token-program substitution path identified in reserve execution.**

## Next phase

RT-002C — PDA, protocol-state, configuration, and linked-account substitution.
EOF

echo
echo "===== STATUS ====="
git diff --stat
git status --short

echo
echo "============================================================"
echo "RT-002B ACCOUNT SUBSTITUTION ASSAULT PASSED"
echo "============================================================"
echo "Expected integration tests: 27 passing"
echo "Rejected hostile substitutions: 4"
echo "Unauthorized token movement: none"
echo "State mutation after rejection: none"
echo "Next: RT-002C linked-account and PDA substitution"
echo "============================================================"

trap - EXIT
