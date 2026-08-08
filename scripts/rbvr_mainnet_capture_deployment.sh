#!/bin/bash

set -e

RPC="https://api.mainnet-beta.solana.com"

if [ -z "$1" ]; then
    echo "Usage:"
    echo "./scripts/rbvr_mainnet_capture_deployment.sh <PROGRAM_ID>"
    exit 1
fi

PROGRAM_ID="$1"

OUTPUT="RBVR-MAINNET-DEPLOYMENT-EVIDENCE.md"

echo "================================"
echo "RBVR MAINNET DEPLOYMENT CAPTURE"
echo "================================"

SLOT=$(solana slot --url "$RPC")

echo "# Rocket Beaver (\$RBVR)" > "$OUTPUT"
echo "# Mainnet Deployment Evidence" >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "## Deployment" >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "Network:" >> "$OUTPUT"
echo "Solana Mainnet" >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "Program ID:" >> "$OUTPUT"
echo "$PROGRAM_ID" >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "Verification Slot:" >> "$OUTPUT"
echo "$SLOT" >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "---" >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "## Program Information" >> "$OUTPUT"
echo "" >> "$OUTPUT"

solana program show \
"$PROGRAM_ID" \
--url "$RPC" >> "$OUTPUT"

echo "" >> "$OUTPUT"
echo "---" >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "Captured:"
echo "$OUTPUT"
