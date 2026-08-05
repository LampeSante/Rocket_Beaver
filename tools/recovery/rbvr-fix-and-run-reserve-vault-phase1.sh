#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

RUNNER="rbvr-reserve-vault-phase1.sh"

if [[ ! -f "$RUNNER" ]]; then
    echo "ERROR: Missing runner: $RUNNER"
    exit 1
fi

echo "============================================================"
echo "FIX AND RUN RESERVE VAULT PHASE 1"
echo "============================================================"

echo
echo "===== REPLACE BRITTLE IDL VERIFICATION ====="

python3 <<'PY'
from pathlib import Path

path = Path("rbvr-reserve-vault-phase1.sh")
text = path.read_text()

start_marker = '''echo
echo "===== VERIFY GENERATED IDL ====="
'''

end_marker = '''echo
echo "===== REVIEW CHANGES ====="
'''

start = text.find(start_marker)
if start == -1:
    raise RuntimeError("Could not locate IDL verification section")

end = text.find(end_marker, start)
if end == -1:
    raise RuntimeError("Could not locate end of IDL verification section")

replacement = r'''echo
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

'''

path.write_text(text[:start] + replacement + text[end:])

print("IDL verification replaced successfully.")
PY

echo
echo "===== VERIFY PATCH ORDER ====="

grep -nE \
    'ANCHOR BUILD AND TYPE GENERATION|TYPESCRIPT CHECK|VERIFY GENERATED IDL|Generated IDL verification passed' \
    "$RUNNER"

echo
echo "===== VERIFY CLEAN CHECKPOINT ====="

CURRENT_COMMIT="$(git rev-parse HEAD)"

if [[ "$CURRENT_COMMIT" != ae6e68b* ]]; then
    echo "ERROR: Expected checkpoint commit beginning with ae6e68b."
    echo "Current commit: $CURRENT_COMMIT"
    exit 1
fi

if grep -q 'RESERVE_VAULT_SEED' \
    programs/treasury-router/src/constants.rs; then
    echo "ERROR: Reserve Vault changes are unexpectedly present before rerun."
    exit 1
fi

echo "Checkpoint state verified."

echo
echo "===== RUN RESERVE VAULT PHASE 1 ====="

chmod +x "$RUNNER"
./"$RUNNER"
