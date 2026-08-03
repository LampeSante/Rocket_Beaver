from __future__ import annotations

from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from pathlib import Path
import csv
import json
import re
import subprocess
import sys

ROOT = Path.cwd()
SRC = ROOT / "programs/treasury-router/src"
TESTS = ROOT / "tests"

REPORT_DIR = ROOT / "red-team"
REPORT = REPORT_DIR / "RT-001-ATTACK-SURFACE-INVENTORY.md"
JSON_REPORT = REPORT_DIR / "RT-001-ATTACK-SURFACE-INVENTORY.json"
CSV_REPORT = REPORT_DIR / "RT-001-ATTACK-REGISTER.csv"

if not SRC.exists():
    raise RuntimeError(f"Missing program source directory: {SRC}")

REPORT_DIR.mkdir(parents=True, exist_ok=True)

PROGRAM_ID_PATTERN = re.compile(
    r'declare_id!\(\s*"([^"]+)"\s*\)'
)

PUBLIC_FUNCTION_PATTERN = re.compile(
    r"(?m)^[ \t]*pub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*"
    r"(?:<[^>{;]*>)?\s*\("
)

ACCOUNT_STRUCT_PATTERN = re.compile(
    r"#\[derive\(Accounts\)\]\s*"
    r"(?:#\[[^\]]+\]\s*)*"
    r"pub\s+struct\s+([A-Za-z_][A-Za-z0-9_]*)"
    r"\s*<'info>\s*\{",
    re.DOTALL,
)

ACCOUNT_FIELD_PATTERN = re.compile(
    r"(?P<attrs>(?:\s*#\[account\([\s\S]*?\)\]\s*)*)"
    r"(?:\s*///[^\n]*\n)*"
    r"\s*pub\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*:\s*"
    r"(?P<type>[^,\n]+)",
    re.MULTILINE,
)

SEED_PATTERN = re.compile(
    r"seeds\s*=\s*\[(.*?)\]",
    re.DOTALL,
)

CONSTRAINT_PATTERN = re.compile(
    r"\bconstraint\s*=\s*(.*?)(?=,\s*(?:constraint|seeds|bump|"
    r"has_one|payer|space|token|mint|associated_token|close|realloc)"
    r"\b|\Z)",
    re.DOTALL,
)

ERROR_PATTERN = re.compile(
    r"@[\s\n]*TreasuryRouterError::([A-Za-z_][A-Za-z0-9_]*)"
)

CPI_PATTERNS = {
    "invoke": re.compile(r"\binvoke\s*\("),
    "invoke_signed": re.compile(r"\binvoke_signed\s*\("),
    "CpiContext::new": re.compile(r"\bCpiContext::new\s*\("),
    "CpiContext::new_with_signer": re.compile(
        r"\bCpiContext::new_with_signer\s*\("
    ),
    "token::transfer": re.compile(r"\btoken::transfer\s*\("),
    "token::transfer_checked": re.compile(
        r"\btoken::transfer_checked\s*\("
    ),
    "token::mint_to": re.compile(r"\btoken::mint_to\s*\("),
    "token::burn": re.compile(r"\btoken::burn\s*\("),
    "token::close_account": re.compile(
        r"\btoken::close_account\s*\("
    ),
    "system_instruction": re.compile(r"\bsystem_instruction::"),
}

