#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(pwd)"
SRC="$ROOT/programs/treasury-router/src"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
BACKUP="$ROOT/.autonomous-release-backup-v2-$STAMP"

FILES=(
  "$SRC/lib.rs"
  "$SRC/engines/execution_guard.rs"
  "$SRC/instructions/reserve.rs"
  "$SRC/instructions/buyback.rs"
  "$SRC/instructions/liquidity.rs"
  "$SRC/instructions/company.rs"
  "$SRC/instructions/founder.rs"
  "$ROOT/tests/rbvr_protocol.ts"
)

mkdir -p "$BACKUP"

restore_backup() {
    echo
    echo "ERROR: autonomous release conversion failed."
    echo "Restoring original files..."

    for file in "${FILES[@]}"; do
        relative="${file#$ROOT/}"
        if [[ -f "$BACKUP/$relative" ]]; then
            cp "$BACKUP/$relative" "$file"
        fi
    done

    echo "Original files restored."
}

trap restore_backup ERR

echo "===== CREATING BACKUP ====="

for file in "${FILES[@]}"; do
    if [[ ! -f "$file" ]]; then
        echo "Required file missing: $file"
        exit 1
    fi

    relative="${file#$ROOT/}"
    mkdir -p "$BACKUP/$(dirname "$relative")"
    cp "$file" "$BACKUP/$relative"
done

echo "Backup: $BACKUP"

python3 <<'PY'
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path.cwd()
SRC = ROOT / "programs/treasury-router/src"

release_files = {
    "reserve.rs": "Reserve",
    "buyback.rs": "BuybackBurn",
    "liquidity.rs": "Liquidity",
    "company.rs": "Company",
    "founder.rs": "Founder",
}


def required_sub(
    text: str,
    pattern: str,
    replacement: str,
    description: str,
    *,
    count: int = 1,
    flags: int = 0,
) -> str:
    updated, replacements = re.subn(
        pattern,
        replacement,
        text,
        count=count,
        flags=flags,
    )

    if replacements != count:
        raise RuntimeError(
            f"{description}: expected {count} replacement(s), "
            f"found {replacements}"
        )

    return updated


# ================================================================
# 1. Add deterministic autonomous authorization
# ================================================================

guard_path = SRC / "engines/execution_guard.rs"
guard = guard_path.read_text()

if "pub fn authorize_autonomous_release(" not in guard:
    insertion_marker = (
        "/// Ensures fee accounting has never allocated more than was received."
    )

    if insertion_marker not in guard:
        raise RuntimeError(
            "execution_guard.rs: insertion marker not found"
        )

    autonomous_function = r'''
/// Computes the exact release amount permitted by RBVR's complete
/// on-chain security system.
///
/// The transaction caller supplies no amount and cannot influence the
/// transfer value. The release amount is derived from the selected
/// bucket's pending balance, Waterfall stage, Beaver Score and adaptive Dam.
pub fn authorize_autonomous_release(
    protocol: &ProtocolState,
    treasury: &TreasuryState,
    bucket: ReleaseBucket,
) -> Result<ExecutionAuthorization> {
    require!(
        !protocol.paused,
        TreasuryRouterError::ProtocolPaused
    );

    validate_accounting_integrity(treasury)?;

    require!(
        treasury.execution_accounting_is_valid(),
        TreasuryRouterError::AccountingInvariantViolation
    );

    if bucket == ReleaseBucket::BuybackBurn {
        require!(
            !treasury.buybacks_paused,
            TreasuryRouterError::BuybacksPaused
        );
    }

    let waterfall_evaluation = waterfall::evaluate(treasury)?;

    let pre_dam_health_score = beaver_score::pre_dam_health_score(
        treasury,
        waterfall_evaluation.reserve_ratio_bps,
        waterfall_evaluation.stage,
    )?;

    let dam_evaluation = dam::evaluate_adaptive(
        waterfall_evaluation.stage,
        pre_dam_health_score,
    );

    require!(
        dam_evaluation.release_bps <= 10_000,
        TreasuryRouterError::InvalidDamReleaseRate
    );

    require!(
        dam_evaluation.level != DamLevel::Filling,
        TreasuryRouterError::DamClosed
    );

    let pending_balance = pending_balance(treasury, bucket);

    require!(
        pending_balance > 0,
        TreasuryRouterError::InsufficientPendingBalance
    );

    let maximum_release = calculate_maximum_release(
        pending_balance,
        dam_evaluation.release_bps,
    )?;

    require!(
        maximum_release > 0,
        TreasuryRouterError::ReleaseLimitExceeded
    );

    Ok(ExecutionAuthorization {
        bucket,
        requested_amount: maximum_release,
        pending_balance,
        maximum_release,
        waterfall_stage: waterfall_evaluation.stage,
        dam_level: dam_evaluation.level,
        release_bps: dam_evaluation.release_bps,
        reserve_ratio_bps: waterfall_evaluation.reserve_ratio_bps,
    })
}

'''

    guard = guard.replace(
        insertion_marker,
        autonomous_function + insertion_marker,
        1,
    )

    guard_path.write_text(guard)


