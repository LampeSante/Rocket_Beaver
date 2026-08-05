#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[2]

SOURCE_PATHS = [
    ROOT / "programs",
    ROOT / "tests",
    ROOT / "target" / "idl",
    ROOT / "docs" / "deployment",
    ROOT / "docs" / "security",
    ROOT / "docs" / "architecture",
]

EXCLUDED_MARKDOWN = {
    "CHANGELOG.md",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
}

RISK_TERMS = {
    "immutable": [
        "immutable",
        "immutability",
        "cannot be changed",
        "non-upgradeable",
    ],
    "founder_fixed_rate": [
        "fixed founder rate",
        "10% of the protocol",
        "0.1% of eligible transaction volume",
        "annual founder cap",
    ],
    "permissionless": [
        "permissionless",
        "any caller",
        "anyone can invoke",
    ],
    "overflow": [
        "overflow",
        "excess company",
        "excess founder",
        "redirected to liquidity",
    ],
    "atomicity": [
        "atomic",
        "atomically",
        "all-or-nothing",
    ],
    "replay": [
        "replay",
        "duplicate execution",
        "cannot be repeated",
    ],
    "reserve": [
        "reserve floor",
        "minimum reserve",
        "cooldown",
        "spillway",
        "surplus deployment",
    ],
    "allocation": [
        "30%",
        "20%",
        "10%",
        "3000",
        "2000",
        "1000",
    ],
}


@dataclass
class Evidence:
    path: Path
    line: int
    text: str


@dataclass
class Finding:
    document: Path
    line: int
    claim: str
    category: str
    status: str
    explanation: str
    evidence: list[Evidence]


def git(*args: str) -> str:
    return subprocess.check_output(
        ["git", *args],
        cwd=ROOT,
        text=True,
    ).strip()


def iter_files(paths: Iterable[Path]) -> Iterable[Path]:
    for base in paths:
        if not base.exists():
            continue

        if base.is_file():
            yield base
            continue

        for path in sorted(base.rglob("*")):
            if not path.is_file():
                continue

            if any(part in {".git", "node_modules", "target"} for part in path.parts):
                if "target" not in path.parts or "idl" not in path.parts:
                    continue

            if path.suffix.lower() in {
                ".rs",
                ".ts",
                ".js",
                ".mjs",
                ".json",
                ".md",
                ".txt",
                ".toml",
            }:
                yield path


def load_corpus() -> list[tuple[Path, list[str]]]:
    corpus: list[tuple[Path, list[str]]] = []

    for path in iter_files(SOURCE_PATHS):
        try:
            lines = path.read_text(
                encoding="utf-8",
                errors="replace",
            ).splitlines()
        except OSError:
            continue

        corpus.append((path, lines))

    return corpus


def search(
    corpus: list[tuple[Path, list[str]]],
    patterns: Iterable[str],
    limit: int = 12,
) -> list[Evidence]:
    compiled = [
        re.compile(pattern, re.IGNORECASE)
        for pattern in patterns
    ]

    evidence: list[Evidence] = []

    for path, lines in corpus:
        for number, line in enumerate(lines, start=1):
            if any(pattern.search(line) for pattern in compiled):
                evidence.append(
                    Evidence(
                        path=path,
                        line=number,
                        text=line.strip(),
                    )
                )

                if len(evidence) >= limit:
                    return evidence

    return evidence


def markdown_documents() -> list[Path]:
    documents = []

    for path in sorted(ROOT.glob("*.md")):
        if path.name in EXCLUDED_MARKDOWN:
            continue

        documents.append(path)

    return documents


def claim_lines(path: Path) -> Iterable[tuple[int, str]]:
    in_code = False

    for number, raw in enumerate(
        path.read_text(
            encoding="utf-8",
            errors="replace",
        ).splitlines(),
        start=1,
    ):
        stripped = raw.strip()

        if stripped.startswith("```"):
            in_code = not in_code
            continue

        if in_code:
            continue

        if not stripped:
            continue

        if stripped.startswith("#"):
            continue

        if stripped.startswith("|"):
            continue

        if stripped.startswith("- ["):
            continue

        yield number, stripped


def category_for(claim: str) -> str | None:
    lowered = claim.lower()

    for category, terms in RISK_TERMS.items():
        if any(term.lower() in lowered for term in terms):
            return category

    return None


