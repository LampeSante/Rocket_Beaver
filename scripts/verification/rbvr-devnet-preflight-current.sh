#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

PROGRAM_ID="5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3"
DEVNET_URL="https://api.devnet.solana.com"
EXPECTED_COMMIT_PREFIX="a89e8b7"
REPORT="RBVR-DEVNET-PREFLIGHT-CURRENT.md"
RAW_DIR=".rbvr-devnet-preflight"

PASS=0
WARN=0
FAIL=0

mkdir -p "$RAW_DIR"

pass() {
    echo "✅ PASS — $1"
    PASS=$((PASS + 1))
}

warn() {
    echo "⚠️ WARN — $1"
    WARN=$((WARN + 1))
}

fail() {
    echo "❌ FAIL — $1"
    FAIL=$((FAIL + 1))
}

echo "============================================================"
echo "RBVR DEVNET READINESS PREFLIGHT"
echo "============================================================"
echo "Program: $PROGRAM_ID"
echo "RPC:     $DEVNET_URL"
echo

BRANCH="$(git branch --show-current)"
COMMIT="$(git rev-parse HEAD)"
SHORT_COMMIT="$(git rev-parse --short HEAD)"

if [[ "$BRANCH" == "red-team" ]]; then
    pass "Current branch is red-team."
else
    fail "Expected branch red-team; found $BRANCH."
fi

if [[ "$COMMIT" == "$EXPECTED_COMMIT_PREFIX"* ]]; then
    pass "Current commit matches the Spillway fuzz checkpoint: $SHORT_COMMIT."
else
    fail "Expected commit beginning with $EXPECTED_COMMIT_PREFIX; found $SHORT_COMMIT."
fi

if git diff --quiet && git diff --cached --quiet; then
    pass "Tracked repository files are clean."
else
    fail "Tracked repository files contain uncommitted changes."
fi

echo
echo "===== PROGRAM ID CONSISTENCY ====="

ANCHOR_PROGRAM_ID="$(
    sed -nE \
      's/^[[:space:]]*treasury_router[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' \
      Anchor.toml |
    head -n 1
)"

DECLARE_ID="$(
    sed -nE \
      's/.*declare_id!\("([^"]+)"\).*/\1/p' \
      programs/treasury-router/src/lib.rs |
    head -n 1
)"

KEYPAIR_FILE="target/deploy/treasury_router-keypair.json"

if [[ "$ANCHOR_PROGRAM_ID" == "$PROGRAM_ID" ]]; then
    pass "Anchor.toml program ID matches."
else
    fail "Anchor.toml program ID is '$ANCHOR_PROGRAM_ID'."
fi

if [[ "$DECLARE_ID" == "$PROGRAM_ID" ]]; then
    pass "declare_id! matches."
else
    fail "declare_id! is '$DECLARE_ID'."
fi

if [[ -f "$KEYPAIR_FILE" ]]; then
    KEYPAIR_PROGRAM_ID="$(solana-keygen pubkey "$KEYPAIR_FILE")"

    if [[ "$KEYPAIR_PROGRAM_ID" == "$PROGRAM_ID" ]]; then
        pass "Deployment keypair derives the expected program ID."
    else
        fail "Deployment keypair derives $KEYPAIR_PROGRAM_ID."
    fi
else
    fail "Missing deployment keypair: $KEYPAIR_FILE."
fi

echo
echo "===== BUILD ARTIFACT ====="

anchor build

PROGRAM_SO="target/deploy/treasury_router.so"

if [[ -s "$PROGRAM_SO" ]]; then
    PROGRAM_SIZE="$(stat -c '%s' "$PROGRAM_SO")"
    PROGRAM_SHA256="$(sha256sum "$PROGRAM_SO" | awk '{print $1}')"

    pass "Program binary exists and is non-empty."
    echo "Binary size:   $PROGRAM_SIZE bytes"
    echo "Binary SHA256: $PROGRAM_SHA256"
else
    fail "Program binary is missing or empty."
    PROGRAM_SIZE="UNKNOWN"
    PROGRAM_SHA256="UNKNOWN"
fi

echo
echo "===== STANDARD SECURITY GATES ====="

RUST_LOG="$RAW_DIR/rust-tests.log"
cargo test --workspace 2>&1 | tee "$RUST_LOG"

if grep -q 'test result: ok. 145 passed; 0 failed' "$RUST_LOG"; then
    pass "All 145 Rust tests pass."
else
    fail "Rust regression did not report 145 passing tests."
fi

cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- \
    -D warnings

pass "Strict Clippy passes."

echo
echo "===== DEVNET WALLET ====="

WALLET_PATH="/home/alec_elliott/rocket-beaver-dev/keys/local-deployment.json"

if [[ ! -f "$WALLET_PATH" ]]; then
    fail "Configured wallet file does not exist: $WALLET_PATH"
    WALLET_ADDRESS="UNKNOWN"
    BALANCE_RAW="UNKNOWN"
