#!/bin/bash

set -e

EXPECTED_CLUSTER="https://api.mainnet-beta.solana.com"
PROGRAM_BINARY="target/deploy/treasury_router.so"
EXPECTED_HASH="5173bc909c97b3b5c525c5ccd1bc8f51621d1d5f18ab8a5cf3000af9cc1cccb6"

echo "================================"
echo "RBVR Mainnet Environment Check"
echo "================================"

RPC=$(solana config get | grep "RPC URL" | awk '{print $3}')

echo
echo "Current RPC:"
echo "$RPC"

if [ "$RPC" != "$EXPECTED_CLUSTER" ]; then
    echo "ERROR: Not connected to Mainnet"
    exit 1
fi

echo "PASS: Mainnet RPC confirmed"

echo
echo "Wallet:"
solana address

echo
echo "Balance:"
solana balance

echo
echo "Checking binary..."

if [ ! -f "$PROGRAM_BINARY" ]; then
    echo "ERROR: Binary missing"
    exit 1
fi

HASH=$(sha256sum "$PROGRAM_BINARY" | awk '{print $1}')

echo "Binary hash:"
echo "$HASH"

if [ "$HASH" != "$EXPECTED_HASH" ]; then
    echo "ERROR: Binary mismatch"
    exit 1
fi

echo
echo "PASS: Environment ready for deployment"
