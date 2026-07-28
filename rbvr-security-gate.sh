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

npm run test:integration || fail "Anchor integration tests failed"
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
