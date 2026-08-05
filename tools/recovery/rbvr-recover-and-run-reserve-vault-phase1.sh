#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

GENERATOR="rbvr-reserve-vault-phase1.py"
RUNNER="rbvr-reserve-vault-phase1.sh"

FILES=(
  "programs/treasury-router/src/constants.rs"
  "programs/treasury-router/src/instructions/initialize_execution_config.rs"
  "programs/treasury-router/src/instructions/reserve.rs"
  "tests/rbvr_protocol.ts"
)

echo "============================================================"
echo "RECOVER AND RERUN RESERVE VAULT PHASE 1"
echo "============================================================"

if [[ ! -f "$GENERATOR" ]]; then
    echo "ERROR: Missing generator: $GENERATOR"
    exit 1
fi

if [[ ! -f "$RUNNER" ]]; then
    echo "ERROR: Missing runner: $RUNNER"
    exit 1
fi

echo
echo "===== LOCATE LATEST PHASE 1 BACKUP ====="

BACKUP="$(
    find . \
      -maxdepth 1 \
      -type d \
      -name '.reserve-vault-phase1-backup-*' \
      -printf '%T@ %p\n' |
    sort -nr |
    head -n 1 |
    cut -d' ' -f2-
)"

if [[ -z "$BACKUP" || ! -d "$BACKUP" ]]; then
    echo "ERROR: No Reserve Vault Phase 1 backup was found."
    exit 1
fi

echo "Using backup: $BACKUP"

echo
echo "===== RESTORE CLEAN CHECKPOINT FILES ====="

for file in "${FILES[@]}"; do
    if [[ ! -f "$BACKUP/$file" ]]; then
        echo "ERROR: Backup is missing: $BACKUP/$file"
        exit 1
    fi

    cp "$BACKUP/$file" "$file"
    echo "Restored: $file"
done

echo
echo "===== VERIFY RESTORED STATE ====="

if grep -q 'RESERVE_VAULT_SEED' \
    programs/treasury-router/src/constants.rs; then
    echo "ERROR: Restored constants still contain RESERVE_VAULT_SEED."
    exit 1
fi

if grep -q 'Reserve Vault funding completed' \
    programs/treasury-router/src/instructions/reserve.rs; then
    echo "ERROR: Restored reserve instruction still contains Phase 1 changes."
    exit 1
fi

grep -n -A 9 -B 1 \
    'reserveDestination = (' \
    tests/rbvr_protocol.ts |
    head -n 14

echo
echo "===== PATCH GENERATOR TO USE EXACT BLOCK ====="

python3 <<'PY'
from pathlib import Path

path = Path("rbvr-reserve-vault-phase1.py")
text = path.read_text()

start_marker = """# Remove creation of the former externally owned Reserve ATA.
"""
end_marker = """# Add tokenProgram to every initializeExecutionConfig account block.
"""

start = text.find(start_marker)
if start == -1:
    raise RuntimeError("Could not find reserve ATA patch section")

end = text.find(end_marker, start)
if end == -1:
    raise RuntimeError("Could not find end of reserve ATA patch section")

replacement = r'''# Remove creation of the former externally owned Reserve ATA.
old_reserve_ata = """    reserveDestination = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        reserveRecipientOwner.publicKey,
      )
    ).address;

"""

if old_reserve_ata not in tests:
    raise RuntimeError(
        "Could not locate the exact external reserve ATA creation block"
    )

tests = tests.replace(old_reserve_ata, "", 1)

'''

updated = text[:start] + replacement + text[end:]
path.write_text(updated)

print("Generator now uses an exact literal reserve ATA replacement.")
PY

echo
echo "===== FIX RUNNER BACKUP RECOVERY ====="

python3 <<'PY'
from pathlib import Path

path = Path("rbvr-reserve-vault-phase1.sh")
text = path.read_text()

old = '''PATCH_OUTPUT="$(python3 rbvr-reserve-vault-phase1.py)"
printf '%s\n' "$PATCH_OUTPUT"

BACKUP="$(
    printf '%s\n' "$PATCH_OUTPUT" |
    sed -n 's/^Backup created: //p' |
    head -n 1
)"
'''

new = '''PATCH_LOG="$(mktemp)"

if ! python3 rbvr-reserve-vault-phase1.py 2>&1 | tee "$PATCH_LOG"; then
    BACKUP="$(
        sed -n 's/^Backup created: //p' "$PATCH_LOG" |
        head -n 1
    )"

    rm -f "$PATCH_LOG"
    exit 1
fi

BACKUP="$(
    sed -n 's/^Backup created: //p' "$PATCH_LOG" |
    head -n 1
)"

rm -f "$PATCH_LOG"
'''

if old not in text:
    raise RuntimeError("Could not locate runner patch-output block")

path.write_text(text.replace(old, new, 1))

print("Runner can now recover the backup even if Python fails.")
PY

echo
echo "===== VERIFY GENERATOR PATCH ====="

grep -n -A 20 \
    'Remove creation of the former externally owned Reserve ATA' \
    "$GENERATOR"

echo
echo "===== RERUN RESERVE VAULT PHASE 1 ====="

chmod +x "$RUNNER"
./"$RUNNER"
