#!/bin/bash

PROGRAM_ID="5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3"
RPC="https://api.mainnet-beta.solana.com"

echo "================================"
echo "RBVR Mainnet Verification"
echo "================================"

echo
echo "Program:"
echo $PROGRAM_ID

echo
echo "Program Information:"
solana program show \
$PROGRAM_ID \
--url $RPC

echo
echo "================================"
echo "Authority Check"
echo "================================"

AUTHORITY=$(solana program show \
$PROGRAM_ID \
--url $RPC | grep Authority)

echo $AUTHORITY

if echo "$AUTHORITY" | grep -q "none"; then
    echo "PASS: Upgrade authority revoked"
else
    echo "WARNING: Upgrade authority still active"
fi

echo
echo "================================"
echo "Verification Complete"
echo "================================"
