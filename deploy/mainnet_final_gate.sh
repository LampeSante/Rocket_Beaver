#!/bin/bash

set -e

EXPECTED_RPC="https://api.mainnet-beta.solana.com"
PROGRAM_BINARY="target/deploy/treasury_router.so"
EXPECTED_HASH="5173bc909c97b3b5c525c5ccd1bc8f51621d1d5f18ab8a5cf3000af9cc1cccb6"

echo "================================"
echo "RBVR MAINNET FINAL GATE"
echo "================================"

echo
echo "Checking RPC..."

RPC=$(solana config get | grep "RPC URL" | awk '{print $3}')

if [ "$RPC" != "$EXPECTED_RPC" ]; then
    echo "FAIL: Wrong RPC"
    echo "Current:"
    echo "$RPC"
    exit 1
fi

echo "PASS: Mainnet RPC"


echo
echo "Checking wallet..."

solana address

echo
echo "Balance:"
solana balance


echo
echo "Checking binary..."

if [ ! -f "$PROGRAM_BINARY" ]; then
    echo "FAIL: Missing program binary"
    exit 1
fi

HASH=$(sha256sum "$PROGRAM_BINARY" | awk '{print $1}')

if [ "$HASH" != "$EXPECTED_HASH" ]; then
    echo "FAIL: Binary hash mismatch"
    exit 1
fi

echo "PASS: Binary hash"


echo
echo "================================"
echo "FINAL GATE PASSED"
echo "================================"

echo
echo "Ready for:"
echo
echo "solana program deploy $PROGRAM_BINARY"
