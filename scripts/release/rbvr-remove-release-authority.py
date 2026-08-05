from pathlib import Path
import re
import shutil
from datetime import datetime, timezone

ROOT = Path.cwd()

HANDLERS = {
    "reserve": ROOT / "programs/treasury-router/src/instructions/reserve.rs",
    "liquidity": ROOT / "programs/treasury-router/src/instructions/liquidity.rs",
    "company": ROOT / "programs/treasury-router/src/instructions/company.rs",
    "founder": ROOT / "programs/treasury-router/src/instructions/founder.rs",
    "buyback": ROOT / "programs/treasury-router/src/instructions/buyback.rs",
}

TESTS = ROOT / "tests/rbvr_protocol.ts"

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup = ROOT / f".release-authority-backup-{timestamp}"
backup.mkdir(parents=True, exist_ok=False)

for source in [*HANDLERS.values(), TESTS]:
    if not source.exists():
        raise RuntimeError(f"Missing required file: {source}")

    destination = backup / source.relative_to(ROOT)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)

print(f"Backup created: {backup}")

# -----------------------------------------------------------------
# Remove the authority Signer account from each autonomous handler.
# -----------------------------------------------------------------

for name, path in HANDLERS.items():
    text = path.read_text()

    if "authorize_autonomous_release" not in text:
        raise RuntimeError(
            f"{path}: autonomous release call is missing; refusing to modify"
        )

    original = text

    # Standard Anchor account field.
    text, removed_fields = re.subn(
        r"""
        \n
        [ \t]*
        (?:\#\[[^\n]*\]\n[ \t]*)*
        pub[ \t]+authority:[ \t]*Signer<'info>,[ \t]*
        """,
        "\n",
        text,
        count=1,
        flags=re.VERBOSE,
    )

    if removed_fields != 1:
        raise RuntimeError(
            f"{path}: expected exactly one authority Signer field; "
            f"removed {removed_fields}"
        )

    # Refuse to leave account constraints referring to a removed authority.
    remaining_constraint_refs = re.findall(
        r"has_one\s*=\s*authority|constraint\s*=.*authority",
        text,
    )

    if remaining_constraint_refs:
        raise RuntimeError(
            f"{path}: account constraints still reference authority"
        )

    # Refuse to leave runtime code using ctx.accounts.authority.
    if "ctx.accounts.authority" in text:
        raise RuntimeError(
            f"{path}: handler still accesses ctx.accounts.authority"
        )

    path.write_text(text)

    print(f"Removed release signer from {name}.rs")

# -----------------------------------------------------------------
# Remove authority accounts from only the five autonomous test calls.
# -----------------------------------------------------------------

tests = TESTS.read_text()

method_blocks = {
    "authorizeReserveExecution": "authority",
    "authorizeLiquidityExecution": "authority",
    "authorizeCompanyExecution": "authority: payer.publicKey",
    "authorizeFounderExecution": "authority: payer.publicKey",
    "authorizeBuybackExecution": "authority",
}

for method, authority_expression in method_blocks.items():
    pattern = re.compile(
        rf"""
        (
            \.{re.escape(method)}\(\)
            \s*
            \.accountsPartial\(\{{
        )
        (?P<body>.*?)
        (
            \}}\)
            \s*
            \.rpc\(\)
        )
        """,
        re.VERBOSE | re.DOTALL,
    )

    match = pattern.search(tests)

    if not match:
        raise RuntimeError(
            f"Could not find accountsPartial block for {method}"
        )

    body = match.group("body")

    if authority_expression == "authority":
        authority_pattern = re.compile(
            r"^[ \t]*authority,[ \t]*\n",
            re.MULTILINE,
        )
    else:
        authority_pattern = re.compile(
            r"^[ \t]*authority:[ \t]*payer\.publicKey,[ \t]*\n",
            re.MULTILINE,
        )

    updated_body, removed = authority_pattern.subn(
        "",
        body,
        count=1,
    )

    if removed != 1:
        raise RuntimeError(
            f"{method}: expected one test authority entry; removed {removed}"
        )

    tests = (
        tests[:match.start()]
        + match.group(1)
        + updated_body
        + match.group(3)
        + tests[match.end():]
    )

TESTS.write_text(tests)

print("Removed release authority accounts from integration tests.")

# -----------------------------------------------------------------
# Verification
# -----------------------------------------------------------------

errors = []

for name, path in HANDLERS.items():
    text = path.read_text()

    if re.search(r"pub\s+authority:\s*Signer<'info>", text):
        errors.append(f"{name}.rs still contains an authority Signer")

    if "ctx.accounts.authority" in text:
        errors.append(f"{name}.rs still accesses ctx.accounts.authority")

    if "authorize_autonomous_release" not in text:
        errors.append(f"{name}.rs lost autonomous release authorization")

if errors:
    raise RuntimeError("\n".join(errors))

print("Source verification passed.")
print(f"Backup retained at: {backup}")
