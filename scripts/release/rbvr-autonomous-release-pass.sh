#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(pwd)"
SRC="$ROOT/programs/treasury-router/src"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
BACKUP="$ROOT/.autonomous-release-backup-$STAMP"

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
    echo "Restoring the original files..."

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


def replace_exact(
    path: Path,
    old: str,
    new: str,
    expected: int = 1,
) -> None:
    text = path.read_text()
    count = text.count(old)

    if count != expected:
        raise RuntimeError(
            f"{path}: expected {expected} occurrence(s) of "
            f"{old!r}, found {count}"
        )

    path.write_text(text.replace(old, new))


def replace_regex(
    path: Path,
    pattern: str,
    replacement: str,
    expected: int = 1,
    flags: int = 0,
) -> None:
    text = path.read_text()
    updated, count = re.subn(pattern, replacement, text, flags=flags)

    if count != expected:
        raise RuntimeError(
            f"{path}: expected {expected} regex replacement(s), "
            f"found {count}. Pattern: {pattern!r}"
        )

    path.write_text(updated)


# ------------------------------------------------------------------
# 1. Add a deterministic autonomous authorization function.
#
# Existing authorize_release remains intact for its existing unit tests,
# fuzzing target and security regression coverage.
# ------------------------------------------------------------------

guard = SRC / "engines/execution_guard.rs"
guard_text = guard.read_text()

if "pub fn authorize_autonomous_release(" in guard_text:
    raise RuntimeError(
        "authorize_autonomous_release already exists. "
        "No duplicate function will be inserted."
    )

marker = """/// Ensures fee accounting has never allocated more than was received.
///
"""

if marker not in guard_text:
    raise RuntimeError(
        "Could not find the execution-guard insertion point."
    )

