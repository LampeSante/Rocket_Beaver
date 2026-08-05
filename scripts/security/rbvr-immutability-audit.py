from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
import re
import subprocess

ROOT = Path.cwd()
SRC = ROOT / "programs/treasury-router/src"
TESTS = ROOT / "tests"
REPORT = ROOT / "RBVR-IMMUTABILITY-AUDIT.md"

if not SRC.exists():
    raise RuntimeError(f"Missing source directory: {SRC}")

rust_files = sorted(SRC.rglob("*.rs"))
test_files = sorted(TESTS.rglob("*.ts")) if TESTS.exists() else []

INITIALIZER_NAMES = {
    "initialize",
    "initialize_protocol_config",
    "initialize_founder",
    "initialize_company",
    "initialize_treasury",
    "initialize_execution_config",
}

LOCKED_ECONOMIC_FIELDS = {
    "reserve_bps",
    "buyback_bps",
    "liquidity_bps",
    "company_bps",
    "founder_bps",
    "company_cap",
    "founder_cap",
    "period_duration",
}

LOCKED_DESTINATION_FIELDS = {
    "reserve_destination",
    "buyback_destination",
    "liquidity_destination",
    "company_destination",
    "founder_destination",
    "settlement_mint",
    "settlement_vault",
}

SECURITY_FIELDS = {
    "paused",
    "buybacks_paused",
    "configuration_updates_enabled",
    "authority",
    "version",
}

ALL_SENSITIVE_FIELDS = (
    LOCKED_ECONOMIC_FIELDS
    | LOCKED_DESTINATION_FIELDS
    | SECURITY_FIELDS
)

@dataclass
class Finding:
    severity: str
    category: str
    file: str
    line: int
    detail: str
    context: str

findings: list[Finding] = []

def line_number(text: str, position: int) -> int:
    return text[:position].count("\n") + 1

def context_for(text: str, line: int, radius: int = 4) -> str:
    lines = text.splitlines()
    index = max(0, line - 1)
    start = max(0, index - radius)
    end = min(len(lines), index + radius + 1)

    rendered = []

    for i in range(start, end):
        marker = ">" if i == index else " "
        rendered.append(f"{marker} {i + 1:4}: {lines[i]}")

    return "\n".join(rendered)

def add_finding(
    severity: str,
    category: str,
    path: Path,
    text: str,
    position: int,
    detail: str,
) -> None:
    line = line_number(text, position)

    findings.append(
        Finding(
            severity=severity,
            category=category,
            file=str(path.relative_to(ROOT)),
            line=line,
            detail=detail,
            context=context_for(text, line),
        )
    )

def extract_accounts_structs(text: str):
    pattern = re.compile(
        r"#\[derive\(Accounts\)\]\s*"
        r"(?:#\[[^\]]+\]\s*)*"
        r"pub\s+struct\s+([A-Za-z_][A-Za-z0-9_]*)"
        r"\s*<'info>\s*\{",
        re.DOTALL,
    )

    for match in pattern.finditer(text):
        brace_start = text.find("{", match.start())
        depth = 0
        end = None

        for position in range(brace_start, len(text)):
            char = text[position]

            if char == "{":
                depth += 1
            elif char == "}":
                depth -= 1

                if depth == 0:
                    end = position + 1
                    break

        if end is not None:
            yield match.group(1), match.start(), end, text[brace_start:end]

def extract_public_functions(text: str):
    pattern = re.compile(
        r"pub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)"
        r"\s*(?:<[^{>]*>)?\s*\(",
        re.MULTILINE,
    )

    return [
        (match.group(1), match.start())
        for match in pattern.finditer(text)
    ]

def run(command: list[str]) -> tuple[int, str]:
    try:
        result = subprocess.run(
            command,
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
            timeout=30,
        )
        return result.returncode, result.stdout.strip()
    except Exception as error:
        return 127, str(error)

# --------------------------------------------------------------------
# 1. Account initialization constraints
# --------------------------------------------------------------------

initializer_inventory: list[dict[str, str]] = []
init_if_needed_inventory: list[dict[str, str]] = []
mutable_inventory: list[dict[str, str]] = []

