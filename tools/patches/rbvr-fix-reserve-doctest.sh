#!/usr/bin/env bash
set -Eeuo pipefail

FILE="programs/treasury-router/src/engines/reserve_deployment.rs"

if [[ ! -f "$FILE" ]]; then
    echo "Missing file: $FILE"
    exit 1
fi

BACKUP="${FILE}.bak.$(date -u +%Y%m%dT%H%M%SZ)"
cp "$FILE" "$BACKUP"

python3 <<'PY'
from pathlib import Path

path = Path("programs/treasury-router/src/engines/reserve_deployment.rs")
text = path.read_text()

old = """//! Core invariant:
//!
//!     remaining_reserve >= reserve_floor
//!
//! Only value above the calculated reserve floor may ever become deployable.
"""

new = """//! Core invariant:
//!
//! ```text
//! remaining_reserve >= reserve_floor
//! ```
//!
//! Only value above the calculated reserve floor may ever become deployable.
"""

if old not in text:
    raise SystemExit(
        "Could not locate the original invariant documentation block."
    )

text = text.replace(old, new, 1)

path.write_text(text)

print("Documentation block updated successfully.")
PY

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== VERIFY DOC TESTS ====="
cargo test --doc

echo
echo "===== FULL REGRESSION ====="
cargo test --workspace

echo
echo "============================================================"
echo "RESERVE ENGINE DOCUMENTATION FIX COMPLETE"
echo "============================================================"
