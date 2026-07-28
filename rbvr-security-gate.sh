#!/usr/bin/env bash
set -Eeuo pipefail

fail() {
  echo
  echo "RBVR SECURITY GATE: FAIL"
  echo "$1"
  exit 1
}

pass_step() {
  echo "PASS: $1"
}

command -v cargo >/dev/null 2>&1 || fail "cargo is not installed or not in PATH"
command -v anchor >/dev/null 2>&1 || fail "anchor is not installed or not in PATH"
command -v npm >/dev/null 2>&1 || fail "npm is not installed or not in PATH"
command -v git >/dev/null 2>&1 || fail "git is not installed or not in PATH"

ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || fail "Run this from the RBVR Git repository"
cd "$ROOT"

TARGET="programs/treasury-router/src/instructions/process_fees.rs"

grep -q 'pre_linkage_report = sentinel::evaluate_linkage' "$TARGET" \
  || fail "Pre-mutation Sentinel V2 gate is missing"

grep -q 'post_linkage_report = sentinel::evaluate_linkage' "$TARGET" \
  || fail "Post-mutation Sentinel V2 gate is missing"

grep -q 'pre_sentinel_report' "$TARGET" \
  || fail "Pre-mutation Sentinel V1 gate is missing"

grep -q 'post_sentinel_report' "$TARGET" \
  || fail "Post-mutation Sentinel V1 gate is missing"

pass_step "Sentinel fail-closed wiring"

cargo fmt --all -- --check || fail "cargo fmt check failed"
pass_step "Rust formatting"

cargo clippy --workspace --all-targets --all-features -- -D warnings \
  || fail "strict Clippy failed"
pass_step "Strict Clippy"

cargo test --workspace --all-features || fail "Rust tests failed"
pass_step "Rust tests"

npm ci || fail "npm ci failed"
pass_step "Locked Node dependencies"

command -v solana-test-validator >/dev/null 2>&1 \
  || fail "solana-test-validator is not installed or not in PATH"

LEDGER_DIR="$ROOT/.rbvr-test-ledger"
VALIDATOR_LOG="$ROOT/.rbvr-test-validator.log"
RPC_URL="http://127.0.0.1:8899"

cleanup_validator() {
  if [[ -n "${VALIDATOR_PID:-}" ]] && kill -0 "$VALIDATOR_PID" 2>/dev/null; then
    kill "$VALIDATOR_PID" 2>/dev/null || true
    wait "$VALIDATOR_PID" 2>/dev/null || true
  fi

  rm -rf "$LEDGER_DIR"
}

trap cleanup_validator EXIT INT TERM

# Prevent any validator already using the test RPC port from contaminating tests.
if command -v fuser >/dev/null 2>&1; then
  fuser -k 8899/tcp >/dev/null 2>&1 || true
else
  pkill -f solana-test-validator 2>/dev/null || true
fi

sleep 1

rm -rf "$LEDGER_DIR"
rm -f "$VALIDATOR_LOG"

solana-test-validator \
  --reset \
  --ledger "$LEDGER_DIR" \
  --rpc-port 8899 \
  >"$VALIDATOR_LOG" 2>&1 &

VALIDATOR_PID=$!

for _ in {1..30}; do
  if solana cluster-version --url "$RPC_URL" >/dev/null 2>&1; then
    break
  fi

  if ! kill -0 "$VALIDATOR_PID" 2>/dev/null; then
    cat "$VALIDATOR_LOG"
    fail "Fresh local Solana validator exited during startup"
  fi

  sleep 1
done

solana cluster-version --url "$RPC_URL" >/dev/null 2>&1 || {
  cat "$VALIDATOR_LOG"
  fail "Fresh local Solana validator did not become ready"
}

# Confirm the process launched by this script owns the test environment.
kill -0 "$VALIDATOR_PID" 2>/dev/null \
  || fail "Validator PID is not active after startup"

# A clean ledger must not contain the locked RBVR program before deployment.
if solana program show \
  5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3 \
  --url "$RPC_URL" >/dev/null 2>&1; then
  cat "$VALIDATOR_LOG"
  fail "RBVR program already exists before deployment; validator state is not clean"
fi

anchor build || fail "Anchor build failed"

anchor deploy \
  --provider.cluster "$RPC_URL" \
  || {
    cat "$VALIDATOR_LOG"
    fail "Local program deployment failed"
  }

ANCHOR_PROVIDER_URL="$RPC_URL" \
ANCHOR_WALLET="/home/alec_elliott/rocket-beaver-dev/keys/local-deployment.json" \
npm run test:integration \
  || {
    cat "$VALIDATOR_LOG"
    fail "Anchor integration tests failed"
  }

pass_step "Anchor integration tests"

python3 scripts/validate-architecture.py || fail "Architecture validation failed"
pass_step "Architecture validation"

node scripts/beavernomics/verify-config.mjs || fail "Beavernomics config verification failed"
pass_step "Beavernomics config"

# Fail if common private-key formats appear in tracked files.
if git grep -nE \
  'BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY|\\[[[:space:]]*[0-9]{1,3}([[:space:]]*,[[:space:]]*[0-9]{1,3}){31,}[[:space:]]*\\]' \
  -- ':!Cargo.lock' ':!package-lock.json' >/tmp/rbvr-secret-scan.txt; then
  cat /tmp/rbvr-secret-scan.txt
  fail "Possible private key material found in tracked files"
fi
pass_step "Tracked-file secret scan"

# Release artifacts should not contain local review bundles or backup folders.
if git ls-files | grep -E \
  '(^|/)(\\.backups|\\.adaptive-dam-|\\.sentinel-|\\.rbvr-security-hardening-backup)|rbvr-.*source.*\\.txt$'; then
  fail "Backup or source-bundle artifacts are tracked by Git"
fi
pass_step "Release repository hygiene"

git diff --check || fail "Git diff contains whitespace errors"
pass_step "Patch cleanliness"

echo
echo "RBVR SECURITY GATE: PASS"
echo "This confirms the reviewed code-level checks passed locally."
echo "It does not replace devnet attack testing or independent external review."
