from __future__ import annotations

from pathlib import Path
from datetime import datetime, timezone
import re
import subprocess

ROOT = Path.cwd()
SRC = ROOT / "programs/treasury-router/src"
REPORT = ROOT / "RBVR-GLOBAL-INVARIANT-MAP.md"

TARGETS = [
    SRC / "engines/release.rs",
    SRC / "engines/execution_guard.rs",
    SRC / "instructions/process_fees.rs",
    SRC / "state/treasury.rs",
    SRC / "state/protocol.rs",
    SRC / "state/protocol_config.rs",
    SRC / "state/company.rs",
    SRC / "state/founder.rs",
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
            timeout=60,
        )
        return result.returncode, result.stdout.strip()
    except Exception as error:
        return 127, str(error)

def extract_balanced_block(
    text: str,
    start: int,
    opening: str = "{",
    closing: str = "}",
) -> tuple[int, str] | None:
    brace = text.find(opening, start)

    if brace == -1:
        return None

    depth = 0
    in_string = False
    escaped = False

    for index in range(brace, len(text)):
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

def line_number(text: str, position: int) -> int:
    return text[:position].count("\n") + 1

def collect_functions(text: str) -> list[tuple[str, int, str]]:
    results: list[tuple[str, int, str]] = []

    pattern = re.compile(
        r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?"
        r"fn[ \t]+([A-Za-z_][A-Za-z0-9_]*)"
        r"[ \t]*(?:<[^>{;]*>)?[ \t]*\("
    )

    for match in pattern.finditer(text):
        block = extract_balanced_block(text, match.start())

        if block is None:
            continue

        _, body = block
        results.append(
            (
                match.group(1),
                line_number(text, match.start()),
                body,
            )
        )

    return results

def collect_structs(text: str) -> list[tuple[str, int, str]]:
    results: list[tuple[str, int, str]] = []

    pattern = re.compile(
        r"(?m)^[ \t]*(?:pub[ \t]+)?struct[ \t]+"
        r"([A-Za-z_][A-Za-z0-9_]*)"
    )

    for match in pattern.finditer(text):
        block = extract_balanced_block(text, match.start())

        if block is None:
            continue

        _, body = block
        results.append(
            (
                match.group(1),
                line_number(text, match.start()),
                body,
            )
        )

    return results

def collect_enums(text: str) -> list[tuple[str, int, str]]:
    results: list[tuple[str, int, str]] = []

    pattern = re.compile(
        r"(?m)^[ \t]*(?:pub[ \t]+)?enum[ \t]+"
        r"([A-Za-z_][A-Za-z0-9_]*)"
    )

    for match in pattern.finditer(text):
        block = extract_balanced_block(text, match.start())

        if block is None:
            continue

        _, body = block
        results.append(
            (
                match.group(1),
                line_number(text, match.start()),
                body,
            )
        )

    return results

def relevant_function(name: str, body: str) -> bool:
    key = name.lower()

    wanted_names = {
        "process_release",
        "authorize_release",
        "pending_balance",
        "valid_treasury",
        "valid_protocol",
        "calculate_share",
        "allocation_total",
    }

    wanted_words = (
        "accounting",
        "invariant",
        "release",
        "pending",
        "allocated",
        "received",
        "lifetime",
        "company",
        "founder",
        "overflow",
        "treasury",
    )

    return (
        name in wanted_names
        or key.startswith("process")
        or key.startswith("valid_")
        or key.startswith("assert_")
        or any(word in key for word in wanted_words)
        or any(
            token in body
            for token in (
                "pending_reserve",
                "released_reserve",
                "lifetime_reserve",
                "total_allocated",
                "total_received",
                "ReleaseBucket",
            )
        )
    )

def fenced(text: str) -> list[str]:
    return ["```rust", text.rstrip(), "```"]

report: list[str] = [
    "# RBVR Global Invariant Engineering Map",
    "",
    f"Generated: **{datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')}**",
    "",
    "This report captures the exact APIs and test helpers required to build",
    "the deterministic global state-machine invariant suite.",
    "",
]

missing: list[str] = []

