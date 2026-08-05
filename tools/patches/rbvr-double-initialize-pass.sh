#!/usr/bin/env bash
set -euo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "DOUBLE INITIALIZATION TEST PASS FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" && -d "$BACKUP" ]]; then
            echo "Restoring:"
            echo "$BACKUP/tests/rbvr_protocol.ts"

            cp \
              "$BACKUP/tests/rbvr_protocol.ts" \
              tests/rbvr_protocol.ts

            echo "Rollback completed."
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "===== ADD DOUBLE INITIALIZATION TEST ====="

OUTPUT="$(python3 rbvr-add-double-initialize-test.py)"
echo "$OUTPUT"

BACKUP="$(
    printf '%s\n' "$OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "Could not determine backup path."
    exit 1
fi

echo
echo "===== TYPESCRIPT CHECK ====="
npx tsc --noEmit

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
echo "===== INTEGRATION TESTS ====="
anchor test --validator legacy

echo
echo "===== VERIFY NEW TEST ====="

grep -n -A 45 \
  'rejects a second protocol initialization' \
  tests/rbvr_protocol.ts

echo
echo "===== DIFF SUMMARY ====="
git diff --stat

echo
echo "===== STATUS ====="
git status --short

echo
echo "============================================================"
echo "CANONICAL PROTOCOL REINITIALIZATION TEST PASSED"
echo "============================================================"
echo
echo "Expected integration count:"
echo "17 passing"

trap - EXIT
