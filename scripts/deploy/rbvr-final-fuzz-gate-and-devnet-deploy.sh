#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

PROGRAM_ID="5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3"
DEVNET_URL="https://api.devnet.solana.com"
WALLET="/home/alec_elliott/rocket-beaver-dev/keys/local-deployment.json"
PROGRAM_KEYPAIR="target/deploy/treasury_router-keypair.json"
PROGRAM_BINARY="target/deploy/treasury_router.so"

EXPECTED_COMMIT_PREFIX="a89e8b7"
TARGET="spillway_invariants"
PID_FILE=".rbvr-fuzz-runs/${TARGET}.pid"
REPORT="RBVR-DEVNET-DEPLOYMENT-RESULT.md"
RAW_DIR=".rbvr-devnet-deployment"

MINIMUM_BALANCE="4.5"

mkdir -p "$RAW_DIR"

echo "============================================================"
echo "RBVR FINAL FUZZ GATE AND DEVNET DEPLOYMENT"
echo "============================================================"
echo "Program: $PROGRAM_ID"
echo "RPC:     $DEVNET_URL"
echo

# -------------------------------------------------------------
# 1. Verify source checkpoint.
# -------------------------------------------------------------

echo "===== SOURCE CHECKPOINT ====="

BRANCH="$(git branch --show-current)"
COMMIT="$(git rev-parse HEAD)"
SHORT_COMMIT="$(git rev-parse --short HEAD)"

if [[ "$BRANCH" != "red-team" ]]; then
    echo "ERROR: Expected branch red-team."
    echo "Current branch: $BRANCH"
    exit 1
fi

if [[ "$COMMIT" != "$EXPECTED_COMMIT_PREFIX"* ]]; then
    echo "ERROR: Expected commit beginning with $EXPECTED_COMMIT_PREFIX."
    echo "Current commit: $COMMIT"
    exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "ERROR: Tracked repository files are not clean."
    git status --short
    exit 1
fi

echo "Branch: $BRANCH"
echo "Commit: $COMMIT"
echo "Tracked files: clean"

# -------------------------------------------------------------
# 2. Block deployment while either fuzz process is running.
# -------------------------------------------------------------

echo
echo "===== SPILLWAY FUZZ COMPLETION GATE ====="

FUZZ_PID=""

if [[ -f "$PID_FILE" ]]; then
    FUZZ_PID="$(tr -d '[:space:]' < "$PID_FILE")"
fi

FUZZ_PROCESSES="$(
    pgrep -af \
        'cargo.*fuzz run spillway_invariants|fuzz/.*/spillway_invariants' \
        || true
)"

if [[ -n "$FUZZ_PROCESSES" ]]; then
    echo "DEPLOYMENT BLOCKED: Spillway fuzzing is still running."
    echo
    printf '%s\n' "$FUZZ_PROCESSES"
    echo

    if [[ -n "$FUZZ_PID" ]]; then
        ps -p "$FUZZ_PID" \
            -o pid,etime,%cpu,%mem,cmd \
            || true
    fi

    echo
    echo "No deployment was performed."
    echo "Rerun this same script after the fuzz process finishes."
    exit 2
fi

echo "No active Spillway fuzz process found."

# -------------------------------------------------------------
# 3. Verify the final fuzz log.
# -------------------------------------------------------------

LATEST_LOG="$(
    find .rbvr-fuzz-runs \
        -maxdepth 1 \
        -type f \
        -name "${TARGET}-*.log" \
        -printf '%T@ %p\n' 2>/dev/null |
    sort -nr |
    head -n 1 |
    cut -d' ' -f2-
)"

if [[ -z "$LATEST_LOG" || ! -f "$LATEST_LOG" ]]; then
    echo "ERROR: No Spillway fuzz log was found."
    exit 1
fi

echo "Final fuzz log: $LATEST_LOG"

if grep -qE \
    'ERROR: libFuzzer|SUMMARY:.*Sanitizer|deadly signal|panic occurred|Assertion.*failed' \
    "$LATEST_LOG"
then
    echo "ERROR: The final fuzz log contains a failure marker."
    tail -n 120 "$LATEST_LOG"
    exit 1
fi

if ! grep -qE \
    'DONE|stat::number_of_executed_units|Done [0-9]+ runs' \
    "$LATEST_LOG"
then
    echo "ERROR: The fuzz log does not show a completed campaign."
    tail -n 80 "$LATEST_LOG"
    exit 1
fi

FAILURE_ARTIFACTS="$(
    find "fuzz/artifacts/$TARGET" \
        -maxdepth 1 \
        -type f \
        \( \
            -name 'crash-*' -o \
            -name 'timeout-*' -o \
            -name 'oom-*' \
        \) \
        -print 2>/dev/null
)"

