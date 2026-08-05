#!/usr/bin/env bash
set -Eeuo pipefail

REPO="${RBVR_REPO:-$HOME/rocket-beaver-contract}"
PROGRAM_NAME="${RBVR_PROGRAM_NAME:-treasury_router}"
EXPECTED_PROGRAM_ID="${RBVR_PROGRAM_ID:-5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3}"
DEVNET_URL="${RBVR_DEVNET_URL:-https://api.devnet.solana.com}"

TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
REPORT_DIR="$REPO/deployments/devnet"
REPORT="$REPORT_DIR/RBVR-DEVNET-DEPLOYMENT-$TIMESTAMP.md"
LOG="$REPORT_DIR/RBVR-DEVNET-DEPLOYMENT-$TIMESTAMP.log"
SO_FILE="$REPO/target/deploy/${PROGRAM_NAME}.so"
PROGRAM_KEYPAIR="$REPO/target/deploy/${PROGRAM_NAME}-keypair.json"

mkdir -p "$REPORT_DIR"
exec > >(tee -a "$LOG") 2>&1

die(){ echo "ERROR: $*" >&2; exit 1; }
need(){ command -v "$1" >/dev/null 2>&1 || die "Missing command: $1"; }

for c in git anchor solana solana-keygen sha256sum awk grep pgrep; do need "$c"; done
[[ -d "$REPO/.git" ]] || die "Repository not found: $REPO"
cd "$REPO"

echo "=== RBVR DEVNET DEPLOYMENT GATE ==="
echo "Repository: $REPO"

if pgrep -af 'cargo-fuzz|cargo fuzz|fuzz run' >/tmp/rbvr-fuzz-processes.txt 2>/dev/null; then
  cat /tmp/rbvr-fuzz-processes.txt
  die "Fuzzing is still running. Deployment is blocked until the fuzz-tested commit is final."
fi

[[ -z "$(git status --porcelain)" ]] || {
  git status --short
  die "Git worktree is not clean."
}

COMMIT="$(git rev-parse HEAD)"
BRANCH="$(git branch --show-current || true)"
TAG="$(git describe --tags --exact-match 2>/dev/null || true)"

echo "Commit: $COMMIT"
echo "Branch: ${BRANCH:-detached HEAD}"
echo "Tag: ${TAG:-none}"

WALLET_PATH="$(solana config get | awk -F': ' '/Keypair Path/ {print $2}')"
[[ -n "$WALLET_PATH" ]] || die "Could not determine wallet path."
WALLET_ADDRESS="$(solana address -k "$WALLET_PATH")"
BALANCE="$(solana balance "$WALLET_ADDRESS" --url "$DEVNET_URL" | awk '{print $1}')"

echo "Wallet: $WALLET_ADDRESS"
echo "Devnet balance: $BALANCE SOL"
awk -v b="$BALANCE" 'BEGIN { exit !(b >= 2.0) }' || die "Need at least 2 devnet SOL."

echo "=== BUILD ==="
anchor build
[[ -f "$SO_FILE" ]] || die "Missing binary: $SO_FILE"
[[ -f "$PROGRAM_KEYPAIR" ]] || die "Missing program keypair: $PROGRAM_KEYPAIR"

BUILT_PROGRAM_ID="$(solana-keygen pubkey "$PROGRAM_KEYPAIR")"
[[ "$BUILT_PROGRAM_ID" == "$EXPECTED_PROGRAM_ID" ]] ||
  die "Program ID mismatch: expected $EXPECTED_PROGRAM_ID, found $BUILT_PROGRAM_ID"

LOCAL_HASH="$(sha256sum "$SO_FILE" | awk '{print $1}')"
LOCAL_SIZE="$(stat -c '%s' "$SO_FILE")"

if solana program show "$EXPECTED_PROGRAM_ID" --url "$DEVNET_URL" >/tmp/rbvr-before.txt 2>&1; then
  cat /tmp/rbvr-before.txt
  die "Program already exists on devnet. Automatic upgrade blocked."
fi

echo "=== DEPLOY ==="
DEPLOY_OUTPUT="$(solana program deploy "$SO_FILE"   --program-id "$PROGRAM_KEYPAIR"   --url "$DEVNET_URL"   --keypair "$WALLET_PATH")"
echo "$DEPLOY_OUTPUT"

PROGRAM_SHOW="$(solana program show "$EXPECTED_PROGRAM_ID" --url "$DEVNET_URL")"
echo "$PROGRAM_SHOW"

PROGRAMDATA_ADDRESS="$(printf '%s\n' "$PROGRAM_SHOW" | awk -F': ' '/ProgramData Address/ {print $2; exit}')"
UPGRADE_AUTHORITY="$(printf '%s\n' "$PROGRAM_SHOW" | awk -F': ' '/Authority/ {print $2; exit}')"
POST_BALANCE="$(solana balance "$WALLET_ADDRESS" --url "$DEVNET_URL" | awk '{print $1}')"

cat > "$REPORT" <<EOF
# RBVR Devnet Deployment Report

- Deployment time: $(date -u '+%Y-%m-%d %H:%M:%S UTC')
- Git commit: \`$COMMIT\`
- Git branch: \`${BRANCH:-detached HEAD}\`
- Exact tag: \`${TAG:-none}\`
- Program ID: \`$EXPECTED_PROGRAM_ID\`
- ProgramData address: \`${PROGRAMDATA_ADDRESS:-not parsed}\`
- Upgrade authority: \`${UPGRADE_AUTHORITY:-not parsed}\`
- Deployment wallet: \`$WALLET_ADDRESS\`
- Balance before: \`$BALANCE SOL\`
- Balance after: \`$POST_BALANCE SOL\`
- Binary SHA-256: \`$LOCAL_HASH\`
- Binary size: \`$LOCAL_SIZE bytes\`

## Deployment output

\`\`\`text
$DEPLOY_OUTPUT
\`\`\`

## On-chain program information

\`\`\`text
$PROGRAM_SHOW
\`\`\`

The devnet program remains upgradeable intentionally. This script does not initialize protocol PDAs and does not revoke upgrade authority.
EOF

echo "=== COMPLETE ==="
echo "Program ID: $EXPECTED_PROGRAM_ID"
echo "ProgramData: ${PROGRAMDATA_ADDRESS:-not parsed}"
echo "Upgrade authority: ${UPGRADE_AUTHORITY:-not parsed}"
echo "Report: $REPORT"
echo "Log: $LOG"