for path in TARGETS:
    relative = path.relative_to(ROOT)

    report.extend(
        [
            f"## `{relative}`",
            "",
        ]
    )

    if not path.exists():
        report.append("**File not present.**")
        report.append("")
        missing.append(str(relative))
        continue

    text = path.read_text()

    structs = collect_structs(text)
    enums = collect_enums(text)
    functions = collect_functions(text)

    relevant_structs = [
        item
        for item in structs
        if any(
            word in item[0].lower()
            for word in (
                "treasury",
                "protocol",
                "company",
                "founder",
                "release",
                "allocation",
                "authorization",
            )
        )
    ]

    relevant_enums = [
        item
        for item in enums
        if any(
            word in item[0].lower()
            for word in (
                "release",
                "bucket",
                "waterfall",
                "dam",
            )
        )
    ]

    relevant_functions = [
        item
        for item in functions
        if relevant_function(item[0], item[2])
    ]

    report.extend(
        [
            f"- Structs found: **{len(structs)}**",
            f"- Enums found: **{len(enums)}**",
            f"- Functions found: **{len(functions)}**",
            f"- Relevant functions selected: **{len(relevant_functions)}**",
            "",
        ]
    )

    if relevant_structs:
        report.append("### Relevant structs")
        report.append("")

        for name, line, body in relevant_structs:
            report.append(f"#### `{name}` — line {line}")
            report.append("")
            report.extend(fenced(body))
            report.append("")

    if relevant_enums:
        report.append("### Relevant enums")
        report.append("")

        for name, line, body in relevant_enums:
            report.append(f"#### `{name}` — line {line}")
            report.append("")
            report.extend(fenced(body))
            report.append("")

    if relevant_functions:
        report.append("### Relevant functions and tests")
        report.append("")

        for name, line, body in relevant_functions:
            report.append(f"#### `{name}` — line {line}")
            report.append("")
            report.extend(fenced(body))
            report.append("")

# Search for exact treasury field names across the source tree.
field_patterns = [
    "total_received",
    "total_allocated",
    "lifetime_reserve",
    "pending_reserve",
    "released_reserve",
    "lifetime_buyback",
    "pending_buyback",
    "released_buyback",
    "lifetime_liquidity",
    "pending_liquidity",
    "released_liquidity",
    "lifetime_company",
    "pending_company",
    "released_company",
    "lifetime_founder",
    "pending_founder",
    "released_founder",
]

report.extend(
    [
        "## Treasury field-location inventory",
        "",
        "| Field | Source locations |",
        "|---|---|",
    ]
)

for field in field_patterns:
    locations: list[str] = []

    for path in sorted(SRC.rglob("*.rs")):
        text = path.read_text()

        for match in re.finditer(rf"\b{re.escape(field)}\b", text):
            locations.append(
                f"`{path.relative_to(ROOT)}:{line_number(text, match.start())}`"
            )

    report.append(
        f"| `{field}` | "
        + (", ".join(locations[:15]) if locations else "**Not found**")
        + " |"
    )

# Current verification baseline.
_, cargo_test = run(["cargo", "test", "--workspace"])
_, clippy = run([
    "cargo",
    "clippy",
    "--workspace",
    "--all-targets",
    "--all-features",
    "--",
    "-D",
    "warnings",
])
_, git_status = run(["git", "status", "--short"])
_, git_head = run(["git", "rev-parse", "HEAD"])

report.extend(
    [
        "",
        "## Current baseline",
        "",
        f"Git commit: `{git_head or 'Unavailable'}`",
        "",
        "### `cargo test --workspace`",
        "",
        "```text",
        cargo_test or "No output.",
        "```",
        "",
        "### Strict Clippy",
        "",
        "```text",
        clippy or "No output.",
        "```",
        "",
        "### Working tree",
        "",
        "```text",
        git_status or "Clean working tree.",
        "```",
        "",
        "## Planned global invariant suite",
        "",
        "The next patch will use the exact APIs above to exercise deterministic",
        "multi-step state transitions and check after every successful release:",
        "",
        "1. `lifetime == pending + released` for every bucket.",
        "2. Sum of bucket lifetime totals equals `total_allocated`.",
        "3. `total_allocated <= total_received`.",
        "4. Releasing one bucket changes no other bucket.",
        "5. Failed releases leave the complete treasury state unchanged.",
        "6. Sequential partial releases conserve all accounting.",
        "7. Full releases leave pending at zero without changing lifetime totals.",
        "8. Every ordering of the five release buckets reaches the same final state.",
        "",
    ]
)

if missing:
    report.extend(
        [
            "## Missing expected files",
            "",
        ]
    )

    for item in missing:
        report.append(f"- `{item}`")

REPORT.write_text("\n".join(report))

print("============================================================")
print("RBVR GLOBAL INVARIANT MAP COMPLETE")
print("============================================================")
print()
print(f"Report: {REPORT}")
print(f"Missing expected files: {len(missing)}")
print()
print("Review the compact extraction with:")
print(
    "grep -nE '^## |^### |^#### |pub fn |fn process_release|"
    "fn authorize_release|struct Treasury|enum ReleaseBucket' "
    "RBVR-GLOBAL-INVARIANT-MAP.md"
)
