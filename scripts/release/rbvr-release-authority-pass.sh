#!/usr/bin/env bash
set -euo pipefail

cd ~/rocket-beaver-contract

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RELEASE AUTHORITY PASS FAILED"
        echo "============================================================"

        if [[ -n "${BACKUP_PATH:-}" && -d "${BACKUP_PATH}" ]]; then
            echo "Restoring modified files from:"
            echo "$BACKUP_PATH"

            cp \
              "$BACKUP_PATH/programs/treasury-router/src/instructions/reserve.rs" \
              programs/treasury-router/src/instructions/reserve.rs

            cp \
              "$BACKUP_PATH/programs/treasury-router/src/instructions/liquidity.rs" \
              programs/treasury-router/src/instructions/liquidity.rs

            cp \
              "$BACKUP_PATH/programs/treasury-router/src/instructions/company.rs" \
              programs/treasury-router/src/instructions/company.rs

            cp \
              "$BACKUP_PATH/programs/treasury-router/src/instructions/founder.rs" \
              programs/treasury-router/src/instructions/founder.rs

            cp \
              "$BACKUP_PATH/programs/treasury-router/src/instructions/buyback.rs" \
              programs/treasury-router/src/instructions/buyback.rs

            cp \
              "$BACKUP_PATH/tests/rbvr_protocol.ts" \
              tests/rbvr_protocol.ts

            echo "Rollback completed."
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "===== REMOVE RELEASE AUTHORITY ====="

PATCH_OUTPUT="$(python3 rbvr-remove-release-authority.py)"
echo "$PATCH_OUTPUT"

BACKUP_PATH="$(
    printf '%s\n' "$PATCH_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$BACKUP_PATH" || ! -d "$BACKUP_PATH" ]]; then
    echo "Could not determine backup path."
    exit 1
fi

echo
echo "===== FORMAT ====="
cargo fmt --all -- --check

echo
echo "===== SOURCE VERIFICATION ====="

for file in \
    programs/treasury-router/src/instructions/reserve.rs \
    programs/treasury-router/src/instructions/liquidity.rs \
    programs/treasury-router/src/instructions/company.rs \
    programs/treasury-router/src/instructions/founder.rs \
    programs/treasury-router/src/instructions/buyback.rs
do
    if grep -n "pub authority: Signer" "$file"; then
        echo "ERROR: release authority remains in $file"
        exit 1
    fi

    if grep -n "ctx.accounts.authority" "$file"; then
        echo "ERROR: runtime authority access remains in $file"
        exit 1
    fi

    if ! grep -q "authorize_autonomous_release" "$file"; then
        echo "ERROR: autonomous release guard missing from $file"
        exit 1
    fi
done

echo "All five autonomous handlers are free of release signers."

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
echo "===== FINAL RELEASE AUTHORITY INVENTORY ====="

grep -RIn \
    --include='reserve.rs' \
    --include='liquidity.rs' \
    --include='company.rs' \
    --include='founder.rs' \
    --include='buyback.rs' \
    -E "authority|authorize_autonomous_release" \
    programs/treasury-router/src/instructions || true

echo
echo "===== DIFF SUMMARY ====="
git diff --stat

echo
echo "===== STATUS ====="
git status --short

echo
echo "============================================================"
echo "RELEASE SIGNER REMOVAL PASSED"
echo "============================================================"
echo
echo "The five release instructions are now permissionless triggers."
echo "Release amounts remain controlled by autonomous Dam rules,"
echo "protocol state, pending balances, pauses, and invariants."
echo
echo "Backup retained at:"
echo "$BACKUP_PATH"

trap - EXIT