ARITHMETIC_PATTERNS = {
    "checked_add": re.compile(r"\.checked_add\s*\("),
    "checked_sub": re.compile(r"\.checked_sub\s*\("),
    "checked_mul": re.compile(r"\.checked_mul\s*\("),
    "checked_div": re.compile(r"\.checked_div\s*\("),
    "saturating_add": re.compile(r"\.saturating_add\s*\("),
    "saturating_sub": re.compile(r"\.saturating_sub\s*\("),
    "wrapping_add": re.compile(r"\.wrapping_add\s*\("),
    "wrapping_sub": re.compile(r"\.wrapping_sub\s*\("),
    "unchecked plus": re.compile(
        r"(?<![+])\b[A-Za-z_][A-Za-z0-9_.]*\s*\+\s*"
        r"[A-Za-z0-9_(]"
    ),
    "unchecked minus": re.compile(
        r"(?<![-])\b[A-Za-z_][A-Za-z0-9_.]*\s*-\s*"
        r"[A-Za-z0-9_(]"
    ),
    "unchecked multiply": re.compile(
        r"\b[A-Za-z_][A-Za-z0-9_.]*\s*\*\s*[A-Za-z0-9_(]"
    ),
    "unchecked divide": re.compile(
        r"\b[A-Za-z_][A-Za-z0-9_.]*\s*/\s*[A-Za-z0-9_(]"
    ),
}

SECURITY_PATTERNS = {
    "remaining_accounts": re.compile(r"\bremaining_accounts\b"),
    "AccountInfo": re.compile(r"\bAccountInfo\s*<'info>"),
    "UncheckedAccount": re.compile(r"\bUncheckedAccount\s*<'info>"),
    "Signer": re.compile(r"\bSigner\s*<'info>"),
    "ProgramData": re.compile(r"\bProgramData\b"),
    "upgrade_authority": re.compile(
        r"\bupgrade_authority\b",
        re.IGNORECASE,
    ),
    "close constraint": re.compile(r"\bclose\s*="),
    "realloc constraint": re.compile(r"\brealloc\s*="),
    "init_if_needed": re.compile(r"\binit_if_needed\b"),
    "init": re.compile(r"\binit\b"),
    "has_one": re.compile(r"\bhas_one\s*="),
    "owner constraint": re.compile(
        r"\bowner\s*==|\bowner\s*="
    ),
    "mint constraint": re.compile(
        r"\bmint\s*==|\bmint\s*="
    ),
    "key comparison": re.compile(r"\.key\(\)\s*(?:==|!=)"),
    "paused mutation": re.compile(
        r"\b(?:paused|buybacks_paused)\s*="
    ),
    "authority mutation": re.compile(
        r"\bauthority\s*="
    ),
}

REQUIRE_PATTERNS = {
    "require!": re.compile(r"\brequire!\s*\("),
    "require_eq!": re.compile(r"\brequire_eq!\s*\("),
    "require_neq!": re.compile(r"\brequire_neq!\s*\("),
    "require_keys_eq!": re.compile(r"\brequire_keys_eq!\s*\("),
    "require_keys_neq!": re.compile(r"\brequire_keys_neq!\s*\("),
}

LIKELY_CRITICAL_INSTRUCTIONS = {
    "initialize",
    "initialize_protocol_config",
    "initialize_treasury",
    "initialize_founder",
    "initialize_company",
    "initialize_execution_config",
    "deposit_settlement",
    "process_fees",
    "authorize_reserve_execution",
    "authorize_buyback_execution",
    "authorize_liquidity_execution",
    "authorize_founder_execution",
    "authorize_company_execution",
}

@dataclass
class Finding:
    finding_id: str
    attack_class: str
    severity: str
    status: str
    file: str
    line: int
    symbol: str
    description: str
    evidence: str
    next_attack: str

def line_number(text: str, position: int) -> int:
    return text[:position].count("\n") + 1

def line_text(text: str, line: int) -> str:
    lines = text.splitlines()
    if 1 <= line <= len(lines):
        return lines[line - 1].strip()
    return ""

def balanced_block(
    text: str,
    start: int,
    opening: str = "{",
    closing: str = "}",
) -> tuple[int, str] | None:
    brace_start = text.find(opening, start)

    if brace_start == -1:
        return None

    depth = 0
    in_string = False
    escaped = False

    for index in range(brace_start, len(text)):
        char = text[index]

        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue

        if char == '"':
            in_string = True
            continue

        if char == opening:
            depth += 1
        elif char == closing:
            depth -= 1

            if depth == 0:
                return index + 1, text[start:index + 1]

    return None