autonomous_function = r'''
/// Computes the exact release amount permitted by the complete RBVR
/// security system.
///
/// The transaction caller supplies no amount and cannot influence the
/// resulting transfer. The amount is derived from the selected bucket's
/// pending balance, current Waterfall stage, Beaver Score and adaptive Dam.
///
/// Account ownership, PDA derivation, settlement mint, destination routing
/// and treasury-vault constraints remain enforced by the calling Anchor
/// instruction.
pub fn authorize_autonomous_release(
    protocol: &ProtocolState,
    treasury: &TreasuryState,
    bucket: ReleaseBucket,
) -> Result<ExecutionAuthorization> {
    require!(!protocol.paused, TreasuryRouterError::ProtocolPaused);

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

    let dam_evaluation =
        dam::evaluate_adaptive(waterfall_evaluation.stage, pre_dam_health_score);

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

    let maximum_release =
        calculate_maximum_release(pending_balance, dam_evaluation.release_bps)?;

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

guard.write_text(
    guard_text.replace(marker, autonomous_function + marker, 1)
)

# ------------------------------------------------------------------
# 2. Convert each execution account from privileged authority to an
# arbitrary transaction caller.
#
# We retain a Signer named authority for compatibility with current
# clients/events, but remove has_one = authority. It is therefore only
# the transaction fee payer/trigger, not a protocol administrator.
# ------------------------------------------------------------------

for filename, bucket in release_files.items():
    path = SRC / "instructions" / filename
    text = path.read_text()

    has_one_count = len(
        re.findall(r"^\s*has_one\s*=\s*authority,\s*$", text, re.MULTILINE)
    )

    if has_one_count != 1:
        raise RuntimeError(
            f"{path}: expected one 'has_one = authority,' constraint, "
            f"found {has_one_count}"
        )

    text = re.sub(
        r"^\s*has_one\s*=\s*authority,\s*\n",
        "",
        text,
        count=1,
        flags=re.MULTILINE,
    )

    text, import_count = re.subn(
        r"execution_guard::\{\s*authorize_release\s*,\s*ReleaseBucket\s*\}",
        "execution_guard::{authorize_autonomous_release, ReleaseBucket}",
        text,
        count=1,
    )

    if import_count != 1:
        raise RuntimeError(
            f"{path}: could not replace the execution-guard import."
        )

    handler_pattern = (
        r"pub fn handler\("
        r"ctx: Context<(?P<context>[A-Za-z0-9_]+)>,\s*"
        r"amount: u64,\s*"
        r"\) -> Result<\(\)> \{"
    )

    text, handler_count = re.subn(
        handler_pattern,
        r"pub fn handler(ctx: Context<\g<context>>) -> Result<()> {",
        text,
        count=1,
        flags=re.MULTILINE,
    )

    if handler_count != 1:
        raise RuntimeError(
            f"{path}: could not remove the caller-supplied amount "
            "from the handler."
        )

    call_pattern = (
        r"let authorization = authorize_release\(\s*"
        r"&ctx\.accounts\.protocol_state,\s*"
        r"&ctx\.accounts\.treasury,\s*"
        rf"ReleaseBucket::{bucket},\s*"
        r"amount,\s*"
        r"\)\?;"
    )

    call_replacement = (
        "let authorization = authorize_autonomous_release(\n"
        "        &ctx.accounts.protocol_state,\n"
        "        &ctx.accounts.treasury,\n"
        f"        ReleaseBucket::{bucket},\n"
        "    )?;\n\n"
        "    let amount = authorization.maximum_release;"
    )

    text, call_count = re.subn(
        call_pattern,
        call_replacement,
        text,
        count=1,
        flags=re.MULTILINE,
    )

    if call_count != 1:
        raise RuntimeError(
            f"{path}: could not replace the authorize_release call."
        )

    # Clarify that this signer is not privileged.
    signer_pattern = r"(\s+)pub authority: Signer<'info>,"

    signer_replacement = (
        r"\1/// Permissionless transaction caller and fee payer.\n"
        r"\1/// This signer is not required to match ProtocolState.authority.\n"
        r"\1pub authority: Signer<'info>,"
    )

    text, signer_count = re.subn(
        signer_pattern,
        signer_replacement,
        text,
        count=1,
    )

    if signer_count != 1:
        raise RuntimeError(
            f"{path}: could not annotate the permissionless caller."
        )

    path.write_text(text)


# ------------------------------------------------------------------
# 3. Remove amount arguments from the five public entrypoints.
#
# Existing instruction names are retained to avoid an unnecessary IDL and
# client naming migration during this security change.
# ------------------------------------------------------------------

lib = SRC / "lib.rs"
lib_text = lib.read_text()

entrypoints = {
    "reserve": "AuthorizeReserveExecution",
    "buyback": "AuthorizeBuybackExecution",
    "liquidity": "AuthorizeLiquidityExecution",
    "founder": "AuthorizeFounderExecution",
    "company": "AuthorizeCompanyExecution",
}

for name, context in entrypoints.items():
    old_pattern = (
        rf"    pub fn authorize_{name}_execution\(\s*"
        rf"ctx: Context<{context}>,\s*"
        rf"amount: u64,\s*"
        rf"\) -> Result<\(\)> \{{\s*"
        rf"instructions::{name}::handler\(ctx, amount\)\s*"
        rf"\}}"
    )

    new_block = (
        f"    pub fn authorize_{name}_execution(\n"
        f"        ctx: Context<{context}>,\n"
        f"    ) -> Result<()> {{\n"
        f"        instructions::{name}::handler(ctx)\n"
        f"    }}"
    )

    lib_text, count = re.subn(
        old_pattern,
        new_block,
        lib_text,
        count=1,
        flags=re.MULTILINE,
    )

    if count != 1:
        raise RuntimeError(
            f"{lib}: could not convert authorize_{name}_execution."
        )

lib.write_text(lib_text)


# ------------------------------------------------------------------
# 4. Update integration-test method calls.
#
# Account mappings remain compatible because the account is still named
# authority. Only the caller-supplied release amount is removed.
# ------------------------------------------------------------------

tests = ROOT / "tests/rbvr_protocol.ts"
test_text = tests.read_text()

method_names = [
    "authorizeReserveExecution",
    "authorizeBuybackExecution",
    "authorizeLiquidityExecution",
    "authorizeFounderExecution",
    "authorizeCompanyExecution",
]

for method in method_names:
    pattern = rf"\.{method}\(\s*[^()]*?\s*\)"
    replacement = f".{method}()"

    test_text, count = re.subn(
        pattern,
        replacement,
        test_text,
        count=1,
        flags=re.MULTILINE,
    )

    if count != 1:
        raise RuntimeError(
            f"{tests}: expected exactly one {method}(amount) call, "
            f"found {count}"
        )

tests.write_text(test_text)

print("Autonomous release source conversion completed.")
PY

echo
echo "===== FORMAT ====="
cargo fmt --all

echo
echo "===== VERIFY PRIVILEGED RELEASE CONSTRAINTS REMOVED ====="

for file in reserve buyback liquidity company founder; do
    path="$SRC/instructions/$file.rs"

    if grep -qE 'has_one[[:space:]]*=[[:space:]]*authority' "$path"; then
        echo "ERROR: privileged authority constraint remains in $path"
        exit 1
    fi
done

echo "No privileged release-authority constraints remain."

echo
echo "===== VERIFY RELEASE AMOUNTS REMOVED FROM PUBLIC API ====="

if grep -nE \
    'authorize_(reserve|buyback|liquidity|company|founder)_execution.*amount' \
    "$SRC/lib.rs"; then
    echo "ERROR: a caller-supplied release amount remains in lib.rs"
    exit 1
fi

for file in reserve buyback liquidity company founder; do
    if grep -nE \
        'pub fn handler\(.*amount: u64' \
        "$SRC/instructions/$file.rs"; then
        echo "ERROR: caller-supplied amount remains in $file.rs"
        exit 1
    fi
done

echo "Caller-supplied release amounts removed."

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
anchor test

echo
echo "===== RELEASE FUZZ BUILD ====="
cargo +nightly fuzz build process_release

echo
echo "===== BOUNDED RELEASE FUZZING ====="
cargo +nightly fuzz run process_release -- \
    -runs=1000000 \
    -max_len=160 \
    -timeout=10 \
    -print_final_stats=1

echo
echo "===== AUTONOMY SECURITY CHECK ====="

grep -RniE \
    --include='*.rs' \
    'authorize_autonomous_release|has_one[[:space:]]*=[[:space:]]*authority|pub authority: Signer' \
    "$SRC/instructions" \
    "$SRC/engines/execution_guard.rs"

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
echo "Five release paths are now permissionless triggers."
echo "The caller supplies no release amount."
echo "The protocol computes the exact maximum permitted release."
echo "Initialization and settlement deposits remain authority-controlled."
echo
echo "Backup retained at:"
echo "$BACKUP"