# ================================================================
# 2. Convert the five release instructions
# ================================================================

for filename, bucket in release_files.items():
    path = SRC / "instructions" / filename
    text = path.read_text()

    # Remove only the release instruction's authority relationship.
    text = required_sub(
        text,
        r"^[ \t]*has_one\s*=\s*authority,\s*\n",
        "",
        f"{filename}: remove has_one authority",
        flags=re.MULTILINE,
    )

    # Replace the guard import, regardless of spacing/layout.
    text = required_sub(
        text,
        r"execution_guard::\{\s*"
        r"authorize_release\s*,\s*"
        r"ReleaseBucket\s*"
        r"\}",
        "execution_guard::{authorize_autonomous_release, ReleaseBucket}",
        f"{filename}: replace execution guard import",
        flags=re.MULTILINE,
    )

    # Remove amount from any handler formatting:
    #
    # pub fn handler(
    #     ctx: Context<...>,
    #     amount: u64,
    # ) -> Result<()> {
    #
    # or:
    #
    # pub fn handler(ctx: Context<...>, amount: u64) -> Result<()> {
    handler_pattern = (
        r"pub\s+fn\s+handler\s*\(\s*"
        r"ctx\s*:\s*Context<(?P<context>[A-Za-z0-9_]+)>\s*,\s*"
        r"amount\s*:\s*u64\s*,?\s*"
        r"\)\s*->\s*Result<\(\)>\s*\{"
    )

    handler_replacement = (
        r"pub fn handler("
        r"ctx: Context<\g<context>>"
        r") -> Result<()> {"
    )

    text = required_sub(
        text,
        handler_pattern,
        handler_replacement,
        f"{filename}: remove handler amount",
        flags=re.MULTILINE | re.DOTALL,
    )

    # Replace authorize_release(..., amount) with autonomous calculation.
    authorize_pattern = (
        r"let\s+authorization\s*=\s*authorize_release\s*\(\s*"
        r"&ctx\.accounts\.protocol_state\s*,\s*"
        r"&ctx\.accounts\.treasury\s*,\s*"
        rf"ReleaseBucket::{bucket}\s*,\s*"
        r"amount\s*,?\s*"
        r"\)\s*\?;"
    )

    authorize_replacement = (
        "let authorization = authorize_autonomous_release(\n"
        "        &ctx.accounts.protocol_state,\n"
        "        &ctx.accounts.treasury,\n"
        f"        ReleaseBucket::{bucket},\n"
        "    )?;\n\n"
        "    let amount = authorization.maximum_release;"
    )

    text = required_sub(
        text,
        authorize_pattern,
        authorize_replacement,
        f"{filename}: replace authorize_release",
        flags=re.MULTILINE | re.DOTALL,
    )

    # Keep a signer as the fee-paying transaction caller, but clarify that
    # it has no protocol privilege.
    signer_pattern = (
        r"(?P<indent>^[ \t]*)"
        r"pub\s+authority\s*:\s*Signer<'info>,"
    )

    signer_replacement = (
        r"\g<indent>/// Permissionless transaction caller and fee payer.\n"
        r"\g<indent>/// This signer does not need to match "
        r"ProtocolState.authority.\n"
        r"\g<indent>pub authority: Signer<'info>,"
    )

    text = required_sub(
        text,
        signer_pattern,
        signer_replacement,
        f"{filename}: annotate caller signer",
        flags=re.MULTILINE,
    )

    path.write_text(text)


# ================================================================
# 3. Remove amount arguments from public entrypoints
# ================================================================

lib_path = SRC / "lib.rs"
lib = lib_path.read_text()

entrypoints = {
    "reserve": "AuthorizeReserveExecution",
    "buyback": "AuthorizeBuybackExecution",
    "liquidity": "AuthorizeLiquidityExecution",
    "founder": "AuthorizeFounderExecution",
    "company": "AuthorizeCompanyExecution",
}