else
    WALLET_ADDRESS="$(solana-keygen pubkey "$WALLET_PATH")"

    if [[ -z "$WALLET_ADDRESS" ]]; then
        fail "Could not derive wallet address from: $WALLET_PATH"
        WALLET_ADDRESS="UNKNOWN"
        BALANCE_RAW="UNKNOWN"
    else
        pass "Configured wallet exists: $WALLET_ADDRESS."
        echo "Wallet path: $WALLET_PATH"

        if solana cluster-version --url "$DEVNET_URL" \
            >"$RAW_DIR/devnet-cluster-version.txt" 2>&1
        then
            pass "Devnet RPC is reachable."
        else
            fail "Devnet RPC is not reachable."
        fi

        BALANCE_RAW="$(
            solana balance "$WALLET_ADDRESS" \
                --url "$DEVNET_URL" \
                2>/dev/null |
            awk '{print $1}'
        )"

        if [[ -n "$BALANCE_RAW" ]]; then
            echo "Devnet balance: $BALANCE_RAW SOL"

            if awk "BEGIN { exit !($BALANCE_RAW >= 2.0) }"; then
                pass "Wallet has at least 2 devnet SOL."
            else
                warn "Wallet balance may be low for deployment."
            fi
        else
            fail "Could not read devnet wallet balance."
            BALANCE_RAW="UNKNOWN"
        fi
    fi
fi

echo
echo "===== CURRENT DEVNET DEPLOYMENT ====="

PROGRAM_SHOW="$RAW_DIR/program-show.txt"

if solana program show "$PROGRAM_ID" \
    --url "$DEVNET_URL" \
    >"$PROGRAM_SHOW" 2>&1
then
    warn "Program ID is already deployed on devnet."

    grep -E \
      'Program Id|Owner|ProgramData Address|Authority|Last Deployed In Slot|Data Length' \
      "$PROGRAM_SHOW" || true
else
    pass "Program ID is not currently deployed on devnet."
fi

echo
echo "===== SPILLWAY FUZZ STATUS ====="

FUZZ_PID_FILE=".rbvr-fuzz-runs/spillway_invariants.pid"
FUZZ_STATUS="UNKNOWN"
FUZZ_PID="UNKNOWN"

if [[ -f "$FUZZ_PID_FILE" ]]; then
    FUZZ_PID="$(cat "$FUZZ_PID_FILE")"

    if kill -0 "$FUZZ_PID" 2>/dev/null; then
        FUZZ_STATUS="RUNNING"
        pass "Spillway invariant fuzzer is still running."

        ps -p "$FUZZ_PID" \
          -o pid,etime,%cpu,%mem,cmd \
          >"$RAW_DIR/spillway-fuzz-process.txt"

        cat "$RAW_DIR/spillway-fuzz-process.txt"
    else
        FUZZ_STATUS="FINISHED"
        warn "Spillway fuzz launcher is no longer running."
    fi
else
    warn "Spillway fuzz PID file was not found."
fi

ARTIFACT_COUNT="$(
    find fuzz/artifacts/spillway_invariants \
      -maxdepth 1 \
      -type f \
      \( \
        -name 'crash-*' -o \
        -name 'timeout-*' -o \
        -name 'oom-*' \
      \) 2>/dev/null |
    wc -l
)"

if [[ "$ARTIFACT_COUNT" -eq 0 ]]; then
    pass "No Spillway crash, timeout, or OOM artifacts exist."
else
    fail "Spillway fuzzing has produced $ARTIFACT_COUNT failure artifact(s)."

    find fuzz/artifacts/spillway_invariants \
      -maxdepth 1 \
      -type f \
      -print
fi

echo
echo "===== WRITE REPORT ====="

cat > "$REPORT" <<EOF
# RBVR Devnet Readiness Preflight

Generated: **$(date -u '+%Y-%m-%d %H:%M:%S UTC')**

## Source checkpoint

- Branch: \`$BRANCH\`
- Commit: \`$COMMIT\`
- Program ID: \`$PROGRAM_ID\`
- Expected checkpoint: \`$EXPECTED_COMMIT_PREFIX\`
- Spillway fuzz status: **$FUZZ_STATUS**
- Spillway fuzz PID: \`$FUZZ_PID\`

## Build artifact

- File: \`$PROGRAM_SO\`
- Size: **$PROGRAM_SIZE bytes**
- SHA-256: \`$PROGRAM_SHA256\`

## Devnet wallet

- Wallet: \`$WALLET_ADDRESS\`
- Balance: **$BALANCE_RAW SOL**

## Verification summary

| Result | Count |
|---|---:|
| Pass | $PASS |
| Warning | $WARN |
| Failure | $FAIL |

## Deployment decision

EOF

if [[ "$FAIL" -gt 0 ]]; then
    cat >> "$REPORT" <<EOF
**BLOCKED**

One or more required preflight checks failed. Do not deploy this build.
EOF
elif [[ "$FUZZ_STATUS" == "RUNNING" ]]; then
    cat >> "$REPORT" <<EOF
**TECHNICALLY READY, FUZZ CAMPAIGN STILL RUNNING**

The build and devnet environment passed the current checks. Deployment remains
intentionally blocked until the eight-hour Spillway fuzz campaign completes
without crash, timeout, OOM, or invariant-failure artifacts.
EOF
else
    cat >> "$REPORT" <<EOF
**READY FOR FINAL FUZZ REVIEW**

The build and devnet environment passed. Review the completed Spillway fuzz log
and artifacts before deployment.
EOF
fi

echo
echo "============================================================"
echo "DEVNET PREFLIGHT COMPLETE"
echo "============================================================"
echo "Pass:      $PASS"
echo "Warnings:  $WARN"
echo "Failures:  $FAIL"
echo "Report:    $REPORT"
echo

if [[ "$FAIL" -gt 0 ]]; then
    echo "Decision: BLOCKED"
    exit 1
elif [[ "$FUZZ_STATUS" == "RUNNING" ]]; then
    echo "Decision: READY, WAITING FOR FUZZ COMPLETION"
else
    echo "Decision: READY FOR FINAL FUZZ REVIEW"
fi

echo "============================================================"
