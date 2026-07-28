#!/usr/bin/env python3
"""
RBVR consolidated security hardening patch.

Applies one fail-closed change to the current 2026-07-27 snapshot:
- Enforces Sentinel v1 and v2 before process_fees mutates accounting.
- Enforces Sentinel v1 and v2 again after mutation so any invariant failure
  aborts the Solana transaction atomically.

The script is intentionally narrow. It does not add new architecture or alter
Beavernomics, Waterfall, Dam, Founder, Company, or release formulas.
"""

from pathlib import Path
import hashlib
import shutil
import sys
from datetime import datetime, timezone

ROOT = Path.cwd()
TARGET = ROOT / "programs/treasury-router/src/instructions/process_fees.rs"
EXPECTED_SHA256 = "1fe676071fbeb2399856e9fc0a22b4fe1cb52fd2c8bab7833334b56603da8972"

def die(message: str) -> None:
    print(f"RBVR HARDENING PATCH: FAIL\n{message}", file=sys.stderr)
    raise SystemExit(1)

if not TARGET.is_file():
    die(f"Run this script from the repository root. Missing: {TARGET}")

source = TARGET.read_text(encoding="utf-8")
current_hash = hashlib.sha256(TARGET.read_bytes()).hexdigest()

already_patched = (
    "pre_linkage_report = sentinel::evaluate_linkage" in source
    and "post_linkage_report = sentinel::evaluate_linkage" in source
)

if already_patched:
    print("RBVR HARDENING PATCH: ALREADY APPLIED")
    raise SystemExit(0)

if current_hash != EXPECTED_SHA256:
    die(
        "process_fees.rs does not match the reviewed snapshot.\n"
        f"Expected SHA-256: {EXPECTED_SHA256}\n"
        f"Actual SHA-256:   {current_hash}\n"
        "No files were changed."
    )

old_import = (
    "engines::{beaver_score, buyback, company, dam, founder, liquidity, "
    "reserve, waterfall},"
)
new_import = """engines::{
        beaver_score, buyback, company, dam, founder, liquidity, reserve, sentinel, waterfall,
    },"""

if old_import not in source:
    die("Expected engines import was not found. No files were changed.")

source = source.replace(old_import, new_import, 1)

old_start = """    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

"""

new_start = """    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    // Fail closed before any accounting mutation. Sentinel V2 verifies the
    // exact locked 30/20/20/20/10 configuration, immutable configuration flag,
    // canonical protocol linkage, and lifetime-allocation conservation.
    let pre_linkage_report = sentinel::evaluate_linkage(
        ctx.accounts.protocol_state.key(),
        &ctx.accounts.protocol_config,
        &ctx.accounts.treasury,
    );

    require!(
        pre_linkage_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    let pre_sentinel_report =
        sentinel::evaluate(&ctx.accounts.protocol_config, &ctx.accounts.treasury)?;

    require!(
        pre_sentinel_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

"""

if old_start not in source:
    die("Expected process_fees entry guard was not found. No files were changed.")

source = source.replace(old_start, new_start, 1)

old_end = """    ctx.accounts.protocol_state.beaver_score = beaver_score_evaluation.total_score;

    msg!("Beavernomics fee accounting completed");
"""

new_end = """    ctx.accounts.protocol_state.beaver_score = beaver_score_evaluation.total_score;

    // Re-run both Sentinel layers after every state mutation. A failed
    // invariant aborts the transaction atomically and rolls all changes back.
    let post_linkage_report = sentinel::evaluate_linkage(
        ctx.accounts.protocol_state.key(),
        &ctx.accounts.protocol_config,
        &ctx.accounts.treasury,
    );

    require!(
        post_linkage_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    let post_sentinel_report =
        sentinel::evaluate(&ctx.accounts.protocol_config, &ctx.accounts.treasury)?;

    require!(
        post_sentinel_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    msg!("Beavernomics fee accounting completed");
"""

if old_end not in source:
    die("Expected process_fees completion point was not found. No files were changed.")

source = source.replace(old_end, new_end, 1)

stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup_dir = ROOT / f".rbvr-security-hardening-backup-{stamp}"
backup_path = backup_dir / TARGET.relative_to(ROOT)
backup_path.parent.mkdir(parents=True, exist_ok=True)
shutil.copy2(TARGET, backup_path)

TARGET.write_text(source, encoding="utf-8")

print("RBVR HARDENING PATCH: APPLIED")
print(f"Backup: {backup_path}")
print(f"Modified: {TARGET}")
print("Next command: bash rbvr-security-gate.sh")