for path in rust_files:
    text = path.read_text()

    for struct_name, struct_start, _, body in extract_accounts_structs(text):
        body_offset = text.find("{", struct_start)

        account_pattern = re.compile(
            r"(?P<attrs>(?:\s*#\[account\([\s\S]*?\)\]\s*)*)"
            r"pub\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*:\s*"
            r"(?P<type>[^,\n]+)",
            re.MULTILINE,
        )

        for match in account_pattern.finditer(body):
            attrs = match.group("attrs")
            account_name = match.group("name")
            account_type = match.group("type").strip()
            absolute_position = body_offset + match.start()

            uses_init_if_needed = bool(
                re.search(r"\binit_if_needed\b", attrs)
            )
            uses_init = bool(
                re.search(r"\binit\b", attrs)
                and not uses_init_if_needed
            )
            is_mut = bool(re.search(r"\bmut\b", attrs))

            record = {
                "file": str(path.relative_to(ROOT)),
                "line": str(line_number(text, absolute_position)),
                "struct": struct_name,
                "account": account_name,
                "type": account_type,
            }

            if uses_init:
                initializer_inventory.append(record)

            if uses_init_if_needed:
                init_if_needed_inventory.append(record)
                add_finding(
                    "CRITICAL",
                    "Reinitialization surface",
                    path,
                    text,
                    absolute_position,
                    (
                        f"`{struct_name}.{account_name}` uses "
                        "`init_if_needed`, which may permit a repeated "
                        "initialization path."
                    ),
                )

            if is_mut:
                mutable_inventory.append(record)

# --------------------------------------------------------------------
# 2. Sensitive field assignments
# --------------------------------------------------------------------

assignment_inventory: list[dict[str, str]] = []

for path in rust_files:
    text = path.read_text()

    for field in sorted(ALL_SENSITIVE_FIELDS):
        pattern = re.compile(
            rf"""
            (?:
                \.\s*{re.escape(field)}
                |
                \b{re.escape(field)}
            )
            \s*=
            (?!=)
            """,
            re.VERBOSE,
        )

        for match in pattern.finditer(text):
            line = line_number(text, match.start())
            context = context_for(text, line)

            # Ignore obvious unit-test fixture mutation.
            in_test_area = (
                "/tests/" in str(path)
                or "#[cfg(test)]" in text[:match.start()][-1000:]
                or re.search(
                    r"\bfn\s+[A-Za-z0-9_]*test[A-Za-z0-9_]*\b",
                    text[max(0, match.start() - 1000):match.start()],
                )
            )

            assignment_inventory.append(
                {
                    "file": str(path.relative_to(ROOT)),
                    "line": str(line),
                    "field": field,
                    "test_only": "Yes" if in_test_area else "No",
                    "context": context,
                }
            )

            if not in_test_area:
                severity = "HIGH"

                if field in LOCKED_ECONOMIC_FIELDS:
                    category = "Economic mutation"
                elif field in LOCKED_DESTINATION_FIELDS:
                    category = "Destination mutation"
                else:
                    category = "Security-state mutation"

                add_finding(
                    severity,
                    category,
                    path,
                    text,
                    match.start(),
                    f"Production code assigns sensitive field `{field}`.",
                )

# --------------------------------------------------------------------
# 3. Public entrypoints and possible mutation routes
# --------------------------------------------------------------------

lib_path = SRC / "lib.rs"
entrypoints: list[dict[str, str]] = []

if lib_path.exists():
    lib_text = lib_path.read_text()

    for name, position in extract_public_functions(lib_text):
        entrypoints.append(
            {
                "name": name,
                "line": str(line_number(lib_text, position)),
                "initializer": "Yes" if name in INITIALIZER_NAMES else "No",
            }
        )

        suspicious_name = bool(
            re.search(
                r"(update|set|change|rotate|replace|pause|unpause|"
                r"configure|admin|authority|destination|cap|period)",
                name,
                re.IGNORECASE,
            )
        )

        if suspicious_name:
            add_finding(
                "CRITICAL",
                "Public mutation entrypoint",
                lib_path,
                lib_text,
                position,
                (
                    f"Public entrypoint `{name}` appears capable of changing "
                    "configuration, authority, or protocol state."
                ),
            )

