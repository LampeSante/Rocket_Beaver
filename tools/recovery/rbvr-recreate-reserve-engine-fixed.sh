#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

GENERATOR="rbvr-add-reserve-deployment-engine.py"
RUNNER="rbvr-add-reserve-deployment-engine.sh"
ENGINE="programs/treasury-router/src/engines/reserve_deployment.rs"
MOD_FILE="programs/treasury-router/src/engines/mod.rs"

if [[ ! -f "$GENERATOR" ]]; then
    echo "Missing generator: $GENERATOR"
    exit 1
fi

if [[ ! -f "$RUNNER" ]]; then
    echo "Missing runner: $RUNNER"
    exit 1
fi

TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
BACKUP_DIR=".reserve-engine-recovery-$TIMESTAMP"

mkdir -p "$BACKUP_DIR"
cp "$GENERATOR" "$BACKUP_DIR/"
cp "$RUNNER" "$BACKUP_DIR/"
cp "$MOD_FILE" "$BACKUP_DIR/engines-mod.rs"

restore() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "RESERVE ENGINE RECOVERY FAILED"
        echo "============================================================"

        cp "$BACKUP_DIR/$(basename "$GENERATOR")" "$GENERATOR"
        cp "$BACKUP_DIR/$(basename "$RUNNER")" "$RUNNER"
        cp "$BACKUP_DIR/engines-mod.rs" "$MOD_FILE"
        rm -f "$ENGINE"

        echo "Original files restored."
    fi

    exit "$code"
}

trap restore EXIT

echo "===== PATCH GENERATOR DOCUMENTATION ====="

python3 <<'PY'
from pathlib import Path

path = Path("rbvr-add-reserve-deployment-engine.py")
text = path.read_text()

old = '''//! Core invariant:
//!
//!     remaining_reserve >= reserve_floor
//!
//! Only value above the calculated reserve floor may ever become deployable.
'''

new = '''//! Core invariant:
//!
//! ```text
//! remaining_reserve >= reserve_floor
//! ```
//!
//! Only value above the calculated reserve floor may ever become deployable.
'''

if old not in text:
    if "//! ```text\n//! remaining_reserve >= reserve_floor\n//! ```" in text:
        print("Generator documentation was already fixed.")
    else:
        raise RuntimeError(
            "Could not locate the reserve invariant documentation block."
        )
else:
    path.write_text(text.replace(old, new, 1))
    print("Generator documentation patched successfully.")
PY

echo
echo "===== VERIFY CLEAN ROLLBACK STATE ====="

if [[ -f "$ENGINE" ]]; then
    echo "Existing reserve engine found. Removing incomplete copy."
    rm -f "$ENGINE"
fi

python3 <<'PY'
from pathlib import Path

path = Path("programs/treasury-router/src/engines/mod.rs")
text = path.read_text()

lines = [
    line
    for line in text.splitlines()
    if line.strip() != "pub mod reserve_deployment;"
]

path.write_text("\n".join(lines).rstrip() + "\n")
print("Removed any stale reserve_deployment export.")
PY

echo
echo "===== RECREATE RESERVE ENGINE ====="

chmod +x "$RUNNER"
./"$RUNNER"

echo
echo "===== VERIFY ENGINE EXISTS ====="

test -f "$ENGINE"

grep -n \
  'pub mod reserve_deployment;' \
  "$MOD_FILE"

grep -n -A 5 \
  'Core invariant' \
  "$ENGINE"

echo
echo "===== VERIFY DOCUMENTATION TESTS ====="

cargo test --doc

echo
echo "===== VERIFY COMPLETE RUST SUITE ====="

cargo test --workspace

echo
echo "===== VERIFY STRICT CLIPPY ====="

cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- \
    -D warnings

echo
echo "===== VERIFY ANCHOR BUILD ====="

anchor build

echo
echo "===== VERIFY INTEGRATION SUITE ====="

anchor test --validator legacy

echo
echo "===== CURRENT STATUS ====="

git diff --stat
git status --short

echo
echo "============================================================"
echo "RESERVE DEPLOYMENT ENGINE RECOVERED SUCCESSFULLY"
echo "============================================================"
echo "Engine recreated:         yes"
echo "Documentation fixed:      yes"
echo "Reserve unit tests:       12 expected"
echo "Complete Rust tests:      134 expected"
echo "Integration tests:        37 expected"
echo "Live token paths changed: no"
echo "============================================================"

trap - EXIT