for name, context in entrypoints.items():
    pattern = (
        rf"pub\s+fn\s+authorize_{name}_execution\s*\(\s*"
        rf"ctx\s*:\s*Context<{context}>\s*,\s*"
        rf"amount\s*:\s*u64\s*,?\s*"
        rf"\)\s*->\s*Result<\(\)>\s*\{{\s*"
        rf"instructions::{name}::handler\s*\(\s*ctx\s*,\s*amount\s*\)\s*"
        rf"\}}"
    )

    replacement = (
        f"pub fn authorize_{name}_execution(\n"
        f"        ctx: Context<{context}>,\n"
        f"    ) -> Result<()> {{\n"
        f"        instructions::{name}::handler(ctx)\n"
        f"    }}"
    )

    lib = required_sub(
        lib,
        pattern,
        replacement,
        f"lib.rs: convert authorize_{name}_execution",
        flags=re.MULTILINE | re.DOTALL,
    )

lib_path.write_text(lib)


# ================================================================
# 4. Update TypeScript integration calls
# ================================================================

tests_path = ROOT / "tests/rbvr_protocol.ts"
tests = tests_path.read_text()

methods = [
    "authorizeReserveExecution",
    "authorizeBuybackExecution",
    "authorizeLiquidityExecution",
    "authorizeFounderExecution",
    "authorizeCompanyExecution",
]

for method in methods:
    # Match a single ordinary Anchor method argument such as:
    # .authorizeReserveExecution(new anchor.BN(100))
    #
    # Balanced nested parentheses are handled for the common new BN form.
    patterns = [
        rf"\.{method}\(\s*new\s+anchor\.BN\([^)]*\)\s*\)",
        rf"\.{method}\(\s*new\s+BN\([^)]*\)\s*\)",
        rf"\.{method}\(\s*[A-Za-z0-9_.$]+\s*\)",
    ]

    total = 0

    for pattern in patterns:
        tests, count = re.subn(
            pattern,
            f".{method}()",
            tests,
            count=1 if total == 0 else 0,
            flags=re.MULTILINE,
        )
        total += count

        if total:
            break

    if total != 1:
        raise RuntimeError(
            f"rbvr_protocol.ts: expected one {method}(amount) call; "
            f"found {total}"
        )

tests_path.write_text(tests)

print("Autonomous release conversion applied successfully.")
PY

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== SOURCE VERIFICATION ====="

for file in reserve buyback liquidity company founder; do
    path="$SRC/instructions/$file.rs"

    if grep -qE \
        'has_one[[:space:]]*=[[:space:]]*authority' \
        "$path"; then
        echo "ERROR: authority relationship remains in $path"
        exit 1
    fi

    if grep -qE \
        'pub fn handler.*amount[[:space:]]*:[[:space:]]*u64' \
        "$path"; then
        echo "ERROR: handler amount remains in $path"
        exit 1
    fi

    if ! grep -q 'authorize_autonomous_release' "$path"; then
        echo "ERROR: autonomous guard missing in $path"
        exit 1
    fi
done

if grep -nE \
    'authorize_(reserve|buyback|liquidity|company|founder)_execution[^}]*amount[[:space:]]*:[[:space:]]*u64' \
    "$SRC/lib.rs"; then
    echo "ERROR: release amount remains in public API"
    exit 1
fi

echo "Source conversion checks passed."

echo
echo "===== SHOW MODIFIED HANDLERS ====="

for file in reserve buyback liquidity company founder; do
    echo
    echo "----- $file.rs -----"
    grep -n -A18 -B5 \
        'pub fn handler' \
        "$SRC/instructions/$file.rs"
done

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
echo "===== RUST TESTS ====="
cargo test --workspace --all-features

echo
echo "===== ANCHOR BUILD ====="
anchor build

echo
echo "===== INTEGRATION TESTS ====="
anchor test --validator legacy

echo
echo "===== FUZZ BUILD ====="
cargo +nightly fuzz build process_release

echo
echo "===== BOUNDED FUZZ CAMPAIGN ====="
cargo +nightly fuzz run process_release -- \
    -runs=1000000 \
    -max_len=160 \
    -timeout=10 \
    -print_final_stats=1

echo
echo "===== AUTHORITY INVENTORY ====="

grep -RniE \
    --include='*.rs' \
    'has_one[[:space:]]*=[[:space:]]*authority|authorize_autonomous_release|pub authority: Signer' \
    "$SRC/instructions" \
    "$SRC/engines/execution_guard.rs" \
    || true

echo
echo "===== DIFF SUMMARY ====="
git diff --stat

echo
echo "===== FINAL STATUS ====="
git status --short

trap - ERR

echo
echo "============================================================"
echo "AUTONOMOUS RELEASE CONVERSION PASSED"
echo "============================================================"
echo
echo "Backup retained at:"
echo "$BACKUP"