def run(command: list[str], timeout: int = 60) -> tuple[int, str]:
    try:
        result = subprocess.run(
            command,
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            timeout=timeout,
            check=False,
        )
        return result.returncode, result.stdout.strip()
    except Exception as error:
        return 127, str(error)

rust_files = sorted(SRC.rglob("*.rs"))
test_files = sorted(TESTS.rglob("*.ts")) if TESTS.exists() else []

program_id = "NOT FOUND"
lib_path = SRC / "lib.rs"

if lib_path.exists():
    lib_text = lib_path.read_text()
    match = PROGRAM_ID_PATTERN.search(lib_text)

    if match:
        program_id = match.group(1)

entrypoints: list[dict[str, object]] = []

if lib_path.exists():
    lib_text = lib_path.read_text()

    for match in PUBLIC_FUNCTION_PATTERN.finditer(lib_text):
        name = match.group(1)
        entrypoints.append(
            {
                "name": name,
                "line": line_number(lib_text, match.start()),
                "critical": name in LIKELY_CRITICAL_INSTRUCTIONS,
            }
        )

accounts: list[dict[str, object]] = []
pdas: list[dict[str, object]] = []
cpis: list[dict[str, object]] = []
arithmetic: list[dict[str, object]] = []
security_references: list[dict[str, object]] = []
requires: list[dict[str, object]] = []
errors_used: list[dict[str, object]] = []
findings: list[Finding] = []

finding_counter = 1

def add_finding(
    attack_class: str,
    severity: str,
    file: str,
    line: int,
    symbol: str,
    description: str,
    evidence: str,
    next_attack: str,
    status: str = "OPEN — TEST REQUIRED",
) -> None:
    global finding_counter

    findings.append(
        Finding(
            finding_id=f"RT-{finding_counter:03d}",
            attack_class=attack_class,
            severity=severity,
            status=status,
            file=file,
            line=line,
            symbol=symbol,
            description=description,
            evidence=evidence,
            next_attack=next_attack,
        )
    )

    finding_counter += 1