def evaluate(
    document: Path,
    line: int,
    claim: str,
    category: str,
    corpus: list[tuple[Path, list[str]]],
) -> Finding:
    if category == "founder_fixed_rate":
        evidence = search(
            corpus,
            [
                r"INITIAL_FOUNDER_BPS",
                r"founder_bps",
                r"period_cap",
                r"earned_current_period",
                r"liquidity_overflow_amount",
            ],
        )

        if evidence:
            return Finding(
                document,
                line,
                claim,
                category,
                "VERIFIED",
                (
                    "Founder allocation uses the fixed configured Founder basis-point share, "
                    "is constrained by a period cap, and redirects capped excess to Liquidity. "
                    "Annual enforcement depends on initializing the period duration to one year."
                ),
                evidence,
            )

        return Finding(
            document,
            line,
            claim,
            category,
            "UNSUPPORTED",
            "The fixed Founder rate or annual-cap controls were not found.",
            [],
        )

    if category == "overflow":
        evidence = search(
            corpus,
            [
                r"all Company and Founder cap overflow",
                r"liquidity_overflow_amount",
                r"company_cap_overflow_redirected_to_liquidity",
                r"founder_cap_overflow_redirected_to_liquidity",
            ],
        )

        if evidence:
            return Finding(
                document,
                line,
                claim,
                category,
                "VERIFIED",
                "Company and Founder capped excess are implemented as Liquidity overflow.",
                evidence,
            )

    if category == "reserve":
        evidence = search(
            corpus,
            [
                r"minimum_reserve_floor",
                r"surplus_deployment_bps",
                r"cooldown_seconds",
                r"ReserveDeploymentCooldownActive",
            ],
        )

        if evidence:
            return Finding(
                document,
                line,
                claim,
                category,
                "VERIFIED",
                "Reserve floor, deployment ratio, and cooldown are present in source.",
                evidence,
            )

    if category == "allocation":
        evidence = search(
            corpus,
            [
                r"INITIAL_RESERVE_BPS",
                r"INITIAL_BUYBACK_BURN_BPS",
                r"INITIAL_LIQUIDITY_BPS",
                r"INITIAL_COMPANY_BPS",
                r"INITIAL_FOUNDER_BPS",
                r'"reserve_bps"\s*:\s*3000',
                r'"founder_bps"\s*:\s*1000',
            ],
        )

        if evidence:
            return Finding(
                document,
                line,
                claim,
                category,
                "VERIFIED",
                "The fixed basis-point configuration is present in constants, initialization, or verified state.",
                evidence,
            )

    if category == "replay":
        evidence = search(
            corpus,
            [
                r"rejects.*replay",
                r"replay.*without mutation",
                r"NoUnprocessedFees",
                r"ReserveDeploymentCooldownActive",
            ],
        )

        if evidence:
            return Finding(
                document,
                line,
                claim,
                category,
                "VERIFIED",
                "Replay rejection is covered by program guards and adversarial tests.",
                evidence,
            )

    if category == "atomicity":
        evidence = search(
            corpus,
            [
                r"occur atomically",
                r"transfer.*accounting",
                r"failed.*does not mutate",
                r"without mutation",
            ],
        )

        if evidence:
            return Finding(
                document,
                line,
                claim,
                category,
                "VERIFIED",
                "Atomic transfer/accounting expectations are supported by source comments and mutation tests.",
                evidence,
            )

    if category == "permissionless":
        evidence = search(
            corpus,
            [
                r"permissionless",
                r"cannot influence the source",
                r"cannot influence.*destination",
                r"Spillway",
            ],
        )

        signer_evidence = search(
            corpus,
            [
                r"struct Spillway",
                r"Signer<'info>",
            ],
            limit=50,
        )

        spillway_signer = any(
            "spillway_release.rs" in str(item.path)
            and "Signer<'info>" in item.text
            for item in signer_evidence
        )

        if evidence and not spillway_signer:
            return Finding(
                document,
                line,
                claim,
                category,
                "VERIFIED",
                "The Spillway source describes permissionless invocation and does not expose caller-selected policy or destinations.",
                evidence,
            )

        return Finding(
            document,
            line,
            claim,
            category,
            "INFERRED",
            "Permissionless behaviour needs manual confirmation from the complete account context.",
            evidence,
        )

    if category == "immutable":
        config_evidence = search(
            corpus,
            [
                r"updates_enabled\s*=\s*false",
                r"configuration_locked",
                r"updates_enabled",
                r"init,",
            ],
        )

        upgrade_evidence = search(
            corpus,
            [
                r"upgrade authority",
                r"UpgradeableLoader",
                r"BPFLoaderUpgradeab",
            ],
        )

        return Finding(
            document,
            line,
            claim,
            category,
            "QUALIFY",
            (
                "Configuration may be initialized once and locked with "
                "`updates_enabled = false`, but the deployed Solana program "
                "remains upgradeable until upgrade authority is revoked."
            ),
            config_evidence + upgrade_evidence,
        )

    return Finding(
        document,
        line,
        claim,
        category,
        "INFERRED",
        "No category-specific proof rule established.",
        [],
    )


