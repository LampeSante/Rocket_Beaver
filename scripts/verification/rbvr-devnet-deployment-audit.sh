#!/usr/bin/env bash
set -uo pipefail

PROGRAM_ID="5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3"
DEVNET_URL="https://api.devnet.solana.com"
REPORT="RBVR-DEVNET-DEPLOYMENT-AUDIT.md"
RAW_DIR=".rbvr-devnet-audit"
TIMESTAMP="$(date -u '+%Y-%m-%d %H:%M:%S UTC')"

mkdir -p "$RAW_DIR"

PASS=0
WARN=0
FAIL=0

pass() {
    printf '✅ PASS — %s\n' "$1"
    PASS=$((PASS + 1))
}

warn() {
    printf '⚠️ WARN — %s\n' "$1"
    WARN=$((WARN + 1))
}

fail() {
    printf '❌ FAIL — %s\n' "$1"
    FAIL=$((FAIL + 1))
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

echo "============================================================"
echo "RBVR DEVNET DEPLOYMENT AUDIT"
echo "============================================================"
echo "Program: $PROGRAM_ID"
echo "RPC:     $DEVNET_URL"
echo

{
    echo "# RBVR Devnet Deployment Audit"
    echo
    echo "Generated: **$TIMESTAMP**"
    echo
    echo "- Program ID: \`$PROGRAM_ID\`"
    echo "- Cluster: **Solana devnet**"
    echo "- RPC: \`$DEVNET_URL\`"
    echo
} > "$REPORT"

if ! command_exists solana; then
    fail "Solana CLI is not installed or not on PATH."
    {
        echo "## Result"
        echo
        echo "Solana CLI was not available."
    } >> "$REPORT"
    exit 1
fi

SOLANA_VERSION="$(solana --version 2>&1 || true)"
CLI_CONFIG="$(solana config get 2>&1 || true)"
WALLET_ADDRESS="$(solana address 2>/dev/null || true)"
DEVNET_BALANCE="$(solana balance --url "$DEVNET_URL" 2>&1 || true)"
GENESIS_HASH="$(solana genesis-hash --url "$DEVNET_URL" 2>&1 || true)"

{
    echo "## Local environment"
    echo
    echo '```text'
    echo "$SOLANA_VERSION"
    echo
    echo "$CLI_CONFIG"
    echo
    echo "Active wallet: ${WALLET_ADDRESS:-unavailable}"
    echo "Devnet balance: $DEVNET_BALANCE"
    echo "Devnet genesis hash: $GENESIS_HASH"
    echo '```'
    echo
} >> "$REPORT"

if [[ -n "$GENESIS_HASH" ]] && [[ "$GENESIS_HASH" != *"error"* ]] && [[ "$GENESIS_HASH" != *"Error"* ]]; then
    pass "Devnet RPC is reachable."
else
    fail "Devnet RPC could not be reached."
fi

PROGRAM_SHOW_FILE="$RAW_DIR/program-show.txt"

if solana program show "$PROGRAM_ID" \
    --url "$DEVNET_URL" \
    >"$PROGRAM_SHOW_FILE" 2>&1
then
    PROGRAM_EXISTS=1
    pass "Program account exists on devnet."
else
    PROGRAM_EXISTS=0
    warn "Program ID is not currently deployed on devnet."
fi

{
    echo "## Program information"
    echo
    echo '```text'
    cat "$PROGRAM_SHOW_FILE"
    echo '```'
    echo
} >> "$REPORT"

if [[ "$PROGRAM_EXISTS" -eq 1 ]]; then
    PROGRAM_OUTPUT="$(cat "$PROGRAM_SHOW_FILE")"

    PROGRAMDATA_ADDRESS="$(
        printf '%s\n' "$PROGRAM_OUTPUT" |
        sed -nE 's/^[[:space:]]*ProgramData Address:[[:space:]]*//p' |
        head -n 1
    )"

    UPGRADE_AUTHORITY="$(
        printf '%s\n' "$PROGRAM_OUTPUT" |
        sed -nE 's/^[[:space:]]*Authority:[[:space:]]*//p' |
        head -n 1
    )"

    PROGRAM_OWNER="$(
        solana account "$PROGRAM_ID" \
            --url "$DEVNET_URL" 2>/dev/null |
        sed -nE 's/^[[:space:]]*Owner:[[:space:]]*//p' |
        head -n 1
    )"

    PROGRAM_EXECUTABLE="$(
        solana account "$PROGRAM_ID" \
            --url "$DEVNET_URL" 2>/dev/null |
        sed -nE 's/^[[:space:]]*Executable:[[:space:]]*//p' |
        head -n 1
    )"

    {
        echo "## Parsed deployment properties"
        echo
        echo "| Property | Value |"
        echo "|---|---|"
        echo "| ProgramData | \`${PROGRAMDATA_ADDRESS:-not parsed}\` |"
        echo "| Upgrade authority | \`${UPGRADE_AUTHORITY:-not parsed}\` |"
        echo "| Program owner | \`${PROGRAM_OWNER:-not parsed}\` |"
        echo "| Executable | \`${PROGRAM_EXECUTABLE:-not parsed}\` |"
        echo
    } >> "$REPORT"

    if [[ -n "$PROGRAMDATA_ADDRESS" ]]; then
        pass "ProgramData address was identified."

        solana account "$PROGRAMDATA_ADDRESS" \
            --url "$DEVNET_URL" \
            >"$RAW_DIR/programdata-account.txt" 2>&1 || true

        {
            echo "## ProgramData account"
            echo
            echo '```text'
            cat "$RAW_DIR/programdata-account.txt"
            echo '```'
            echo
        } >> "$REPORT"
    else
        warn "ProgramData address could not be parsed."
    fi

    if [[ "$PROGRAM_EXECUTABLE" == "true" ]]; then
        pass "Program account is executable."
    else
        fail "Program account was not confirmed executable."
    fi

    if [[ "$UPGRADE_AUTHORITY" == "none" ]] ||
       [[ "$UPGRADE_AUTHORITY" == "None" ]] ||
       [[ "$UPGRADE_AUTHORITY" == "null" ]]; then
        pass "Upgrade authority is revoked."
        UPGRADE_STATUS="REVOKED"
    elif [[ -n "$UPGRADE_AUTHORITY" ]]; then
        warn "Program remains upgradeable by $UPGRADE_AUTHORITY."
        UPGRADE_STATUS="ACTIVE"
    else
        warn "Upgrade authority could not be parsed."
        UPGRADE_STATUS="UNKNOWN"
    fi

    LOCAL_SO="target/deploy/treasury_router.so"

    if [[ -f "$LOCAL_SO" ]]; then
        LOCAL_SHA256="$(sha256sum "$LOCAL_SO" | awk '{print $1}')"
        LOCAL_SIZE="$(stat -c '%s' "$LOCAL_SO")"

        {
            echo "## Local build artifact"
            echo
            echo "- File: \`$LOCAL_SO\`"
            echo "- Size: **$LOCAL_SIZE bytes**"
            echo "- SHA-256: \`$LOCAL_SHA256\`"
            echo
            echo "> This hash describes the local binary only. It is not yet proof"
            echo "> that the deployed bytecode is identical."
            echo
        } >> "$REPORT"

        pass "Local treasury-router binary exists."
    else
        warn "No local target/deploy/treasury_router.so binary was found."
    fi
