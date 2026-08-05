from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path.cwd()
SRC = ROOT / "programs/treasury-router/src"
REPORT = ROOT / "RBVR-AUTHORITY-LOCKDOWN-AUDIT.md"

PROGRAM_ID = "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3"

if not SRC.exists():
    raise RuntimeError(f"Missing program source directory: {SRC}")

rust_files = sorted(SRC.rglob("*.rs"))

INITIALIZATION_FILES = {
    "initialize.rs",
    "initialize_protocol_config.rs",
    "initialize_treasury.rs",
    "initialize_founder.rs",
    "initialize_company.rs",
    "initialize_execution_config.rs",
}

EXPECTED_OPERATIONAL_SIGNER_FILES = {
    "deposit_settlement.rs",
}

RELEASE_FILES = {
    "reserve.rs",
    "liquidity.rs",
    "company.rs",
    "founder.rs",
    "buyback.rs",
}

danger_patterns = {
    "Signer account": re.compile(r"\bSigner\s*<'info>"),
    "Signer constraint": re.compile(r"\bsigner\b"),
    "Authority comparison": re.compile(
        r"(authority\s*==|authority\s*!=|key\(\)\s*==\s*.*authority|"
        r"key\(\)\s*!=\s*.*authority|has_one\s*=\s*authority)"
    ),
    "Authority assignment": re.compile(
        r"\.\s*authority\s*=|authority\s*:\s*ctx\.accounts"
    ),
    "Configuration mutation": re.compile(
        r"(protocol_config|execution_config|protocol_state)"
        r"\.[A-Za-z_][A-Za-z0-9_]*\s*="
    ),
    "Pause mutation": re.compile(
        r"(paused|pause|buybacks_paused)\s*="
    ),
    "Destination mutation": re.compile(
        r"(reserve_destination|buyback_destination|liquidity_destination|"
        r"company_destination|founder_destination)\s*="
    ),
    "Upgrade reference": re.compile(
        r"(upgrade_authority|set_upgrade_authority|programdata)",
        re.IGNORECASE,
    ),
}

def run_command(command: list[str]) -> tuple[int, str]:
    try:
        result = subprocess.run(
            command,
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            timeout=30,
            check=False,
        )
        return result.returncode, result.stdout.strip()
    except (FileNotFoundError, subprocess.TimeoutExpired) as error:
        return 127, str(error)

def find_context(lines: list[str], index: int, radius: int = 5) -> str:
    start = max(0, index - radius)
    end = min(len(lines), index + radius + 1)

    rendered = []

    for line_number in range(start, end):
        marker = ">" if line_number == index else " "
        rendered.append(
            f"{marker} {line_number + 1:4}: {lines[line_number]}"
        )

    return "\n".join(rendered)

def classify_signer_file(path: Path) -> tuple[str, str]:
    name = path.name

    if name in INITIALIZATION_FILES:
        return (
            "Initialization-only",
            "Expected only before the protocol is permanently locked.",
        )

    if name in EXPECTED_OPERATIONAL_SIGNER_FILES:
        return (
            "Operational depositor",
            "May be required to authorize movement from the depositor's token account.",
        )

    if name in RELEASE_FILES:
        return (
            "CRITICAL FAILURE",
            "Release instructions must remain permissionless.",
        )

    return (
        "Requires review",
        "Signer exists outside the currently recognized initialization/deposit paths.",
    )

findings: list[dict[str, object]] = []
signer_inventory: list[dict[str, str]] = []
mutable_accounts: list[dict[str, str]] = []

