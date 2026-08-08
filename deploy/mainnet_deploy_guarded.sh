#!/bin/bash

set -e

PROGRAM_BINARY="target/deploy/treasury_router.so"
EXPECTED_HASH="5173bc909c97b3b5c525c5ccd1bc8f51621d1d5f18ab8a5cf3000af9cc1cccb6"
EXPECTED_CLUSTER="https://api.mainnet-beta.solana.com"

echo "================================"
echo "RBVR Guarded Mainnet Deployment"
echo "================================"

RPC=$(solana config get | grep "RPC URL" | awk '{print $3}')

if [ "$RPC" != "$EXPECTED_CLUSTER" ]; then
    echo "ERROR: RPC is not Mainnet"
    exit 1
fi

echo "PASS: Mainnet RPC"

if [ ! -f "$PROGRAM_BINARY" ]; then
    echo "ERROR: Program binary missing"
    exit 1
fi

HASH=$(sha256sum "$PROGRAM_BINARY" | awk '{print $1}')

if [ "$HASH" != "$EXPECTED_HASH" ]; then
    echo "ERROR: Binary hash mismatch"
    exit 1
fi

echo "PASS: Binary verified"

echo
echo "Wallet:"
solana address

echo
read -p "Type DEPLOY-RBVR to continue: " CONFIRM

if [ "$CONFIRM" != "DEPLOY-RBVR" ]; then
    echo "Deployment cancelled"
    exit 1
fi

echo
echo "Ready."
echo "Deployment command must be executed manually."
