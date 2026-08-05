#!/usr/bin/env bash
set -euo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "DOUBLE INITIALIZATION PASS FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" && -f "$BACKUP/tests/rbvr_protocol.ts" ]]; then
            cp "$BACKUP/tests/rbvr_protocol.ts" tests/rbvr_protocol.ts
            echo "The integration test file was restored."
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "===== PATCH TEST SUITE ====="

PATCH_OUTPUT="$(python3 rbvr-add-double-initialize-test-v2.py)"
printf '%s\n' "$PATCH_OUTPUT"

BACKUP="$(
    printf '%s\n' "$PATCH_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "Could not identify the backup directory."
    exit 1
fi

echo
echo "===== VERIFY INSERTION ====="
grep -n -A 48 \
  'rejects a second protocol initialization' \
  tests/rbvr_protocol.ts

echo
echo "===== TYPESCRIPT ====="
npx tsc --noEmit

echo
echo "===== RUST TESTS ====="
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
echo "===== ANCHOR BUILD ====="
anchor build

echo
echo "===== INTEGRATION TESTS ====="
anchor test --validator legacy

echo
echo "===== TEST COUNT ====="
grep -c '^ *it("' tests/rbvr_protocol.ts

echo
echo "===== DIFF ====="
git diff --stat
git status --short

echo
echo "============================================================"
echo "CANONICAL REINITIALIZATION PROTECTION VERIFIED"
echo "============================================================"
echo "Expected integration result: 17 passing"

trap - EXIT
