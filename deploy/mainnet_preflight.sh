#!/bin/bash

set -e

PROGRAM_BINARY="target/deploy/treasury_router.so"
EXPECTED_HASH="5173bc909c97b3b5c525c5ccd1bc8f51621d1d5f18ab8a5cf3000af9cc1cccb6"

echo "================================"
echo "RBVR Mainnet Preflight Check"
echo "================================"

echo
echo "Solana Configuration:"
solana config get

echo
echo "Wallet:"
solana address

echo
echo "Balance:"
solana balance

echo
echo "Checking binary..."

if [ ! -f "$PROGRAM_BINARY" ]; then
    echo "ERROR: Program binary missing"
    exit 1
fi

echo "Binary found:"
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

echo
echo "PASS: Binary verified"
echo
echo "Preflight complete"