if [[ -n "$FAILURE_ARTIFACTS" ]]; then
    echo "ERROR: Fuzz failure artifacts exist:"
    printf '%s\n' "$FAILURE_ARTIFACTS"
    exit 1
fi

echo "Final fuzz log: clean"
echo "Crash artifacts: none"
echo "Timeout artifacts: none"
echo "OOM artifacts: none"

tail -n 35 "$LATEST_LOG" |
    tee "$RAW_DIR/final-spillway-fuzz-summary.txt"

# -------------------------------------------------------------
# 4. Rebuild and rerun authoritative security gates.
# -------------------------------------------------------------

echo
echo "===== FINAL BUILD ====="
anchor build

echo
echo "===== TYPESCRIPT CHECK ====="
npx tsc --noEmit

echo
echo "===== COMPLETE RUST REGRESSION ====="

RUST_LOG="$RAW_DIR/final-rust-tests.log"
cargo test --workspace 2>&1 | tee "$RUST_LOG"

if ! grep -q 'test result: ok. 145 passed; 0 failed' "$RUST_LOG"; then
    echo "ERROR: Expected 145 passing Rust tests."
    exit 1
fi

echo
echo "===== STRICT CLIPPY ====="

cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- \
    -D warnings

echo
echo "===== COMPLETE INTEGRATION REGRESSION ====="

INTEGRATION_LOG="$RAW_DIR/final-integration-tests.log"
anchor test --validator legacy 2>&1 | tee "$INTEGRATION_LOG"

if ! grep -q '42 passing' "$INTEGRATION_LOG"; then
    echo "ERROR: Expected 42 passing integration tests."
    exit 1
fi

# -------------------------------------------------------------
# 5. Verify deployment identities and artifact.
# -------------------------------------------------------------

echo
echo "===== DEPLOYMENT IDENTITY VERIFICATION ====="

for file in \
    "$WALLET" \
    "$PROGRAM_KEYPAIR" \
    "$PROGRAM_BINARY"
do
    if [[ ! -f "$file" ]]; then
        echo "ERROR: Missing required file: $file"
        exit 1
    fi
done

WALLET_ADDRESS="$(solana-keygen pubkey "$WALLET")"
KEYPAIR_PROGRAM_ID="$(solana-keygen pubkey "$PROGRAM_KEYPAIR")"

if [[ "$KEYPAIR_PROGRAM_ID" != "$PROGRAM_ID" ]]; then
    echo "ERROR: Program keypair derives the wrong program ID."
    echo "Expected: $PROGRAM_ID"
    echo "Actual:   $KEYPAIR_PROGRAM_ID"
    exit 1
fi

DECLARE_ID="$(
    sed -nE \
        's/.*declare_id!\("([^"]+)"\).*/\1/p' \
        programs/treasury-router/src/lib.rs |
    head -n 1
)"

if [[ "$DECLARE_ID" != "$PROGRAM_ID" ]]; then
    echo "ERROR: declare_id! does not match the deployment ID."
    exit 1
fi

PROGRAM_SIZE="$(stat -c '%s' "$PROGRAM_BINARY")"
PROGRAM_SHA256="$(sha256sum "$PROGRAM_BINARY" | awk '{print $1}')"

echo "Deployment wallet: $WALLET_ADDRESS"
echo "Program ID:        $PROGRAM_ID"
echo "Binary size:       $PROGRAM_SIZE bytes"
echo "Binary SHA-256:    $PROGRAM_SHA256"

# -------------------------------------------------------------
# 6. Ensure the program is not already deployed.
# -------------------------------------------------------------

echo
echo "===== EXISTING DEVNET PROGRAM CHECK ====="

if solana program show "$PROGRAM_ID" \
    --url "$DEVNET_URL" \
    >"$RAW_DIR/pre-deployment-program-show.txt" 2>&1
then
    echo "ERROR: The program is already deployed on Devnet."
    echo
    cat "$RAW_DIR/pre-deployment-program-show.txt"
    echo
    echo "This script intentionally refuses to perform an upgrade."
    exit 1
fi

echo "Program is not currently deployed on Devnet."

# -------------------------------------------------------------
# 7. Check deployment wallet balance.
# -------------------------------------------------------------

echo
echo "===== DEVNET BALANCE ====="

BALANCE="$(
    solana balance "$WALLET_ADDRESS" \
        --url "$DEVNET_URL" |
    awk '{print $1}'
)"

echo "Current balance: $BALANCE SOL"

if ! awk \
    "BEGIN { exit !($BALANCE >= $MINIMUM_BALANCE) }"
then
    echo
    echo "DEPLOYMENT BLOCKED: More Devnet SOL is recommended."
    echo "Required gate: $MINIMUM_BALANCE SOL"
    echo "Current:       $BALANCE SOL"
    echo
    echo "Request Devnet SOL, then rerun this script:"
    echo
    echo "solana airdrop 2 $WALLET_ADDRESS --url $DEVNET_URL"
    echo
    echo "The exact same binary remains ready for deployment."
    exit 3