else
    UPGRADE_STATUS="NOT_DEPLOYED"

    {
        echo "## Deployment conclusion"
        echo
        echo "The configured RBVR program ID was not found on devnet."
        echo
        echo "No upgrade-authority, ProgramData, or on-chain state verification"
        echo "can be completed until a devnet deployment exists."
        echo
    } >> "$REPORT"
fi

FUZZ_PID_FILE=".rbvr-fuzz-runs/process_release.pid"

if [[ -f "$FUZZ_PID_FILE" ]]; then
    FUZZ_PID="$(cat "$FUZZ_PID_FILE" 2>/dev/null || true)"

    if [[ -n "$FUZZ_PID" ]] && kill -0 "$FUZZ_PID" 2>/dev/null; then
        FUZZ_STATUS="$(ps -p "$FUZZ_PID" -o pid=,etime=,%cpu=,%mem=,cmd=)"
        pass "Background process_release fuzzer is still running."

        {
            echo "## Concurrent fuzzing"
            echo
            echo '```text'
            echo "$FUZZ_STATUS"
            echo '```'
            echo
        } >> "$REPORT"
    else
        warn "The recorded process_release fuzz PID is no longer active."
    fi
else
    warn "No process_release fuzz PID file was found."
fi

{
    echo "## Audit summary"
    echo
    echo "| Result | Count |"
    echo "|---|---:|"
    echo "| Pass | $PASS |"
    echo "| Warning | $WARN |"
    echo "| Failure | $FAIL |"
    echo
    echo "- Deployment status: **$([[ "$PROGRAM_EXISTS" -eq 1 ]] && echo DEPLOYED || echo NOT DEPLOYED)**"
    echo "- Upgrade-authority status: **$UPGRADE_STATUS**"
    echo
    echo "## Safety note"
    echo
    echo "This audit performed read-only RPC and local filesystem operations."
    echo "It did not deploy, upgrade, initialize, transfer, or revoke authority."
} >> "$REPORT"

echo
echo "============================================================"
echo "AUDIT COMPLETE"
echo "============================================================"
echo "Pass:     $PASS"
echo "Warnings: $WARN"
echo "Failures: $FAIL"
echo "Report:   $REPORT"
echo
echo "Review with:"
echo "cat $REPORT"
echo "============================================================"

if [[ "$FAIL" -gt 0 ]]; then
    exit 1
fi
