#!/bin/bash

echo "RBVR Mainnet Deployment Commands"
echo
echo "WARNING:"
echo "Review all commands before execution."
echo "Do not run until deployment wallet and RPC are confirmed."
echo

echo "1. Configure Mainnet RPC"
echo "solana config set --url https://api.mainnet-beta.solana.com"

echo

echo "2. Confirm wallet"
echo "solana address"
echo "solana balance"

echo

echo "3. Deploy program"
echo "solana program deploy target/deploy/treasury_router.so"

echo

echo "4. Verify deployment"
echo "solana program show <PROGRAM_ID> --url https://api.mainnet-beta.solana.com"

echo

echo "5. Revoke upgrade authority"
echo "solana program set-upgrade-authority <PROGRAM_ID> --final --url https://api.mainnet-beta.solana.com"

echo

echo "6. Confirm immutable state"
echo "solana program show <PROGRAM_ID> --url https://api.mainnet-beta.solana.com"

