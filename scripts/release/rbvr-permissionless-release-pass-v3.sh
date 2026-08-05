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

        if [[ -n "$EVENT_BACKUP" && -d "$EVENT_BACKUP" ]]; then
            echo "Restoring event-stage files from:"
            echo "$EVENT_BACKUP"

            while IFS= read -r -d '' source; do
                relative="${source#"$EVENT_BACKUP/"}"
                destination="$relative"

                mkdir -p "$(dirname "$destination")"
                cp "$source" "$destination"
            done < <(
                find "$EVENT_BACKUP" -type f -print0
            )
        fi

        echo "Rollback completed."
    fi

    exit "$code"
}

trap rollback EXIT

echo "===== LOCATE AND REMOVE EVENT AUTHORITY ====="

EVENT_OUTPUT="$(
    python3 rbvr-remove-release-event-authority-v2.py
)"

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

SIGNER_OUTPUT="$(
    python3 rbvr-remove-release-authority.py
)"

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
echo "===== REMOVE REMAINING RELEASE TEST AUTHORITIES ====="

TEST_OUTPUT="$(
    python3 rbvr-remove-release-test-authority.py
)"

echo "$TEST_OUTPUT"

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== VERIFY HANDLERS ====="

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
        echo "ERROR: autonomous release guard missing from $file"
        exit 1
    fi
done

echo "All five release handlers are signer-free."

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
echo "===== RELEASE AUTHORITY INVENTORY ====="

grep -RIn \
    --include='*.rs' \
    -E \
    "ReserveExecutionAuthorized|LiquidityExecutionAuthorized|CompanyExecutionAuthorized|FounderExecutionAuthorized|BuybackExecutionAuthorized|pub authority: Signer|ctx\.accounts\.authority|authorize_autonomous_release" \
    programs/treasury-router/src || true

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
echo "The five release instructions now require no human signer."
echo "Amounts and destinations remain protocol-controlled."
echo
echo "Backups retained at:"
echo "$EVENT_BACKUP"
echo "$SIGNER_BACKUP"

trap - EXIT
