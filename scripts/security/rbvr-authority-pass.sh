#!/usr/bin/env bash
set -euo pipefail

ROOT="$(pwd)"
SRC="programs/treasury-router/src"
REPORT="$ROOT/RBVR-AUTHORITY-MAP.md"

echo "===== BACKING UP FILE ====="
cp \
  "$SRC/instructions/process_fees.rs" \
  "$SRC/instructions/process_fees.rs.before-clippy-fix"

echo "===== FIXING CONFIRMED CLIPPY WARNINGS ====="
python3 <<'PY'
from pathlib import Path

path = Path("programs/treasury-router/src/instructions/process_fees.rs")
text = path.read_text()

old_1 = "&*ctx.accounts.treasury"
new_1 = "&ctx.accounts.treasury"

count = text.count(old_1)

if count != 2:
    raise SystemExit(
        f"Expected exactly 2 occurrences of {old_1!r}, found {count}. "
        "No changes were written."
    )

path.write_text(text.replace(old_1, new_1))
print(f"Replaced {count} explicit auto-dereferences.")
PY

echo
echo "===== FORMAT ====="
cargo fmt --all -- --check

echo
echo "===== STRICT CLIPPY ====="
cargo clippy \
  --workspace \
  --all-targets \
  --all-features \
  -- \
  -D warnings \
  -A unexpected_cfgs

echo
echo "===== PRODUCTION TESTS ====="
cargo test --workspace --all-features

echo
echo "===== FUZZ BUILD ====="
cargo +nightly fuzz build process_release

echo
echo "===== BOUNDED FUZZ RUN ====="
cargo +nightly fuzz run process_release -- \
  -runs=1000000 \
  -max_len=160 \
  -timeout=10 \
  -print_final_stats=1

echo
echo "===== GENERATING AUTHORITY MAP ====="

cat > "$REPORT" <<EOF
# RBVR Authority and Autonomy Map

- Generated: $(date -Iseconds)
- Branch: $(git branch --show-current)
- Commit: $(git rev-parse HEAD)

## Security objective

RBVR intends to use fixed destinations, deterministic accounting and
one-time initialization, with administrative authority removed only after
deployment, validation and treasury-control verification.

Removing signer checks without replacing them with deterministic execution
conditions could expose treasury releases to arbitrary callers.

## Protocol authority definition

EOF

grep -RniE \
  --include='*.rs' \
  'pub authority: Pubkey|protocol_state\.authority[[:space:]]*=|has_one[[:space:]]*=[[:space:]]*authority|pub authority: Signer' \
  "$SRC" >> "$REPORT" || true

cat >> "$REPORT" <<'EOF'

## Public program entrypoints

EOF

grep -nE 'pub fn [a-zA-Z0-9_]+\(' \
  "$SRC/lib.rs" >> "$REPORT" || true

cat >> "$REPORT" <<'EOF'

## Authority-gated instructions

The following files contain both an authority signer and an Anchor
`has_one = authority` relationship.

EOF

for file in "$SRC"/instructions/*.rs; do
    if grep -q 'has_one[[:space:]]*=[[:space:]]*authority' "$file" &&
       grep -q "Signer<'info>" "$file"; then
        {
            echo
            echo "### $(basename "$file")"
            echo
            echo '```text'
            grep -nE \
              'pub struct|has_one[[:space:]]*=[[:space:]]*authority|pub authority: Signer|pub fn [a-zA-Z0-9_]+\(' \
              "$file" || true
            echo '```'
        } >> "$REPORT"
    fi
done

cat >> "$REPORT" <<'EOF'

## Treasury-signing CPI paths

These instructions use the treasury PDA as the SPL-token transfer authority.
The caller authority and the treasury PDA signer are separate controls:
the user signs the instruction, while the program signs the token CPI using
treasury PDA seeds.

EOF

grep -RniE \
  --include='*.rs' \
  'new_with_signer|treasury_signer_seeds|token::transfer|transfer_checked' \
  "$SRC/instructions" >> "$REPORT" || true

cat >> "$REPORT" <<'EOF'

## Initialization paths

EOF

grep -RniE \
  --include='initialize*.rs' \
  'pub struct|init,|init_if_needed|has_one[[:space:]]*=[[:space:]]*authority|pub authority: Signer|pub fn' \
  "$SRC/instructions" >> "$REPORT" || true

cat >> "$REPORT" <<'EOF'

## Mutable configuration and authority-changing paths

EOF

grep -RniE \
  --include='*.rs' \
  'set_authority|update_authority|new_authority|configuration_updates_enabled|realloc|close[[:space:]]*=|upgrade' \
  "$SRC" >> "$REPORT" || true

cat >> "$REPORT" <<'EOF'

## Preliminary classification

### Initialization-only authority

Likely candidates:

- initialize
- initialize_protocol_config
- initialize_treasury
- initialize_founder
- initialize_company
- initialize_execution_config

These should remain authority-controlled during deployment. They should
become unusable after their PDA accounts have been initialized.

### Deposit authority

`deposit_settlement` currently links the depositor to the protocol authority.
That means ordinary third parties cannot fund the settlement vault directly.

This may be intentional during testing, but it is not necessary for a fully
autonomous protocol if deposits should be open to anyone. Its source account
already has to be owned by the transaction signer.

### Fee processing

`process_fees` does not appear in the authority-gated list from the previous
automated audit. This is consistent with deterministic permissionless
processing, provided its state and account constraints fully prevent caller
discretion.

### Release authority

The following release instructions currently require the protocol authority:

- reserve
- buyback
- liquidity
- company
- founder

These checks cannot simply be deleted. A permissionless caller must not be
able to select arbitrary release timing or amounts.

Before converting these instructions to permissionless execution, RBVR needs
one of these deterministic models:

1. Anyone may trigger a release, but the program calculates the exact amount.
2. Anyone may trigger a release only after a fixed interval and only up to an
   on-chain deterministic amount.
3. A dedicated automation signer triggers releases, with no power to change
   destinations, allocations, caps or accounting rules.
4. Release authority remains temporarily controlled until the complete
   automation and fail-closed rules are deployed and tested.

## Recommended current decision

Keep release authority temporarily.

Do not revoke or replace it until:

- release amounts are computed entirely on-chain;
- time and period conditions are enforced on-chain;
- all destinations are immutable;
- configuration updates are permanently disabled;
- the program upgrade authority has an explicit final disposition;
- permissionless callers cannot accelerate founder or company compensation;
- repeat calls cannot release the same pending balance twice;
- the complete autonomous path has integration and fuzz coverage.

EOF

echo
echo "===== DIFF ====="
git diff -- \
  "$SRC/instructions/process_fees.rs"

echo
echo "===== AUTHORITY MAP ====="
cat "$REPORT"

echo
echo "===== FINAL STATUS ====="
git status --short

echo
echo "Completed successfully."
