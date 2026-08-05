#!/usr/bin/env bash
set -euo pipefail

cd ~/rocket-beaver-contract

EVENT_BACKUP=""
SIGNER_BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "PERMISSIONLESS RELEASE PASS FAILED"
        echo "============================================================"

        if [[ -n "$EVENT_BACKUP" && -d "$EVENT_BACKUP" ]]; then
            echo "Restoring event-stage files from:"
            echo "$EVENT_BACKUP"

            cp \
              "$EVENT_BACKUP/programs/treasury-router/src/events.rs" \
              programs/treasury-router/src/events.rs

            for file in reserve liquidity company founder buyback; do
                cp \
                  "$EVENT_BACKUP/programs/treasury-router/src/instructions/$file.rs" \
                  "programs/treasury-router/src/instructions/$file.rs"
            done
        fi

        if [[ -n "$SIGNER_BACKUP" && -d "$SIGNER_BACKUP" ]]; then
            echo "Restoring signer-stage files from:"
            echo "$SIGNER_BACKUP"

            for file in reserve liquidity company founder buyback; do
                cp \
                  "$SIGNER_BACKUP/programs/treasury-router/src/instructions/$file.rs" \
                  "programs/treasury-router/src/instructions/$file.rs"
            done

            cp \
              "$SIGNER_BACKUP/tests/rbvr_protocol.ts" \
              tests/rbvr_protocol.ts
        fi

        echo "Rollback completed."
    fi

    exit "$code"
}

trap rollback EXIT

echo "===== REMOVE EVENT AUTHORITY FIELDS ====="

EVENT_OUTPUT="$(python3 rbvr-remove-release-event-authority.py)"
echo "$EVENT_OUTPUT"

EVENT_BACKUP="$(
    printf '%s\n' "$EVENT_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$EVENT_BACKUP" || ! -d "$EVENT_BACKUP" ]]; then
    echo "Could not determine event backup path."
    exit 1
fi

echo
echo "===== REMOVE RELEASE SIGNERS ====="

SIGNER_OUTPUT="$(python3 rbvr-remove-release-authority.py)"
echo "$SIGNER_OUTPUT"

SIGNER_BACKUP="$(
    printf '%s\n' "$SIGNER_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$SIGNER_BACKUP" || ! -d "$SIGNER_BACKUP" ]]; then
    echo "Could not determine signer backup path."
    exit 1
fi

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== VERIFY NO RELEASE SIGNERS ====="

for file in \
    programs/treasury-router/src/instructions/reserve.rs \
    programs/treasury-router/src/instructions/liquidity.rs \
    programs/treasury-router/src/instructions/company.rs \
    programs/treasury-router/src/instructions/founder.rs \
    programs/treasury-router/src/instructions/buyback.rs
do
    if grep -n "pub authority: Signer" "$file"; then
        echo "ERROR: signer remains in $file"
        exit 1
    fi

    if grep -n "ctx.accounts.authority" "$file"; then
        echo "ERROR: authority access remains in $file"
        exit 1
    fi

    if ! grep -q "authorize_autonomous_release" "$file"; then
        echo "ERROR: autonomous guard missing from $file"
        exit 1
    fi
done

echo "All five release instructions are signer-free."

echo
echo "===== VERIFY RELEASE EVENTS ====="

for event in \
    ReserveExecutionAuthorized \
    LiquidityExecutionAuthorized \
    CompanyExecutionAuthorized \
    FounderExecutionAuthorized \
    BuybackExecutionAuthorized
do
    if sed -n "/pub struct $event/,/^}/p" \
        programs/treasury-router/src/events.rs |
        grep -q "pub authority:"
    then
        echo "ERROR: authority remains in event $event"
        exit 1
    fi
done

echo "All five release events are authority-free."

echo
echo "===== STRICT CLIPPY ====="
cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- \
    -D warnings

echo
echo "===== RUST TESTS ====="
cargo test --workspace

echo
echo "===== ANCHOR BUILD ====="
anchor build

echo
echo "===== TYPESCRIPT CHECK ====="
npx tsc --noEmit

echo
echo "===== INTEGRATION TESTS ====="
anchor test --validator legacy

echo
echo "===== FINAL AUTONOMOUS RELEASE INVENTORY ====="

grep -RIn \
    --include='reserve.rs' \
    --include='liquidity.rs' \
    --include='company.rs' \
    --include='founder.rs' \
    --include='buyback.rs' \
    -E "Signer<'info>|ctx\.accounts\.authority|authorize_autonomous_release" \
    programs/treasury-router/src/instructions || true

echo
echo "===== DIFF SUMMARY ====="
git diff --stat

echo
echo "===== STATUS ====="
git status --short

echo
echo "============================================================"
echo "PERMISSIONLESS AUTONOMOUS RELEASE PASSED"
echo "============================================================"
echo
echo "The five release instructions now:"
echo "- accept no caller-selected amount"
echo "- require no release authority signer"
echo "- use fixed protocol destinations"
echo "- remain controlled by Dam limits and protocol invariants"
echo
echo "Backups retained at:"
echo "$EVENT_BACKUP"
echo "$SIGNER_BACKUP"

trap - EXIT