def apply_safe_patches() -> list[Path]:
    changed: list[Path] = []

    replacements = [
        (
            re.compile(
                r"Founder compensation follows a progressive marginal "
                r"volume schedule in which the applicable rate declines "
                r"as eligible volume increases\.",
                re.IGNORECASE,
            ),
            (
                "Founder compensation is period-capped and includes a "
                "`current_tier` state field. The current source audit did "
                "not verify an implemented progressive marginal-rate "
                "calculation, so no declining-rate schedule is claimed here."
            ),
        ),
        (
            re.compile(
                r"Founder compensation is capped per period and follows "
                r"a progressive marginal volume schedule\.",
                re.IGNORECASE,
            ),
            (
                "Founder compensation is capped per period. The account "
                "includes a `current_tier` field, but the current source "
                "audit did not verify progressive marginal-rate mathematics."
            ),
        ),
        (
            re.compile(
                r"immutable protocol configuration",
                re.IGNORECASE,
            ),
            (
                "one-time protocol configuration with "
                "`updates_enabled = false` in the verified deployment"
            ),
        ),
        (
            re.compile(
                r"the protocol is immutable",
                re.IGNORECASE,
            ),
            (
                "the verified configuration is locked against ordinary "
                "updates; the program remains upgradeable while an upgrade "
                "authority exists"
            ),
        ),
    ]

    for path in markdown_documents():
        original = path.read_text(
            encoding="utf-8",
            errors="replace",
        )
        updated = original

        for pattern, replacement in replacements:
            updated = pattern.sub(replacement, updated)

        if updated != original:
            path.write_text(updated, encoding="utf-8")
            changed.append(path)

    return changed


def write_report(
    findings: list[Finding],
    changed: list[Path],
    report_path: Path,
) -> None:
    status_counts: dict[str, int] = {}

    for finding in findings:
        status_counts[finding.status] = (
            status_counts.get(finding.status, 0) + 1
        )

    with report_path.open("w", encoding="utf-8") as report:
        report.write("# RBVR Documentation Source Audit\n\n")
        report.write(
            f"Generated: "
            f"{datetime.now(timezone.utc).isoformat(timespec='seconds')}\n\n"
        )
        report.write(f"Commit before documentation patch: `{git('rev-parse', 'HEAD')}`\n\n")

        report.write("## Scope\n\n")

        for document in markdown_documents():
            report.write(f"- `{document.relative_to(ROOT)}`\n")

        report.write("\n## Classification\n\n")
        report.write("- **VERIFIED**: direct source, IDL, test, or deployment evidence found.\n")
        report.write("- **QUALIFY**: partly true but requires narrower wording.\n")
        report.write("- **INFERRED**: plausible but not directly established by the automated rules.\n")
        report.write("- **UNSUPPORTED**: the stated implementation claim was not found.\n\n")

        report.write("## Summary\n\n")

        for status in sorted(status_counts):
            report.write(f"- {status}: {status_counts[status]}\n")

        report.write("\n## Automatically changed files\n\n")

        if changed:
            for path in changed:
                report.write(f"- `{path.relative_to(ROOT)}`\n")
        else:
            report.write("- None\n")

        report.write("\n## Findings\n\n")

        for finding in findings:
            report.write(
                f"### {finding.status}: "
                f"`{finding.document.relative_to(ROOT)}:{finding.line}`\n\n"
            )
            report.write(f"Category: `{finding.category}`\n\n")
            report.write(f"> {finding.claim}\n\n")
            report.write(f"{finding.explanation}\n\n")

            if finding.evidence:
                report.write("Evidence:\n\n")

                for item in finding.evidence[:12]:
                    report.write(
                        f"- `{item.path.relative_to(ROOT)}:{item.line}` — "
                        f"`{item.text.replace('`', chr(39))}`\n"
                    )

                report.write("\n")

        report.write("## Required manual review\n\n")
        report.write(
            "Automated text matching cannot prove every semantic or security "
            "claim. Review every `QUALIFY`, `INFERRED`, and `UNSUPPORTED` "
            "finding against the complete handler and account constraints.\n"
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Apply conservative wording corrections.",
    )
    parser.add_argument(
        "--report",
        type=Path,
        required=True,
    )
    args = parser.parse_args()

    corpus = load_corpus()
    findings: list[Finding] = []

    for document in markdown_documents():
        for line, claim in claim_lines(document):
            category = category_for(claim)

            if category is None:
                continue

            findings.append(
                evaluate(
                    document=document,
                    line=line,
                    claim=claim,
                    category=category,
                    corpus=corpus,
                )
            )

    changed = apply_safe_patches() if args.apply else []

    report_path = (
        args.report
        if args.report.is_absolute()
        else ROOT / args.report
    )
    report_path.parent.mkdir(parents=True, exist_ok=True)

    write_report(
        findings=findings,
        changed=changed,
        report_path=report_path,
    )

    print(f"Report: {report_path.relative_to(ROOT)}")
    print(f"Findings: {len(findings)}")
    print(f"Changed files: {len(changed)}")

    unsupported = sum(
        finding.status == "UNSUPPORTED"
        for finding in findings
    )
    qualified = sum(
        finding.status == "QUALIFY"
        for finding in findings
    )
    inferred = sum(
        finding.status == "INFERRED"
        for finding in findings
    )

    print(f"Unsupported: {unsupported}")
    print(f"Require qualification: {qualified}")
    print(f"Inferred: {inferred}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
