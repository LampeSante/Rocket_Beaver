#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

OUT="RBVR-RESERVE-INTEGRATION-SOURCE.txt"
SRC="programs/treasury-router/src"

if [[ ! -d "$SRC" ]]; then
    echo "Missing source directory: $SRC"
    exit 1
fi

REQUIRED_FILES=(
  "$SRC/lib.rs"
  "$SRC/engines/mod.rs"
  "$SRC/engines/reserve.rs"
  "$SRC/engines/reserve_deployment.rs"
  "$SRC/engines/release.rs"
  "$SRC/engines/execution_guard.rs"
  "$SRC/instructions/reserve.rs"
  "$SRC/instructions/initialize_treasury.rs"
  "$SRC/instructions/initialize_execution_config.rs"
  "$SRC/instructions/mod.rs"
  "$SRC/state/mod.rs"
)

missing=0

for file in "${REQUIRED_FILES[@]}"; do
    if [[ ! -f "$file" ]]; then
        echo "Missing required file: $file"
        missing=1
    fi
done

if [[ "$missing" -ne 0 ]]; then
    echo
    echo "Inspection stopped because a required reserve file is missing."
    exit 1
fi

mapfile -t DISCOVERED_FILES < <(
    {
        find "$SRC" \
          -maxdepth 3 \
          -type f \
          \( \
            -path '*/errors/*.rs' -o \
            -name 'error.rs' -o \
            -name 'errors.rs' -o \
            -path '*/events/*.rs' -o \
            -name 'events.rs' -o \
            -path '*/state/*.rs' -o \
            -path '*/instructions/*.rs' \
          \)

        printf '%s\n' "${REQUIRED_FILES[@]}"
    } |
    sort -u
)

if [[ "${#DISCOVERED_FILES[@]}" -eq 0 ]]; then
    echo "No source files were discovered."
    exit 1
fi

{
    echo "RBVR RESERVE INTEGRATION SOURCE"
    echo "Generated: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "Branch: $(git branch --show-current)"
    echo "Commit: $(git rev-parse HEAD)"
    echo

    echo "============================================================"
    echo "DISCOVERED ERROR MODULES"
    echo "============================================================"

    find "$SRC" \
      -maxdepth 4 \
      -type f \
      \( \
        -path '*/errors/*.rs' -o \
        -name 'error.rs' -o \
        -name 'errors.rs' \
      \) \
      -print | sort

    echo
    echo "============================================================"
    echo "DISCOVERED SOURCE FILES"
    echo "============================================================"

    printf '%s\n' "${DISCOVERED_FILES[@]}"

    for file in "${DISCOVERED_FILES[@]}"; do
        echo
        echo "============================================================"
        echo "FILE: $file"
        echo "============================================================"
        cat "$file"
    done

    echo
    echo "============================================================"
    echo "ERROR DEFINITIONS AND REFERENCES"
    echo "============================================================"

    grep -RInE \
      '#\[error_code\]|enum .*Error|ErrorCode|error!\(|err!\(|require!\(|require_eq!\(|require_keys_eq!\(' \
      "$SRC" || true

    echo
    echo "============================================================"
    echo "RESERVE INTEGRATION REFERENCES"
    echo "============================================================"

    grep -RInE \
      'reserve_destination|reserveDestination|reserve_deployment|ReserveDeployment|pending_reserve|released_reserve|lifetime_reserve' \
      "$SRC" \
      tests/rbvr_protocol.ts || true

    echo
    echo "============================================================"
    echo "TOKEN VAULT AND AUTHORITY REFERENCES"
    echo "============================================================"

    grep -RInE \
      'settlement_vault|TREASURY_VAULT_SEED|token::authority|CpiContext::new_with_signer|with_signer|seeds *=|bump *=' \
      "$SRC" \
      tests/rbvr_protocol.ts || true

    echo
    echo "============================================================"
    echo "CURRENT RESERVE INTEGRATION TEST"
    echo "============================================================"

    grep -n -A 120 -B 30 \
      'autonomously transfers the reserve allocation' \
      tests/rbvr_protocol.ts || true

    echo
    echo "============================================================"
    echo "EXECUTION CONFIG INITIALIZATION TEST"
    echo "============================================================"

    grep -n -A 140 -B 30 \
      'initializes the execution config' \
      tests/rbvr_protocol.ts || true

    echo
    echo "============================================================"
    echo "RESERVE DESTINATION CREATION"
    echo "============================================================"

    grep -n -A 30 -B 30 \
      'reserveDestination' \
      tests/rbvr_protocol.ts || true

    echo
    echo "============================================================"
    echo "PUBLIC ENTRYPOINTS"
    echo "============================================================"

    grep -nE \
      '^[[:space:]]*pub fn ' \
      "$SRC/lib.rs" || true

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
echo "Source files captured: ${#DISCOVERED_FILES[@]}"
echo
echo "Error modules found:"
find "$SRC" \
  -maxdepth 4 \
  -type f \
  \( \
    -path '*/errors/*.rs' -o \
    -name 'error.rs' -o \
    -name 'errors.rs' \
  \) \
  -print | sort
echo
wc -l "$OUT"
echo "============================================================"