# --------------------------------------------------------------------
# 4. Search instruction files for direct sensitive mutation functions
# --------------------------------------------------------------------

for path in rust_files:
    text = path.read_text()

    mutation_function_pattern = re.compile(
        r"pub\s+fn\s+"
        r"([A-Za-z_][A-Za-z0-9_]*(?:update|set|change|rotate|replace|"
        r"pause|unpause|configure|admin)[A-Za-z0-9_]*)"
        r"\s*\(",
        re.IGNORECASE,
    )

    for match in mutation_function_pattern.finditer(text):
        function_name = match.group(1)

        if function_name.startswith("test"):
            continue

        add_finding(
            "HIGH",
            "Mutation function",
            path,
            text,
            match.start(),
            f"Function `{function_name}` may expose a mutation path.",
        )

# --------------------------------------------------------------------
# 5. Check PDA initialization seeds and payer patterns
# --------------------------------------------------------------------

pda_seed_inventory: list[dict[str, str]] = []

for path in rust_files:
    text = path.read_text()

    for struct_name, struct_start, _, body in extract_accounts_structs(text):
        for account_match in re.finditer(
            r"#\[account\((?P<attrs>[\s\S]*?)\)\]\s*"
            r"pub\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*:",
            body,
        ):
            attrs = account_match.group("attrs")
            account_name = account_match.group("name")

            if not re.search(r"\binit(?:_if_needed)?\b", attrs):
                continue

            seeds = re.findall(
                r"seeds\s*=\s*\[(.*?)\]",
                attrs,
                re.DOTALL,
            )

            payer = re.search(
                r"payer\s*=\s*([A-Za-z_][A-Za-z0-9_]*)",
                attrs,
            )

            absolute = text.find("{", struct_start) + account_match.start()

            pda_seed_inventory.append(
                {
                    "file": str(path.relative_to(ROOT)),
                    "line": str(line_number(text, absolute)),
                    "struct": struct_name,
                    "account": account_name,
                    "seeds": (
                        " ".join(seeds[0].split())
                        if seeds
                        else "NO SEEDS FOUND"
                    ),
                    "payer": payer.group(1) if payer else "Not found",
                }
            )

            if not seeds:
                add_finding(
                    "HIGH",
                    "Initialization uniqueness",
                    path,
                    text,
                    absolute,
                    (
                        f"Initialized account `{account_name}` has no detected "
                        "PDA seeds. Verify repeated or alternate initialization "
                        "cannot create parallel protocol state."
                    ),
                )

# --------------------------------------------------------------------
# 6. Test coverage for repeated initialization and immutability
# --------------------------------------------------------------------

all_test_text = "\n".join(path.read_text() for path in test_files)

test_requirements = {
    "initialize twice": [
        r"initialize.*twice",
        r"rejects?.*second.*initial",
        r"already.*initialized",
    ],
    "protocol config reinitialization": [
        r"protocol config.*twice",
        r"reinitializ.*protocol config",
        r"initializeProtocolConfig.*reject",
    ],
    "treasury reinitialization": [
        r"treasury.*twice",
        r"reinitializ.*treasury",
        r"initializeTreasury.*reject",
    ],
    "execution config reinitialization": [
        r"execution config.*twice",
        r"reinitializ.*execution config",
        r"initializeExecutionConfig.*reject",
    ],
    "founder reinitialization": [
        r"founder.*twice",
        r"reinitializ.*founder",
        r"initializeFounder.*reject",
    ],
    "company reinitialization": [
        r"company.*twice",
        r"reinitializ.*company",
        r"initializeCompany.*reject",
    ],
    "destination immutability": [
        r"destination.*immutable",
        r"cannot.*change.*destination",
        r"rejects?.*destination.*change",
    ],
    "allocation immutability": [
        r"allocation.*immutable",
        r"cannot.*change.*allocation",
        r"locked.*allocation",
    ],
}

test_coverage: list[dict[str, str]] = []