for path in rust_files:
    relative = str(path.relative_to(ROOT))
    text = path.read_text()

    # Account structures
    for struct_match in ACCOUNT_STRUCT_PATTERN.finditer(text):
        struct_name = struct_match.group(1)
        block = balanced_block(text, struct_match.start())

        if block is None:
            continue

        _, struct_text = block
        struct_offset = struct_match.start()

        for field_match in ACCOUNT_FIELD_PATTERN.finditer(struct_text):
            attrs = field_match.group("attrs")
            account_name = field_match.group("name")
            account_type = field_match.group("type").strip()

            absolute_position = struct_offset + field_match.start()
            line = line_number(text, absolute_position)

            signer = bool(re.search(r"\bSigner\s*<'info>", account_type))
            mutable = bool(re.search(r"\bmut\b", attrs))
            init = bool(re.search(r"\binit\b", attrs))
            init_if_needed = bool(
                re.search(r"\binit_if_needed\b", attrs)
            )
            unchecked = (
                "UncheckedAccount" in account_type
                or "AccountInfo" in account_type
            )
            executable = bool(re.search(r"\bexecutable\b", attrs))
            has_one = re.findall(
                r"\bhas_one\s*=\s*([A-Za-z_][A-Za-z0-9_]*)",
                attrs,
            )
            seeds_matches = SEED_PATTERN.findall(attrs)
            seeds = [
                " ".join(seed.split())
                for seed in seeds_matches
            ]
            constraints = [
                " ".join(value.split())
                for value in CONSTRAINT_PATTERN.findall(attrs)
            ]

            record = {
                "file": relative,
                "line": line,
                "struct": struct_name,
                "account": account_name,
                "type": account_type,
                "signer": signer,
                "mutable": mutable,
                "init": init,
                "init_if_needed": init_if_needed,
                "unchecked": unchecked,
                "executable": executable,
                "has_one": has_one,
                "seeds": seeds,
                "constraints": constraints,
                "raw_attributes": " ".join(attrs.split()),
            }

            accounts.append(record)

            if seeds:
                pdas.append(record)

            if init_if_needed:
                add_finding(
                    "Reinitialization",
                    "CRITICAL",
                    relative,
                    line,
                    f"{struct_name}.{account_name}",
                    "`init_if_needed` introduces a repeated initialization surface.",
                    record["raw_attributes"],
                    "Attempt repeated initialization, account closure, and recreation.",
                )

            if unchecked:
                add_finding(
                    "Unchecked account substitution",
                    "HIGH",
                    relative,
                    line,
                    f"{struct_name}.{account_name}",
                    "Unchecked account type requires manual validation review.",
                    account_type,
                    "Supply attacker-owned, fake-mint, fake-vault, and wrong-program accounts.",
                )

            if mutable and unchecked:
                add_finding(
                    "Writable unchecked account",
                    "CRITICAL",
                    relative,
                    line,
                    f"{struct_name}.{account_name}",
                    "Account is both writable and unchecked.",
                    record["raw_attributes"],
                    "Attempt arbitrary writable-account substitution and state corruption.",
                )

            if signer and account_name.lower() not in {
                "authority",
                "payer",
                "user",
                "depositor",
            }:
                add_finding(
                    "Unexpected signer",
                    "HIGH",
                    relative,
                    line,
                    f"{struct_name}.{account_name}",
                    "Signer exists under an unexpected account name.",
                    account_type,
                    "Determine whether this signer enables hidden privilege.",
                )

            if init and not seeds:
                add_finding(
                    "Parallel state creation",
                    "HIGH",
                    relative,
                    line,
                    f"{struct_name}.{account_name}",
                    "Initialized account has no detected PDA seeds.",
                    record["raw_attributes"],
                    "Attempt creation of multiple parallel protocol accounts.",
                )

    # CPI inventory
    for cpi_name, pattern in CPI_PATTERNS.items():
        for match in pattern.finditer(text):
            line = line_number(text, match.start())

            cpis.append(
                {
                    "file": relative,
                    "line": line,
                    "type": cpi_name,
                    "evidence": line_text(text, line),
                }
            )

    # Arithmetic inventory
    for operation, pattern in ARITHMETIC_PATTERNS.items():
        for match in pattern.finditer(text):
            line = line_number(text, match.start())

            arithmetic.append(
                {
                    "file": relative,
                    "line": line,
                    "operation": operation,
                    "evidence": line_text(text, line),
                }
            )

            if operation.startswith("unchecked"):
                # Test code gets lower severity because fixture arithmetic
                # is not a production exploit path.
                is_test = (
                    "#[cfg(test)]" in text[:match.start()]
                    or "/tests/" in relative
                )

                add_finding(
                    "Arithmetic review",
                    "LOW" if is_test else "MEDIUM",
                    relative,
                    line,
                    operation,
                    "Potential unchecked arithmetic expression detected.",
                    line_text(text, line),
                    "Test maximum values, zero values, rounding, and overflow behavior.",
                    status=(
                        "REVIEW — LIKELY TEST CODE"
                        if is_test
                        else "OPEN — MANUAL REVIEW"
                    ),
                )

    # Other security references
    for name, pattern in SECURITY_PATTERNS.items():
        for match in pattern.finditer(text):
            line = line_number(text, match.start())

            security_references.append(
                {
                    "file": relative,
                    "line": line,
                    "type": name,
                    "evidence": line_text(text, line),
                }
            )

    # require macros
    for name, pattern in REQUIRE_PATTERNS.items():
        for match in pattern.finditer(text):
            line = line_number(text, match.start())

            requires.append(
                {
                    "file": relative,
                    "line": line,
                    "type": name,
                    "evidence": line_text(text, line),
                }
            )

    # Errors referenced
    for match in ERROR_PATTERN.finditer(text):
        errors_used.append(
            {
                "file": relative,
                "line": line_number(text, match.start()),
                "error": match.group(1),
            }
        )

