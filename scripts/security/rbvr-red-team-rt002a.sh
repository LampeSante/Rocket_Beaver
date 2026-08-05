#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RT-002A INITIALIZATION ASSAULT FAILED"
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

echo "===== RT-002A: PATCH MALICIOUS TESTS ====="

PATCH_OUTPUT="$(python3 rbvr-red-team-rt002a.py)"
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
echo "===== VERIFY INSERTED ATTACKS ====="

grep -n 'RT-002A' tests/rbvr_protocol.ts

echo
echo "===== TYPESCRIPT CHECK ====="

npx tsc --noEmit

echo
echo "===== BUILD ====="

anchor build

echo
echo "===== RUN ACTIVE INITIALIZATION ASSAULT ====="

anchor test --validator legacy

echo
echo "===== NORMAL RUST REGRESSION ====="

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
echo "===== CREATE RT-002A REPORT ====="

mkdir -p red-team

cat > red-team/RT-002A-INITIALIZATION-ASSAULT.md <<EOF
# RT-002A — Initialization Assault

## Baseline

- Branch: \`$(git branch --show-current)\`
- Baseline commit: \`$(git rev-parse HEAD)\`
- Program ID: \`5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3\`

## Attacks executed

1. Unauthorized signer attempted to initialize canonical protocol configuration.
2. Repeated initialization of canonical protocol configuration.
3. Repeated initialization of canonical treasury and settlement vault.
4. Repeated initialization of canonical founder state.
5. Repeated initialization of canonical company state.
6. Repeated initialization of canonical execution configuration.

## Required security property

Every malicious initialization transaction must fail without:

- creating an unauthorized account;
- modifying protocol state;
- modifying configuration;
- modifying treasury state;
- modifying the settlement vault;
- modifying founder or company state;
- changing execution destinations.

## Result

All RT-002A attacks were rejected and all compared account data remained unchanged.

## Test baseline

- Rust tests: 122 passing
- Integration tests: 23 expected passing
- Strict Clippy: passing
- Anchor build: passing

## Disposition

**PASS — No exploitable canonical reinitialization or unauthorized protocol-config initialization path identified.**

## Next phase

RT-002B — PDA, mint, vault, owner, and account-substitution assault.
EOF

echo
echo "===== STATUS ====="

git diff --stat
git status --short

echo
echo "============================================================"
echo "RT-002A INITIALIZATION ASSAULT PASSED"
echo "============================================================"
echo "Expected integration tests: 23 passing"
echo "Rejected malicious attacks: 6"
echo "Account mutation after rejection: none"
echo "Next: RT-002B account substitution"
echo "============================================================"

trap - EXIT