for requirement, patterns in test_requirements.items():
    covered = any(
        re.search(pattern, all_test_text, re.IGNORECASE | re.DOTALL)
        for pattern in patterns
    )

    test_coverage.append(
        {
            "requirement": requirement,
            "covered": "Yes" if covered else "No",
        }
    )

# --------------------------------------------------------------------
# 7. Git and build metadata
# --------------------------------------------------------------------

_, git_head = run(["git", "rev-parse", "HEAD"])
_, git_status = run(["git", "status", "--short"])
_, cargo_check = run(["cargo", "check", "--workspace"])

# --------------------------------------------------------------------
# 8. Verdict
# --------------------------------------------------------------------

production_findings = [
    finding
    for finding in findings
    if finding.severity in {"CRITICAL", "HIGH"}
]

critical_findings = [
    finding
    for finding in findings
    if finding.severity == "CRITICAL"
]

missing_tests = [
    item["requirement"]
    for item in test_coverage
    if item["covered"] == "No"
]

if critical_findings:
    verdict = (
        "❌ **FAIL / MANUAL REVIEW REQUIRED:** One or more potentially "
        "critical reinitialization or public mutation paths were detected."
    )
elif production_findings:
    verdict = (
        "⚠️ **CONDITIONAL:** No obvious critical public mutation entrypoint "
        "was detected, but sensitive production assignments require "
        "line-by-line verification."
    )
else:
    verdict = (
        "✅ **PRELIMINARY PASS:** No obvious post-initialization mutation "
        "entrypoint or `init_if_needed` path was detected."
    )

# --------------------------------------------------------------------
# 9. Report
# --------------------------------------------------------------------

generated = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")

report: list[str] = [
    "# RBVR Immutable Protocol Audit",
    "",
    f"Generated: **{generated}**",
    "",
    f"Git commit: `{git_head or 'Unavailable'}`",
    "",
    "## Executive summary",
    "",
    f"- Public entrypoints: **{len(entrypoints)}**",
    f"- Initialized accounts detected: **{len(initializer_inventory)}**",
    f"- `init_if_needed` accounts detected: **{len(init_if_needed_inventory)}**",
    f"- Mutable accounts detected: **{len(mutable_inventory)}**",
    f"- Sensitive assignments detected: **{len(assignment_inventory)}**",
    f"- Critical findings: **{len(critical_findings)}**",
    f"- High-severity findings: **{len([f for f in findings if f.severity == 'HIGH'])}**",
    f"- Missing immutability test categories: **{len(missing_tests)}**",
    "",
    "## Preliminary verdict",
    "",
    verdict,
    "",
    "This is a static source audit. It identifies candidate mutation and "
    "reinitialization paths but does not replace transaction-level tests.",
    "",
    "## Public entrypoints",
    "",
    "| Entrypoint | Line | Initializer |",
    "|---|---:|---:|",
]

for entrypoint in entrypoints:
    report.append(
        f"| `{entrypoint['name']}` | {entrypoint['line']} | "
        f"{entrypoint['initializer']} |"
    )

report.extend(
    [
        "",
        "## Initialized account inventory",
        "",
        "| File | Line | Accounts struct | Account | Type |",
        "|---|---:|---|---|---|",
    ]
)

if initializer_inventory:
    for item in initializer_inventory:
        report.append(
            f"| `{item['file']}` | {item['line']} | "
            f"`{item['struct']}` | `{item['account']}` | "
            f"`{item['type']}` |"
        )
else:
    report.append("| — | — | — | — | No `init` accounts detected |")

report.extend(
    [
        "",
        "## `init_if_needed` inventory",
        "",
        "| File | Line | Accounts struct | Account | Type |",
        "|---|---:|---|---|---|",
    ]
)

if init_if_needed_inventory:
    for item in init_if_needed_inventory:
        report.append(
            f"| `{item['file']}` | {item['line']} | "
            f"`{item['struct']}` | `{item['account']}` | "
            f"`{item['type']}` |"
        )
else:
    report.append("| — | — | — | — | None detected |")

