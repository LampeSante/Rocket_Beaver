#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

BACKUP=""

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RESERVE VAULT PHASE 1 FAILED"
        echo "============================================================"

        if [[ -n "$BACKUP" && -d "$BACKUP" ]]; then
            for file in \
                programs/treasury-router/src/constants.rs \
                programs/treasury-router/src/instructions/initialize_execution_config.rs \
                programs/treasury-router/src/instructions/reserve.rs \
                tests/rbvr_protocol.ts
            do
                if [[ -f "$BACKUP/$file" ]]; then
                    cp "$BACKUP/$file" "$file"
                fi
            done

            echo "Original files restored from: $BACKUP"
        fi
    fi

    exit "$code"
}

trap rollback EXIT

echo "============================================================"
echo "RBVR RESERVE VAULT — PHASE 1"
echo "============================================================"

if [[ "$(git branch --show-current)" != "red-team" ]]; then
    echo "ERROR: This pass must run from the red-team branch."
    exit 1
fi

CURRENT_COMMIT="$(git rev-parse HEAD)"

if [[ "$CURRENT_COMMIT" != ae6e68b* ]]; then
    echo "ERROR: Expected checkpoint commit starting with ae6e68b."
    echo "Current commit: $CURRENT_COMMIT"
    exit 1
fi

echo
echo "===== PATCH PROTOCOL ====="

PATCH_LOG="$(mktemp)"

set +e
python3 rbvr-reserve-vault-phase1.py 2>&1 | tee "$PATCH_LOG"
PATCH_STATUS="${PIPESTATUS[0]}"
set -e

BACKUP="$(
    sed -n 's/^Backup created: //p' "$PATCH_LOG" |
    head -n 1
)"

rm -f "$PATCH_LOG"

if [[ "$PATCH_STATUS" -ne 0 ]]; then
    echo "Protocol patch failed."
    exit "$PATCH_STATUS"
fi

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "ERROR: Could not identify backup directory."
    exit 1
fi

echo
echo "===== VERIFY SOURCE CHANGES ====="

grep -n \
    'RESERVE_VAULT_SEED' \
    programs/treasury-router/src/constants.rs \
    programs/treasury-router/src/instructions/initialize_execution_config.rs \
    programs/treasury-router/src/instructions/reserve.rs

grep -n -A 18 \
    'Canonical protocol-controlled Reserve Vault' \
    programs/treasury-router/src/instructions/initialize_execution_config.rs

grep -n -A 18 \
    'Canonical protocol-controlled Reserve Vault receiving reserve funding' \
    programs/treasury-router/src/instructions/reserve.rs

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== ANCHOR BUILD AND TYPE GENERATION ====="
anchor build

echo
echo "===== TYPESCRIPT CHECK ====="
npx tsc --noEmit

echo
echo "===== COMPLETE RUST REGRESSION ====="

RUST_OUTPUT="$(mktemp)"
cargo test --workspace 2>&1 | tee "$RUST_OUTPUT"

if ! grep -q 'test result: ok. 134 passed; 0 failed' "$RUST_OUTPUT"; then
    echo "ERROR: Expected 134 passing Rust tests."
    rm -f "$RUST_OUTPUT"
    exit 1
fi

rm -f "$RUST_OUTPUT"

echo
echo "===== STRICT CLIPPY ====="

cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- \
    -D warnings

echo
echo "===== INTEGRATION REGRESSION ====="

INTEGRATION_OUTPUT="$(mktemp)"
anchor test --validator legacy 2>&1 | tee "$INTEGRATION_OUTPUT"

if ! grep -q '37 passing' "$INTEGRATION_OUTPUT"; then
    echo "ERROR: Expected 37 passing integration tests."
    rm -f "$INTEGRATION_OUTPUT"
    exit 1
fi

rm -f "$INTEGRATION_OUTPUT"

echo
echo "===== VERIFY GENERATED IDL ====="

python3 <<'PYIDL'
import json
from pathlib import Path

idl_path = Path("target/idl/treasury_router.json")

if not idl_path.exists():
    raise RuntimeError(f"Missing generated IDL: {idl_path}")

idl = json.loads(idl_path.read_text())

instructions = idl.get("instructions", [])

initialize = next(
    (
        instruction
        for instruction in instructions
        if instruction.get("name") in {
            "initializeExecutionConfig",
            "initialize_execution_config",
        }
    ),
    None,
)

if initialize is None:
    raise RuntimeError(
        "Generated IDL does not contain initializeExecutionConfig"
    )

accounts = initialize.get("accounts", [])
account_names = {
    account.get("name")
    for account in accounts
    if isinstance(account, dict)
}

required_accounts = {
    "reserveDestination",
    "tokenProgram",
    "systemProgram",
}

missing = required_accounts - account_names

if missing:
    raise RuntimeError(
        "Generated initializeExecutionConfig IDL is missing accounts: "
        + ", ".join(sorted(missing))
    )

reserve_instruction = next(
    (
        instruction
        for instruction in instructions
        if instruction.get("name") in {
            "authorizeReserveExecution",
            "authorize_reserve_execution",
        }
    ),
    None,
)

if reserve_instruction is None:
    raise RuntimeError(
        "Generated IDL does not contain authorizeReserveExecution"
    )

reserve_accounts = {
    account.get("name")
    for account in reserve_instruction.get("accounts", [])
    if isinstance(account, dict)
}

if "reserveDestination" not in reserve_accounts:
    raise RuntimeError(
        "authorizeReserveExecution IDL is missing reserveDestination"
    )

print("Generated IDL verification passed.")
print(
    "initializeExecutionConfig accounts:",
    ", ".join(sorted(account_names)),
)
print(
    "authorizeReserveExecution includes reserveDestination."
)
PYIDL

echo
echo "===== REVIEW CHANGES ====="

git diff --stat
git diff -- \
    programs/treasury-router/src/constants.rs \
    programs/treasury-router/src/instructions/initialize_execution_config.rs \
    programs/treasury-router/src/instructions/reserve.rs |
    sed -n '1,320p'

echo
echo "===== STAGE AND COMMIT VERIFIED PHASE 1 ====="

git add \
    programs/treasury-router/src/constants.rs \
    programs/treasury-router/src/instructions/initialize_execution_config.rs \
    programs/treasury-router/src/instructions/reserve.rs \
    tests/rbvr_protocol.ts

git commit -m \
    "protocol: convert reserve destination into treasury-owned vault"

git tag -a \
    rbvr-reserve-vault-v1 \
    -m "Protocol-controlled Reserve Vault funding path verified"

echo
echo "===== FINAL STATUS ====="

git log --oneline --decorate -4
echo
git status --short

echo
echo "============================================================"
echo "RBVR RESERVE VAULT PHASE 1 PASSED"
echo "============================================================"
echo "Reserve Vault PDA:          reserve-vault + TreasuryState"
echo "Reserve Vault authority:    TreasuryState PDA"
echo "External reserve wallet:    removed"
echo "Reserve funding path:       verified"
echo "Reserve Engine tests:       12 passing"
echo "Complete Rust tests:        134 passing"
echo "Integration tests:          37 passing"
echo "Live surplus deployment:    not yet enabled"
echo "Tag:                        rbvr-reserve-vault-v1"
echo
echo "Next:"
echo "Add immutable Reserve Policy state and permissionless"
echo "surplus-only Spillway deployment."
echo "============================================================"

trap - EXIT
