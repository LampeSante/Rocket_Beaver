#!/bin/bash

set -e

PROGRAM_ID="TBD"
RPC="https://api.mainnet-beta.solana.com"

echo "================================"
echo "RBVR MAINNET POST DEPLOY VERIFY"
echo "================================"

if [ "$PROGRAM_ID" = "TBD" ]; then
    echo "ERROR: Replace PROGRAM_ID after deployment"
    exit 1
fi

echo
echo "Program:"
echo "$PROGRAM_ID"

echo
echo "Program Information:"
solana program show \
"$PROGRAM_ID" \
--url "$RPC"

echo
echo "Verification complete"
