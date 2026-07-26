#!/usr/bin/env bash
set -euo pipefail

ROOT="$(pwd)"

OLD_CONFIG="scripts/meteora/devnet/create-dbc-config-v8.mjs"
OLD_POOL="scripts/meteora/devnet/create-rbvr-pool-v9.mjs"

NEW_CONFIG="scripts/meteora/devnet/create-dbc-config-v10.mjs"
NEW_POOL="scripts/meteora/devnet/create-rbvr-pool-v10.mjs"

OLD_ENV=".localnet/devnet-v9/rbvr-devnet.env"
NEW_DIR=".localnet/devnet-v10"
NEW_ENV="${NEW_DIR}/rbvr-devnet.env"

NEW_MANIFEST="deployments/devnet/rbvr-beavernomics-v10.json"

echo "============================================================"
echo " Rocket Beaver — Prepare Devnet V10"
echo "============================================================"

for file in "$OLD_CONFIG" "$OLD_POOL"; do
  if [[ ! -f "$file" ]]; then
    echo "ERROR: Required source script not found:"
    echo "  $file"
    exit 1
  fi
done

mkdir -p "$NEW_DIR"
mkdir -p deployments/devnet

if [[ -e "$NEW_CONFIG" || -e "$NEW_POOL" ]]; then
  echo "ERROR: A V10 script already exists."
  echo "Nothing was overwritten."
  exit 1
fi

cp "$OLD_CONFIG" "$NEW_CONFIG"
cp "$OLD_POOL" "$NEW_POOL"

if [[ -f "$OLD_ENV" ]]; then
  cp "$OLD_ENV" "$NEW_ENV"
else
  echo "WARNING: Existing environment file was not found:"
  echo "  $OLD_ENV"
  echo
  echo "The new environment file must be completed manually:"
  echo "  $NEW_ENV"
  touch "$NEW_ENV"
  chmod 600 "$NEW_ENV"
fi

python3 <<'PY'
from pathlib import Path
import re
import sys

config_file = Path("scripts/meteora/devnet/create-dbc-config-v10.mjs")
pool_file = Path("scripts/meteora/devnet/create-rbvr-pool-v10.mjs")

config = config_file.read_text()
pool = pool_file.read_text()

# --------------------------------------------------------------------------
# CONFIG SCRIPT
# --------------------------------------------------------------------------

# Route all DBC trading fees to the pool creator.
updated, count = re.subn(
    r"creatorTradingFeePercentage\s*:\s*0\b",
    "creatorTradingFeePercentage: 100",
    config,
)

if count != 1:
    raise SystemExit(
        "ERROR: Expected exactly one "
        "'creatorTradingFeePercentage: 0' in the V8 config script, "
        f"but found {count}."
    )

config = updated

# Set a fixed 1.00% fee by changing both scheduler endpoints to 100 bps.
starting_pattern = r"startingFeeBps\s*:\s*\d+"
ending_pattern = r"endingFeeBps\s*:\s*\d+"

config, starting_count = re.subn(
    starting_pattern,
    "startingFeeBps: 100",
    config,
)

config, ending_count = re.subn(
    ending_pattern,
    "endingFeeBps: 100",
    config,
)

if starting_count < 1:
    raise SystemExit(
        "ERROR: Could not find startingFeeBps in the config script."
    )

if ending_count < 1:
    raise SystemExit(
        "ERROR: Could not find endingFeeBps in the config script."
    )

# Keep dynamic fees off.
config = re.sub(
    r"dynamicFeeEnabled\s*:\s*true\b",
    "dynamicFeeEnabled: false",
    config,
)

# Move local V9 output into a separate V10 folder.
config = config.replace(
    ".localnet/devnet-v9",
    ".localnet/devnet-v10",
)

config = config.replace(
    "create-dbc-config-v8.mjs",
    "create-dbc-config-v10.mjs",
)

config = config.replace(
    "DBC Devnet Config V8",
    "DBC Devnet Config V10",
)

config = config.replace(
    "V8 CONFIG",
    "V10 CONFIG",
)

# --------------------------------------------------------------------------
# POOL SCRIPT
# --------------------------------------------------------------------------

pool = pool.replace(
    ".localnet/devnet-v9",
    ".localnet/devnet-v10",
)

pool = pool.replace(
    "deployments/devnet/rbvr-deployment.json",
    "deployments/devnet/rbvr-beavernomics-v10.json",
)

pool = pool.replace(
    "create-rbvr-pool-v9.mjs",
    "create-rbvr-pool-v10.mjs",
)

pool = pool.replace(
    "Deployment V9",
    "Deployment V10",
)

pool = pool.replace(
    "V9 DEPLOYMENT",
    "V10 DEPLOYMENT",
)

config_file.write_text(config)
pool_file.write_text(pool)

print("Created:")
print(f"  {config_file}")
print(f"  {pool_file}")
print()
print("Applied:")
print("  startingFeeBps: 100")
print("  endingFeeBps: 100")
print("  creatorTradingFeePercentage: 100")
print("  dynamicFeeEnabled: false")
PY

chmod 700 "$NEW_CONFIG" "$NEW_POOL"
chmod 600 "$NEW_ENV" 2>/dev/null || true

echo
echo "Verifying critical settings..."
echo "------------------------------------------------------------"

grep -n \
  "startingFeeBps\|endingFeeBps\|creatorTradingFeePercentage\|dynamicFeeEnabled" \
  "$NEW_CONFIG"

echo "------------------------------------------------------------"

python3 <<'PY'
from pathlib import Path
import re

text = Path(
    "scripts/meteora/devnet/create-dbc-config-v10.mjs"
).read_text()

checks = {
    "creator allocation is 100%":
        bool(re.search(r"creatorTradingFeePercentage\s*:\s*100\b", text)),
    "starting fee is 100 bps":
        bool(re.search(r"startingFeeBps\s*:\s*100\b", text)),
    "ending fee is 100 bps":
        bool(re.search(r"endingFeeBps\s*:\s*100\b", text)),
    "dynamic fee is disabled":
        not bool(re.search(r"dynamicFeeEnabled\s*:\s*true\b", text)),
}

failed = [name for name, passed in checks.items() if not passed]

for name, passed in checks.items():
    print(f"{'PASS' if passed else 'FAIL'} — {name}")

if failed:
    raise SystemExit(
        "\nERROR: V10 validation failed:\n- " + "\n- ".join(failed)
    )
PY

echo
echo "V10 preparation complete."
echo
echo "New config script:"
echo "  $NEW_CONFIG"
echo
echo "New pool script:"
echo "  $NEW_POOL"
echo
echo "New environment file:"
echo "  $NEW_ENV"
echo
echo "New deployment manifest:"
echo "  $NEW_MANIFEST"
echo
echo "The existing V8/V9 deployment was not modified."
