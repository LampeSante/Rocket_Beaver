#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

OUT="RBVR-RESERVE-INTEGRATION-SOURCE.txt"

FILES=(
  "programs/treasury-router/src/lib.rs"
  "programs/treasury-router/src/errors.rs"
  "programs/treasury-router/src/engines/mod.rs"
  "programs/treasury-router/src/engines/reserve.rs"
  "programs/treasury-router/src/engines/reserve_deployment.rs"
  "programs/treasury-router/src/engines/release.rs"
  "programs/treasury-router/src/engines/execution_guard.rs"
  "programs/treasury-router/src/instructions/reserve.rs"
  "programs/treasury-router/src/instructions/initialize_treasury.rs"
  "programs/treasury-router/src/instructions/initialize_execution_config.rs"
  "programs/treasury-router/src/instructions/mod.rs"
  "programs/treasury-router/src/state/mod.rs"
  "programs/treasury-router/src/state/treasury.rs"
  "programs/treasury-router/src/state/execution_config.rs"
  "programs/treasury-router/src/events/mod.rs"
)

missing=0

for file in "${FILES[@]}"; do
    if [[ ! -f "$file" ]]; then
        echo "Missing expected file: $file"
        missing=1
    fi
done

if [[ "$missing" -ne 0 ]]; then
    echo
    echo "Inspection stopped because one or more expected files are absent."
    exit 1
fi

{
    echo "RBVR RESERVE INTEGRATION SOURCE"
    echo "Generated: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "Branch: $(git branch --show-current)"
    echo "Commit: $(git rev-parse HEAD)"
    echo

    for file in "${FILES[@]}"; do
        echo
        echo "============================================================"
        echo "FILE: $file"
        echo "============================================================"
        cat "$file"
    done

    echo
    echo "============================================================"
    echo "CURRENT RESERVE TESTS"
    echo "============================================================"

    grep -n -A 90 -B 25 \
      'autonomously transfers the reserve allocation' \
      tests/rbvr_protocol.ts || true

    echo
    echo "============================================================"
    echo "RESERVE DESTINATION CREATION"
    echo "============================================================"

    grep -n -A 20 -B 20 \
      'reserveDestination' \
      tests/rbvr_protocol.ts || true

    echo
    echo "============================================================"
    echo "TOKEN AUTHORITY REFERENCES"
    echo "============================================================"

    grep -RInE \
      'reserve_destination|reserveDestination|settlement_vault|token::authority|authority *=|seeds *=|with_signer' \
      programs/treasury-router/src \
      tests/rbvr_protocol.ts || true

    echo
    echo "============================================================"
    echo "GIT STATUS"
    echo "============================================================"

    git status --short
} > "$OUT"

echo "============================================================"
echo "RBVR RESERVE INTEGRATION INSPECTION COMPLETE"
echo "============================================================"
echo "Output: $OUT"
echo
echo "Important architecture rule:"
echo "The existing reserve allocation must fund a protocol-controlled"
echo "Reserve Vault. A separate instruction must deploy only surplus."
echo "============================================================"

wc -l "$OUT"