for path in rust_files:
    relative = path.relative_to(ROOT)
    text = path.read_text()
    lines = text.splitlines()

    # Locate account structs.
    struct_pattern = re.compile(
        r"#\[derive\(Accounts\)\]\s*"
        r"pub\s+struct\s+([A-Za-z_][A-Za-z0-9_]*)\s*<'info>\s*\{",
        re.DOTALL,
    )

    for struct_match in struct_pattern.finditer(text):
        struct_name = struct_match.group(1)
        brace_start = text.find("{", struct_match.start())

        depth = 0
        struct_end = None

        for position in range(brace_start, len(text)):
            character = text[position]

            if character == "{":
                depth += 1
            elif character == "}":
                depth -= 1

                if depth == 0:
                    struct_end = position + 1
                    break

        if struct_end is None:
            continue

        struct_body = text[brace_start:struct_end]

        for signer_match in re.finditer(
            r"pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*Signer\s*<'info>",
            struct_body,
        ):
            account_name = signer_match.group(1)
            classification, explanation = classify_signer_file(path)

            absolute_position = brace_start + signer_match.start()
            line_number = text[:absolute_position].count("\n") + 1

            signer_inventory.append(
                {
                    "file": str(relative),
                    "line": str(line_number),
                    "instruction_accounts": struct_name,
                    "account": account_name,
                    "classification": classification,
                    "explanation": explanation,
                }
            )

        for account_match in re.finditer(
            r"pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*"
            r"(?:AccountLoader|Account)\s*<'info,\s*"
            r"([A-Za-z_][A-Za-z0-9_:]*)>",
            struct_body,
        ):
            account_name = account_match.group(1)
            account_type = account_match.group(2)

            prefix = struct_body[
                max(0, account_match.start() - 300):account_match.start()
            ]

            if "mut" in prefix:
                absolute_position = brace_start + account_match.start()
                line_number = text[:absolute_position].count("\n") + 1

                mutable_accounts.append(
                    {
                        "file": str(relative),
                        "line": str(line_number),
                        "instruction_accounts": struct_name,
                        "account": account_name,
                        "type": account_type,
                    }
                )

    for finding_name, pattern in danger_patterns.items():
        for match in pattern.finditer(text):
            line_index = text[:match.start()].count("\n")

            findings.append(
                {
                    "type": finding_name,
                    "file": str(relative),
                    "line": line_index + 1,
                    "context": find_context(lines, line_index),
                }
            )

# Check public instruction entrypoints in lib.rs.
lib_path = SRC / "lib.rs"
entrypoints: list[str] = []

if lib_path.exists():
    lib_text = lib_path.read_text()

    entrypoints = re.findall(
        r"pub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
        lib_text,
    )

# Check release files explicitly.
release_verification: list[tuple[str, bool, bool, bool]] = []

for filename in sorted(RELEASE_FILES):
    path = SRC / "instructions" / filename

    if not path.exists():
        release_verification.append((filename, False, False, False))
        continue

    text = path.read_text()

    has_signer = bool(re.search(r"\bSigner\s*<'info>", text))
    has_authority_access = "ctx.accounts.authority" in text
    has_autonomous_guard = "authorize_autonomous_release" in text

    release_verification.append(
        (
            filename,
            has_signer,
            has_authority_access,
            has_autonomous_guard,
        )
    )

# Query program upgrade authority.
solana_available = shutil.which("solana") is not None
upgrade_output = ""

if solana_available:
    _, config_output = run_command(["solana", "config", "get"])
    _, program_output = run_command(
        ["solana", "program", "show", PROGRAM_ID]
    )

    upgrade_output = (
        "### Solana configuration\n\n"
        "```text\n"
        f"{config_output or 'No configuration output.'}\n"
        "```\n\n"
        "### Program information\n\n"
        "```text\n"
        f"{program_output or 'Program not found on the configured cluster.'}\n"
        "```"
    )
else:
    upgrade_output = (
        "The Solana CLI was unavailable, so upgrade authority status "
        "could not be queried."
    )

timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")

critical_signers = [
    signer for signer in signer_inventory
    if signer["classification"] == "CRITICAL FAILURE"
]

review_signers = [
    signer for signer in signer_inventory
    if signer["classification"] == "Requires review"
]

report: list[str] = [
    "# RBVR Authority Lockdown Audit",
    "",
    f"Generated: **{timestamp}**",
    "",
    "## Executive summary",
    "",
    f"- Public Rust entrypoints found: **{len(entrypoints)}**",
    f"- Signer accounts found: **{len(signer_inventory)}**",
    f"- Mutable program accounts found: **{len(mutable_accounts)}**",
    f"- Signers found in release handlers: **{len(critical_signers)}**",
    f"- Unclassified signers requiring review: **{len(review_signers)}**",
    "",
    "This report distinguishes initialization authority, depositor authority, "
    "release authority, mutable program accounts, and program upgrade authority.",
    "",
    "## Release permissionlessness verification",
    "",
    "| Handler | Signer present | Authority access | Autonomous guard | Result |",
    "|---|---:|---:|---:|---|",
]