report.extend(
    [
        "",
        "## PDA uniqueness inventory",
        "",
        "| File | Line | Accounts struct | Account | Seeds | Payer |",
        "|---|---:|---|---|---|---|",
    ]
)

for item in pda_seed_inventory:
    report.append(
        f"| `{item['file']}` | {item['line']} | "
        f"`{item['struct']}` | `{item['account']}` | "
        f"`{item['seeds']}` | `{item['payer']}` |"
    )

report.extend(
    [
        "",
        "## Mutable account inventory",
        "",
        "| File | Line | Accounts struct | Account | Type |",
        "|---|---:|---|---|---|",
    ]
)

for item in mutable_inventory:
    report.append(
        f"| `{item['file']}` | {item['line']} | "
        f"`{item['struct']}` | `{item['account']}` | "
        f"`{item['type']}` |"
    )

report.extend(
    [
        "",
        "## Sensitive field assignments",
        "",
        "| File | Line | Field | Test-only heuristic |",
        "|---|---:|---|---:|",
    ]
)

for item in assignment_inventory:
    report.append(
        f"| `{item['file']}` | {item['line']} | "
        f"`{item['field']}` | {item['test_only']} |"
    )

report.extend(
    [
        "",
        "## Immutability test coverage",
        "",
        "| Required test category | Detected |",
        "|---|---:|",
    ]
)

for item in test_coverage:
    report.append(
        f"| {item['requirement']} | {item['covered']} |"
    )

report.extend(
    [
        "",
        "## Findings requiring review",
        "",
    ]
)

if findings:
    severity_order = {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 2, "LOW": 3}
    ordered = sorted(
        findings,
        key=lambda item: (
            severity_order.get(item.severity, 99),
            item.file,
            item.line,
        ),
    )

    for index, finding in enumerate(ordered, start=1):
        report.extend(
            [
                f"### {index}. {finding.severity}: {finding.category}",
                "",
                f"**Location:** `{finding.file}:{finding.line}`",
                "",
                finding.detail,
                "",
                "```text",
                finding.context,
                "```",
                "",
            ]
        )
else:
    report.append("No candidate mutation or reinitialization paths detected.")

report.extend(
    [
        "## Build verification",
        "",
        "```text",
        cargo_check or "No cargo output.",
        "```",
        "",
        "## Current working tree",
        "",
        "```text",
        git_status or "Clean working tree.",
        "```",
        "",
        "## Required next tests",
        "",
    ]
)

if missing_tests:
    for test_name in missing_tests:
        report.append(f"- Add explicit test: **{test_name}**")
else:
    report.append("- All searched immutability test categories were detected.")

report.extend(
    [
        "",
        "## Manual review questions",
        "",
        "1. Does every protocol account use one canonical PDA?",
        "2. Can any parallel protocol/config/treasury account be created?",
        "3. Can initialization instructions succeed after their canonical PDA exists?",
        "4. Are destination, cap, period, mint, vault, and allocation fields assigned only once?",
        "5. Is there any public update, pause, authority-rotation, or destination-change instruction?",
        "6. Does every mutable initialization account change only a one-time completion flag?",
        "7. Can any close-account path reopen an initialization surface?",
        "",
    ]
)

REPORT.write_text("\n".join(report))

print("============================================================")
print("RBVR IMMUTABILITY AUDIT COMPLETE")
print("============================================================")
print()
print(f"Report: {REPORT}")
print(f"Public entrypoints: {len(entrypoints)}")
print(f"Initialized accounts: {len(initializer_inventory)}")
print(f"init_if_needed accounts: {len(init_if_needed_inventory)}")
print(f"Mutable accounts: {len(mutable_inventory)}")
print(f"Sensitive assignments: {len(assignment_inventory)}")
print(f"Critical findings: {len(critical_findings)}")
print(f"Missing test categories: {len(missing_tests)}")
print()
print("Verdict:")
print(verdict)
print()
print("Missing tests:")

if missing_tests:
    for test_name in missing_tests:
        print(f"  - {test_name}")
else:
    print("  None detected by the static search.")

print()
print("Review with:")
print("sed -n '1,260p' RBVR-IMMUTABILITY-AUDIT.md")
