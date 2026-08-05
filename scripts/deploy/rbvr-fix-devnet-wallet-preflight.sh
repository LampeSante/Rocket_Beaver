#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

PREFLIGHT="rbvr-devnet-preflight-current.sh"
CONFIGURED_WALLET="/home/alec_elliott/rocket-beaver-dev/keys/local-deployment.json"
DEVNET_URL="https://api.devnet.solana.com"

echo "============================================================"
echo "RBVR DEVNET WALLET PREFLIGHT FIX"
echo "============================================================"

if [[ ! -f "$PREFLIGHT" ]]; then
    echo "ERROR: Missing preflight script: $PREFLIGHT"
    exit 1
fi

echo
echo "===== VERIFY CONFIGURED WALLET ====="

if [[ ! -f "$CONFIGURED_WALLET" ]]; then
    echo "ERROR: Configured wallet does not exist:"
    echo "$CONFIGURED_WALLET"
    exit 1
fi

chmod 600 "$CONFIGURED_WALLET"

WALLET_ADDRESS="$(solana-keygen pubkey "$CONFIGURED_WALLET")"

echo "Wallet file:    $CONFIGURED_WALLET"
echo "Wallet address: $WALLET_ADDRESS"

if [[ -z "$WALLET_ADDRESS" ]]; then
    echo "ERROR: Could not derive wallet address."
    exit 1
fi

echo
echo "===== VERIFY DEVNET BALANCE ====="

solana balance "$WALLET_ADDRESS" --url "$DEVNET_URL"

echo
echo "===== PATCH PREFLIGHT WALLET RESOLUTION ====="

python3 <<'PY'
from pathlib import Path

path = Path("rbvr-devnet-preflight-current.sh")
text = path.read_text()

start_marker = '''echo
echo "===== DEVNET WALLET ====="
'''

end_marker = '''echo
echo "===== CURRENT DEVNET DEPLOYMENT ====="
'''

start = text.find(start_marker)
if start == -1:
    raise RuntimeError("Could not locate DEVNET WALLET section")

end = text.find(end_marker, start)
if end == -1:
    raise RuntimeError("Could not locate deployment-status section")

replacement = r'''echo
echo "===== DEVNET WALLET ====="

CONFIG_OUTPUT="$(solana config get 2>&1)"

WALLET_PATH="$(
    printf '%s\n' "$CONFIG_OUTPUT" |
    awk -F': ' '
        $1 ~ /^[[:space:]]*Keypair Path$/ {
            print $2
            exit
        }
    '
)"

# Use the known configured RBVR development wallet if CLI output parsing
# changes between Solana versions.
if [[ -z "$WALLET_PATH" ]]; then
    WALLET_PATH="/home/alec_elliott/rocket-beaver-dev/keys/local-deployment.json"
fi

# Expand a leading tilde if one is present in the Solana configuration.
if [[ "$WALLET_PATH" == "~/"* ]]; then
    WALLET_PATH="$HOME/${WALLET_PATH#~/}"
fi

if [[ -f "$WALLET_PATH" ]]; then
    WALLET_ADDRESS="$(solana-keygen pubkey "$WALLET_PATH")"

    if [[ -n "$WALLET_ADDRESS" ]]; then
        pass "Configured wallet exists: $WALLET_ADDRESS."
        echo "Wallet path: $WALLET_PATH"
    else
        fail "Could not derive the configured wallet address."
        WALLET_ADDRESS="UNKNOWN"
    fi
else
    fail "Configured wallet file does not exist: $WALLET_PATH"
    WALLET_ADDRESS="UNKNOWN"
fi

if solana cluster-version --url "$DEVNET_URL" \
    >"$RAW_DIR/devnet-cluster-version.txt" 2>&1
then
    pass "Devnet RPC is reachable."
else
    fail "Devnet RPC is not reachable."
fi

if [[ "$WALLET_ADDRESS" != "UNKNOWN" ]]; then
    BALANCE_RAW="$(
        solana balance "$WALLET_ADDRESS" \
            --url "$DEVNET_URL" \
            2>/dev/null |
        awk '{print $1}'
    )"
else
    BALANCE_RAW=""
fi

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

'''

path.write_text(text[:start] + replacement + text[end:])

print("Devnet wallet resolution updated successfully.")
PY

echo
echo "===== VERIFY PATCH ====="

grep -n -A 70 \
    '===== DEVNET WALLET =====' \
    "$PREFLIGHT" |
head -n 75

echo
echo "===== RERUN DEVNET PREFLIGHT ====="

chmod +x "$PREFLIGHT"
./"$PREFLIGHT"
