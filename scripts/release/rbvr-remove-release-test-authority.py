from pathlib import Path
from datetime import datetime, timezone
import shutil
import re

ROOT = Path.cwd()
TESTS = ROOT / "tests/rbvr_protocol.ts"

if not TESTS.exists():
    raise RuntimeError(f"Missing integration test file: {TESTS}")

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup = ROOT / f".release-test-authority-backup-{timestamp}"
backup.mkdir(parents=True, exist_ok=False)

destination = backup / TESTS.relative_to(ROOT)
destination.parent.mkdir(parents=True, exist_ok=True)
shutil.copy2(TESTS, destination)

print(f"Backup created: {backup}")

text = TESTS.read_text()

# Match each .accountsPartial({ ... }) account block.
block_pattern = re.compile(
    r"""
    (?P<prefix>\.accountsPartial\s*\(\s*\{\s*\n)
    (?P<body>.*?)
    (?P<suffix>^[ \t]*\}\s*\))
    """,
    re.VERBOSE | re.DOTALL | re.MULTILINE,
)

matches = list(block_pattern.finditer(text))

if not matches:
    raise RuntimeError("No .accountsPartial({...}) blocks were found")

output = []
cursor = 0
changed_blocks = 0
removed_entries = 0

for match in matches:
    output.append(text[cursor:match.start()])

    prefix = match.group("prefix")
    body = match.group("body")
    suffix = match.group("suffix")

    # Release instructions use the locked execution configuration and the
    # token program. Initialization/configuration blocks do not match both.
    is_release_block = (
        re.search(r"^\s*executionConfig\s*:", body, re.MULTILINE)
        and re.search(r"^\s*tokenProgram\s*:", body, re.MULTILINE)
    )

    if is_release_block:
        updated_body, count = re.subn(
            r"^[ \t]*authority,[ \t]*\n",
            "",
            body,
            count=1,
            flags=re.MULTILINE,
        )

        if count == 1:
            body = updated_body
            changed_blocks += 1
            removed_entries += 1
        elif re.search(r"^[ \t]*authority,[ \t]*$", body, re.MULTILINE):
            raise RuntimeError(
                "A release account block contains authority, but it "
                "could not be removed safely"
            )

    output.append(prefix)
    output.append(body)
    output.append(suffix)

    cursor = match.end()

output.append(text[cursor:])
updated_text = "".join(output)

if removed_entries == 0:
    raise RuntimeError(
        "No release authority entries were removed. "
        "The test layout may have changed."
    )

TESTS.write_text(updated_text)

# Verification: no release block may still contain authority.
verification_text = TESTS.read_text()

for match in block_pattern.finditer(verification_text):
    body = match.group("body")

    is_release_block = (
        re.search(r"^\s*executionConfig\s*:", body, re.MULTILINE)
        and re.search(r"^\s*tokenProgram\s*:", body, re.MULTILINE)
    )

    if is_release_block and re.search(
        r"^[ \t]*authority,[ \t]*$",
        body,
        re.MULTILINE,
    ):
        raise RuntimeError(
            "Verification failed: authority remains in a release block"
        )

# Make sure legitimate authority uses were not globally deleted.
required_legitimate_uses = [
    ".initialize()",
    ".initializeProtocolConfig()",
    ".initializeTreasury()",
    ".initializeFounder(",
    ".initializeCompany(",
    ".depositSettlement(",
]

for required in required_legitimate_uses:
    if required not in verification_text:
        raise RuntimeError(
            f"Verification failed: expected test call missing: {required}"
        )

print(
    f"Removed {removed_entries} obsolete release authority "
    f"account entr{'y' if removed_entries == 1 else 'ies'} "
    f"from {changed_blocks} release block"
    f"{'' if changed_blocks == 1 else 's'}."
)

print("Legitimate initialization and deposit authority uses were preserved.")
print("Release-test verification passed.")
print(f"Backup retained at: {backup}")