for filename, has_signer, has_authority_access, has_guard in release_verification:
    passed = (
        not has_signer
        and not has_authority_access
        and has_guard
    )

    report.append(
        f"| `{filename}` | "
        f"{'YES' if has_signer else 'No'} | "
        f"{'YES' if has_authority_access else 'No'} | "
        f"{'Yes' if has_guard else 'NO'} | "
        f"{'PASS' if passed else 'FAIL'} |"
    )

report.extend(
    [
        "",
        "## Remaining signer inventory",
        "",
        "| File | Line | Accounts struct | Signer | Classification | Reason |",
        "|---|---:|---|---|---|---|",
    ]
)

if signer_inventory:
    for signer in signer_inventory:
        report.append(
            f"| `{signer['file']}` | {signer['line']} | "
            f"`{signer['instruction_accounts']}` | "
            f"`{signer['account']}` | "
            f"**{signer['classification']}** | "
            f"{signer['explanation']} |"
        )
else:
    report.append("| — | — | — | — | No signers found | — |")

report.extend(
    [
        "",
        "## Mutable account inventory",
        "",
        "| File | Line | Accounts struct | Account | Type |",
        "|---|---:|---|---|---|",
    ]
)

if mutable_accounts:
    for account in mutable_accounts:
        report.append(
            f"| `{account['file']}` | {account['line']} | "
            f"`{account['instruction_accounts']}` | "
            f"`{account['account']}` | `{account['type']}` |"
        )
else:
    report.append("| — | — | — | — | No mutable accounts found |")

report.extend(
    [
        "",
        "## Public instruction entrypoints",
        "",
    ]
)

for entrypoint in entrypoints:
    report.append(f"- `{entrypoint}`")

report.extend(
    [
        "",
        "## Upgrade authority status",
        "",
        upgrade_output,
        "",
        "## Authority-related source findings",
        "",
    ]
)

if findings:
    for index, finding in enumerate(findings, start=1):
        report.extend(
            [
                f"### {index}. {finding['type']}",
                "",
                f"File: `{finding['file']}:{finding['line']}`",
                "",
                "```text",
                str(finding["context"]),
                "```",
                "",
            ]
        )
else:
    report.append("No authority-related patterns were found.")

report.extend(
    [
        "## Automated preliminary verdict",
        "",
    ]
)

release_failures = [
    item for item in release_verification
    if item[1] or item[2] or not item[3]
]

if release_failures:
    report.append(
        "❌ **FAIL:** At least one release instruction is not fully "
        "permissionless."
    )
elif critical_signers:
    report.append(
        "❌ **FAIL:** A signer remains in at least one release handler."
    )
elif review_signers:
    report.append(
        "⚠️ **REVIEW REQUIRED:** Release handlers are permissionless, "
        "but one or more signers exist outside recognized initialization "
        "or deposit paths."
    )
else:
    report.append(
        "✅ **PRELIMINARY PASS:** Release handlers are permissionless, "
        "and all remaining signers are currently classified as "
        "initialization-only or depositor-controlled."
    )

report.extend(
    [
        "",
        "A final launch verdict still requires manual review of:",
        "",
        "1. Whether initialization can occur more than once.",
        "2. Whether configuration accounts can ever be replaced or mutated.",
        "3. Whether the program remains upgradeable.",
        "4. Whether any pause mechanism creates permanent human discretion.",
        "5. Whether initialization authority is permanently irrelevant after setup.",
        "",
    ]
)

REPORT.write_text("\n".join(report))

print("============================================================")
print("RBVR AUTHORITY LOCKDOWN AUDIT GENERATED")
print("============================================================")
print()
print(f"Report: {REPORT}")
print()
print(f"Signer accounts found: {len(signer_inventory)}")
print(f"Mutable accounts found: {len(mutable_accounts)}")
print(f"Release signer failures: {len(critical_signers)}")
print(f"Unclassified signer paths: {len(review_signers)}")
print()

for signer in signer_inventory:
    print(
        f"{signer['classification']}: "
        f"{signer['file']}:{signer['line']} "
        f"({signer['account']})"
    )

print()
print("Release verification:")

for filename, has_signer, has_authority_access, has_guard in release_verification:
    passed = (
        not has_signer
        and not has_authority_access
        and has_guard
    )

    print(
        f"  {filename}: "
        f"{'PASS' if passed else 'FAIL'}"
    )

print()
print("Review the report with:")
print("cat RBVR-AUTHORITY-LOCKDOWN-AUDIT.md")