fi

# -------------------------------------------------------------
# 8. Record the pre-deployment binary permanently.
# -------------------------------------------------------------

echo
echo "===== RECORD DEPLOYMENT ARTIFACT ====="

cp "$PROGRAM_BINARY" \
    "$RAW_DIR/treasury_router-${SHORT_COMMIT}.so"

printf '%s\n' "$PROGRAM_SHA256" \
    >"$RAW_DIR/treasury_router-${SHORT_COMMIT}.sha256"

git diff --quiet
git diff --cached --quiet

# -------------------------------------------------------------
# 9. Perform the first Devnet deployment.
# -------------------------------------------------------------

echo
echo "============================================================"
echo "DEPLOYING RBVR TO SOLANA DEVNET"
echo "============================================================"

DEPLOY_LOG="$RAW_DIR/devnet-deploy.log"

solana program deploy \
    "$PROGRAM_BINARY" \
    --program-id "$PROGRAM_KEYPAIR" \
    --keypair "$WALLET" \
    --url "$DEVNET_URL" \
    --commitment confirmed \
    2>&1 |
tee "$DEPLOY_LOG"

# -------------------------------------------------------------
# 10. Verify the on-chain deployment.
# -------------------------------------------------------------

echo
echo "===== VERIFY ON-CHAIN PROGRAM ====="

PROGRAM_SHOW="$RAW_DIR/post-deployment-program-show.txt"

solana program show "$PROGRAM_ID" \
    --url "$DEVNET_URL" \
    >"$PROGRAM_SHOW"

cat "$PROGRAM_SHOW"

ONCHAIN_PROGRAM_ID="$(
    sed -n 's/^Program Id: //p' "$PROGRAM_SHOW" |
    tr -d '[:space:]'
)"

UPGRADE_AUTHORITY="$(
    sed -n 's/^Authority: //p' "$PROGRAM_SHOW" |
    tr -d '[:space:]'
)"

if [[ "$ONCHAIN_PROGRAM_ID" != "$PROGRAM_ID" ]]; then
    echo "ERROR: On-chain program ID verification failed."
    exit 1
fi

if [[ "$UPGRADE_AUTHORITY" != "$WALLET_ADDRESS" ]]; then
    echo "ERROR: Unexpected Devnet upgrade authority."
    echo "Expected: $WALLET_ADDRESS"
    echo "Actual:   $UPGRADE_AUTHORITY"
    exit 1
fi

echo
echo "On-chain program ID: verified"
echo "Devnet upgrade authority: verified"
echo "Upgrade authority has NOT been revoked."

POST_BALANCE="$(
    solana balance "$WALLET_ADDRESS" \
        --url "$DEVNET_URL" |
    awk '{print $1}'
)"

# -------------------------------------------------------------
# 11. Write deployment report.
# -------------------------------------------------------------

cat > "$REPORT" <<EOF
# RBVR Devnet Deployment Result

Generated: **$(date -u '+%Y-%m-%d %H:%M:%S UTC')**

## Source

- Branch: \`$BRANCH\`
- Commit: \`$COMMIT\`
- Program ID: \`$PROGRAM_ID\`

## Verified security gates

- Rust tests: **145 passing**
- Integration tests: **42 passing**
- Strict Clippy: **passing**
- Spillway fuzz campaign: **completed without recorded failure artifacts**
- Devnet preflight: **passed**

## Deployment artifact

- Binary: \`$PROGRAM_BINARY\`
- Size: **$PROGRAM_SIZE bytes**
- SHA-256: \`$PROGRAM_SHA256\`

## Deployment

- Cluster: **Solana Devnet**
- Program ID: \`$PROGRAM_ID\`
- Deployment wallet: \`$WALLET_ADDRESS\`
- Wallet balance before: **$BALANCE SOL**
- Wallet balance after: **$POST_BALANCE SOL**
- Upgrade authority: \`$UPGRADE_AUTHORITY\`

## Authority status

The Devnet program remains upgradeable for testing.

**No authority was revoked by this deployment script.**
EOF

echo
echo "============================================================"
echo "RBVR DEVNET DEPLOYMENT PASSED"
echo "============================================================"
echo "Program ID:          $PROGRAM_ID"
echo "Commit:              $SHORT_COMMIT"
echo "Binary SHA-256:      $PROGRAM_SHA256"
echo "Upgrade authority:   $UPGRADE_AUTHORITY"
echo "Authority revoked:   no"
echo "Report:              $REPORT"
echo
echo "Next:"
echo "Initialize the Devnet protocol accounts and run the complete"
echo "fees → releases → Reserve Vault → Spillway transaction cycle."
echo "============================================================"
