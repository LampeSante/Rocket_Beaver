#!/bin/bash

set -e

PROGRAM_BINARY="target/deploy/treasury_router.so"
EXPECTED_HASH="5173bc909c97b3b5c525c5ccd1bc8f51621d1d5f18ab8a5cf3000af9cc1cccb6"
EXPECTED_RPC="https://api.mainnet-beta.solana.com"

echo "================================"
echo "RBVR MAINNET DEPLOYMENT EXECUTOR"
echo "================================"

echo
echo "Checking RPC..."

RPC=$(solana config get | grep "RPC URL" | awk '{print $3}')

if [ "$RPC" != "$EXPECTED_RPC" ]; then
    echo "ERROR: Not connected to Solana Mainnet"
    echo "Current RPC: $RPC"
    exit 1
fi

echo "PASS: Mainnet RPC"


echo
echo "Checking wallet..."

WALLET=$(solana address)

echo "Wallet:"
echo "$WALLET"


echo
echo "Checking balance..."

solana balance


echo
echo "Checking binary..."

if [ ! -f "$PROGRAM_BINARY" ]; then
    echo "ERROR: Program binary missing"
    exit 1
fi

ls -lh "$PROGRAM_BINARY"


echo
echo "Checking SHA256..."

ACTUAL_HASH=$(sha256sum "$PROGRAM_BINARY" | awk '{print $1}')

echo "Expected:"
echo "$EXPECTED_HASH"

echo "Actual:"
echo "$ACTUAL_HASH"

if [ "$ACTUAL_HASH" != "$EXPECTED_HASH" ]; then
    echo "ERROR: Binary hash mismatch"
    exit 1
fi

echo "PASS: Binary verified"


echo
echo "================================"
echo "READY FOR DEPLOYMENT"
echo "================================"

echo
echo "Review:"
echo "- RPC confirmed"
echo "- Wallet confirmed"
echo "- Binary confirmed"
echo
echo "To deploy manually run:"
echo
echo "solana program deploy $PROGRAM_BINARY"
echo

read -p "Type DEPLOY to continue: " CONFIRM

if [ "$CONFIRM" != "DEPLOY" ]; then
    echo "Deployment cancelled"
    exit 0
fi


echo
echo "Deploying..."

solana program deploy "$PROGRAM_BINARY"

echo
echo "Deployment complete."