# Entrypoint attack hypotheses
for entrypoint in entrypoints:
    name = str(entrypoint["name"])
    line = int(entrypoint["line"])

    if name.startswith("initialize"):
        attack_class = "Initialization replay"
        severity = "CRITICAL"
        next_attack = (
            "Call twice; call out of order; race two transactions; "
            "supply alternate PDA, wrong authority, wrong linked accounts."
        )
    elif name == "deposit_settlement":
        attack_class = "Deposit substitution"
        severity = "HIGH"
        next_attack = (
            "Supply wrong mint, wrong owner, wrong source authority, "
            "fake vault, zero amount, maximum amount."
        )
    elif name == "process_fees":
        attack_class = "Accounting corruption"
        severity = "CRITICAL"
        next_attack = (
            "Corrupt every bucket independently; overflow totals; "
            "repeat processing; manipulate timestamps and caps."
        )
    elif "authorize_" in name:
        attack_class = "Treasury release theft"
        severity = "CRITICAL"
        next_attack = (
            "Supply fake destinations, duplicate accounts, wrong mint, "
            "wrong vault, corrupt accounting, paused state, excessive release."
        )
    else:
        attack_class = "Public instruction"
        severity = "HIGH"
        next_attack = "Construct malicious account matrix and invalid state transitions."

    add_finding(
        attack_class,
        severity,
        str(lib_path.relative_to(ROOT)),
        line,
        name,
        f"Public instruction `{name}` is an externally reachable attack surface.",
        f"pub fn {name}(...)",
        next_attack,
        status="PLANNED — ADVERSARIAL TEST",
    )

# Consolidate known high-value protocol engines.
engine_files = sorted((SRC / "engines").glob("*.rs"))
engines = [
    {
        "name": path.stem,
        "file": str(path.relative_to(ROOT)),
        "lines": len(path.read_text().splitlines()),
    }
    for path in engine_files
]

# Test names
tests_inventory: list[dict[str, object]] = []

for path in rust_files:
    text = path.read_text()

    for match in re.finditer(
        r"#\[test\]\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)",
        text,
    ):
        tests_inventory.append(
            {
                "type": "rust",
                "file": str(path.relative_to(ROOT)),
                "line": line_number(text, match.start()),
                "name": match.group(1),
            }
        )

for path in test_files:
    text = path.read_text()

    for match in re.finditer(
        r'\bit\(\s*["\']([^"\']+)["\']',
        text,
    ):
        tests_inventory.append(
            {
                "type": "integration",
                "file": str(path.relative_to(ROOT)),
                "line": line_number(text, match.start()),
                "name": match.group(1),
            }
        )

# Fuzz targets
fuzz_targets_dir = ROOT / "fuzz/fuzz_targets"
fuzz_targets = []

if fuzz_targets_dir.exists():
    for path in sorted(fuzz_targets_dir.glob("*.rs")):
        fuzz_targets.append(
            {
                "name": path.stem,
                "file": str(path.relative_to(ROOT)),
                "lines": len(path.read_text().splitlines()),
            }
        )

# Git and tool metadata
_, git_head = run(["git", "rev-parse", "HEAD"])
_, git_status = run(["git", "status", "--short"])
_, git_log = run(["git", "log", "-5", "--oneline", "--decorate"])
_, cargo_test = run(["cargo", "test", "--workspace"], timeout=180)

severity_counts: dict[str, int] = {}

for finding in findings:
    severity_counts[finding.severity] = (
        severity_counts.get(finding.severity, 0) + 1
    )

report: list[str] = [
    "# RT-001 — RBVR Attack Surface Inventory",
    "",
    f"Generated: **{datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')}**",
    "",
    f"Program ID: `{program_id}`",
    "",
    f"Git commit: `{git_head or 'Unavailable'}`",
    "",
    "## Mission",
    "",
    "Map every externally reachable instruction, account, PDA, signer, CPI,",
    "token transfer, arithmetic operation, and state-mutation surface before",
    "attempting active exploitation.",
    "",
    "## Executive summary",
    "",
    f"- Public instruction entrypoints: **{len(entrypoints)}**",
    f"- Anchor account fields: **{len(accounts)}**",
    f"- PDA-constrained account fields: **{len(pdas)}**",
    f"- Signer fields: **{sum(1 for item in accounts if item['signer'])}**",
    f"- Writable account fields: **{sum(1 for item in accounts if item['mutable'])}**",
    f"- Unchecked account fields: **{sum(1 for item in accounts if item['unchecked'])}**",
    f"- CPI/token-operation references: **{len(cpis)}**",
    f"- Arithmetic-operation references: **{len(arithmetic)}**",
    f"- Require/constraint macro references: **{len(requires)}**",
    f"- Rust and integration tests: **{len(tests_inventory)}**",
    f"- Fuzz targets: **{len(fuzz_targets)}**",
    f"- Attack hypotheses registered: **{len(findings)}**",
    "",
    "## Severity count",
    "",
    "| Severity | Count |",
    "|---|---:|",
]

for severity in ["CRITICAL", "HIGH", "MEDIUM", "LOW"]:
    report.append(
        f"| {severity} | {severity_counts.get(severity, 0)} |"
    )

report.extend(
    [
        "",
        "## Public instruction attack surface",
        "",
        "| Entrypoint | lib.rs line | Priority |",
        "|---|---:|---|",
    ]
)

for entrypoint in entrypoints:
    report.append(
        f"| `{entrypoint['name']}` | {entrypoint['line']} | "
        f"{'Critical path' if entrypoint['critical'] else 'Review'} |"
    )

report.extend(
    [
        "",
        "## Account and PDA inventory",
        "",
        "| File | Line | Accounts struct | Account | Type | Signer | Mut | Init | PDA seeds | Unchecked |",
        "|---|---:|---|---|---|---:|---:|---:|---|---:|",
    ]
)

for item in accounts:
    seed_text = "; ".join(item["seeds"]) if item["seeds"] else "—"

    report.append(
        f"| `{item['file']}` | {item['line']} | "
        f"`{item['struct']}` | `{item['account']}` | "
        f"`{item['type']}` | "
        f"{'Yes' if item['signer'] else 'No'} | "
        f"{'Yes' if item['mutable'] else 'No'} | "
        f"{'Yes' if item['init'] else 'No'} | "
        f"`{seed_text}` | "
        f"{'YES' if item['unchecked'] else 'No'} |"
    )

report.extend(
    [
        "",
        "## CPI and token movement inventory",
        "",
        "| File | Line | Operation | Evidence |",
        "|---|---:|---|---|",
    ]
)

for item in cpis:
    evidence = str(item["evidence"]).replace("|", "\\|")
    report.append(
        f"| `{item['file']}` | {item['line']} | "
        f"`{item['type']}` | `{evidence}` |"
    )

report.extend(
    [
        "",
        "## Protocol engines",
        "",
        "| Engine | File | Lines |",
        "|---|---|---:|",
    ]
)

for item in engines:
    report.append(
        f"| `{item['name']}` | `{item['file']}` | {item['lines']} |"
    )

report.extend(
    [
        "",
        "## Fuzz targets",
        "",
        "| Target | File | Lines |",
        "|---|---|---:|",
    ]
)

for item in fuzz_targets:
    report.append(
        f"| `{item['name']}` | `{item['file']}` | {item['lines']} |"
    )

report.extend(
    [
        "",
        "## Master attack register",
        "",
        "| ID | Attack class | Severity | Status | Target | Location | Required next attack |",
        "|---|---|---|---|---|---|---|",
    ]
)

severity_order = {
    "CRITICAL": 0,
    "HIGH": 1,
    "MEDIUM": 2,
    "LOW": 3,
}

ordered_findings = sorted(
    findings,
    key=lambda item: (
        severity_order.get(item.severity, 99),
        item.file,
        item.line,
    ),
)

for finding in ordered_findings:
    next_attack = finding.next_attack.replace("|", "\\|")
    report.append(
        f"| {finding.finding_id} | {finding.attack_class} | "
        f"**{finding.severity}** | {finding.status} | "
        f"`{finding.symbol}` | "
        f"`{finding.file}:{finding.line}` | "
        f"{next_attack} |"
    )

report.extend(
    [
        "",
        "## Detailed attack hypotheses",
        "",
    ]
)

for finding in ordered_findings:
    report.extend(
        [
            f"### {finding.finding_id} — {finding.attack_class}",
            "",
            f"- **Severity:** {finding.severity}",
            f"- **Status:** {finding.status}",
            f"- **Target:** `{finding.symbol}`",
            f"- **Location:** `{finding.file}:{finding.line}`",
            f"- **Description:** {finding.description}",
            f"- **Evidence:** `{finding.evidence}`",
            f"- **Required attack:** {finding.next_attack}",
            "",
        ]
    )

report.extend(
    [
        "## Current test baseline",
        "",
        "```text",
        cargo_test or "No cargo test output.",
        "```",
        "",
        "## Recent Git history",
        "",
        "```text",
        git_log or "Unavailable.",
        "```",
        "",
        "## Working tree",
        "",
        "```text",
        git_status or "Clean working tree.",
        "```",
        "",
        "## RT-001 exit assessment",
        "",
        "RT-001 is an inventory phase, not a security pass.",
        "",
        "The inventory is complete when every public instruction and high-value",
        "account path has an assigned adversarial test. Findings marked OPEN or",
        "PLANNED are attack hypotheses, not confirmed vulnerabilities.",
        "",
        "## Recommended RT-002 target",
        "",
        "**Initialization and account-substitution assault.**",
        "",
        "Attempt repeated initialization, alternate PDAs, wrong bumps, wrong",
        "owners, wrong mints, wrong vaults, duplicate accounts, and out-of-order",
        "initialization across all canonical protocol accounts.",
        "",
    ]
)

REPORT.write_text("\n".join(report))

json_payload = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "program_id": program_id,
    "git_commit": git_head,
    "entrypoints": entrypoints,
    "accounts": accounts,
    "pdas": pdas,
    "cpis": cpis,
    "arithmetic": arithmetic,
    "security_references": security_references,
    "requires": requires,
    "errors_used": errors_used,
    "engines": engines,
    "fuzz_targets": fuzz_targets,
    "tests": tests_inventory,
    "findings": [asdict(item) for item in ordered_findings],
}

JSON_REPORT.write_text(
    json.dumps(json_payload, indent=2, sort_keys=True)
)

with CSV_REPORT.open("w", newline="") as csv_file:
    writer = csv.DictWriter(
        csv_file,
        fieldnames=[
            "finding_id",
            "attack_class",
            "severity",
            "status",
            "file",
            "line",
            "symbol",
            "description",
            "evidence",
            "next_attack",
        ],
    )

    writer.writeheader()

    for finding in ordered_findings:
        writer.writerow(asdict(finding))

print("============================================================")
print("RT-001 ATTACK SURFACE INVENTORY COMPLETE")
print("============================================================")
print()
print(f"Program ID:          {program_id}")
print(f"Public entrypoints:  {len(entrypoints)}")
print(f"Account fields:      {len(accounts)}")
print(f"PDA fields:          {len(pdas)}")
print(f"Signer fields:       {sum(1 for item in accounts if item['signer'])}")
print(f"Unchecked fields:    {sum(1 for item in accounts if item['unchecked'])}")
print(f"CPI references:      {len(cpis)}")
print(f"Fuzz targets:        {len(fuzz_targets)}")
print(f"Attack hypotheses:   {len(findings)}")
print()
print("Severity counts:")

for severity in ["CRITICAL", "HIGH", "MEDIUM", "LOW"]:
    print(f"  {severity}: {severity_counts.get(severity, 0)}")

print()
print(f"Markdown report: {REPORT}")
print(f"JSON report:     {JSON_REPORT}")
print(f"CSV register:    {CSV_REPORT}")
print()
print("Next planned phase: RT-002 Initialization and Account Substitution")
