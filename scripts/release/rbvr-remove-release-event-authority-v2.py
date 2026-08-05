from pathlib import Path
from datetime import datetime, timezone
import re
import shutil

ROOT = Path.cwd()
SRC = ROOT / "programs/treasury-router/src"

EVENT_NAMES = [
    "ReserveExecutionAuthorized",
    "LiquidityExecutionAuthorized",
    "CompanyExecutionAuthorized",
    "FounderExecutionAuthorized",
    "BuybackExecutionAuthorized",
]

HANDLERS = [
    SRC / "instructions/reserve.rs",
    SRC / "instructions/liquidity.rs",
    SRC / "instructions/company.rs",
    SRC / "instructions/founder.rs",
    SRC / "instructions/buyback.rs",
]

for path in HANDLERS:
    if not path.exists():
        raise RuntimeError(f"Missing handler: {path}")

# ------------------------------------------------------------
# Locate every event struct automatically.
# ------------------------------------------------------------

rust_files = list(SRC.rglob("*.rs"))
event_locations = {}

for event_name in EVENT_NAMES:
    matches = []

    pattern = re.compile(
        rf"\bpub\s+struct\s+{re.escape(event_name)}\s*\{{"
    )

    for path in rust_files:
        text = path.read_text()

        if pattern.search(text):
            matches.append(path)

    if len(matches) != 1:
        raise RuntimeError(
            f"{event_name}: expected exactly one definition, "
            f"found {len(matches)}: {matches}"
        )

    event_locations[event_name] = matches[0]

print("Located release events:")

for event_name, path in event_locations.items():
    print(f"  {event_name}: {path.relative_to(ROOT)}")

files_to_backup = set(HANDLERS)
files_to_backup.update(event_locations.values())

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup = ROOT / f".release-event-authority-backup-v2-{timestamp}"
backup.mkdir(parents=True, exist_ok=False)

for source in files_to_backup:
    destination = backup / source.relative_to(ROOT)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)

print(f"Backup created: {backup}")

# ------------------------------------------------------------
# Remove authority field from each event struct.
# ------------------------------------------------------------

for event_name, path in event_locations.items():
    text = path.read_text()

    struct_pattern = re.compile(
        rf"""
        (
            pub\s+struct\s+{re.escape(event_name)}\s*\{{
        )
        (?P<body>.*?)
        (
            \n\}}
        )
        """,
        re.VERBOSE | re.DOTALL,
    )

    match = struct_pattern.search(text)

    if not match:
        raise RuntimeError(
            f"{path}: could not parse {event_name}"
        )

    body = match.group("body")

    new_body, removed = re.subn(
        r"^[ \t]*pub[ \t]+authority:[ \t]*Pubkey,[ \t]*\n",
        "",
        body,
        count=1,
        flags=re.MULTILINE,
    )

    if removed != 1:
        raise RuntimeError(
            f"{event_name}: expected one authority field; "
            f"removed {removed}"
        )

    updated = (
        text[:match.start()]
        + match.group(1)
        + new_body
        + match.group(3)
        + text[match.end():]
    )

    path.write_text(updated)

    print(f"Removed authority field from {event_name}")

# ------------------------------------------------------------
# Remove authority assignment from each emit! call.
# ------------------------------------------------------------

for path in HANDLERS:
    text = path.read_text()

    updated, removed = re.subn(
        r"""
        ^[ \t]*
        authority:
        [ \t]*
        ctx\.accounts\.authority\.key\(\),
        [ \t]*\n
        """,
        "",
        text,
        count=1,
        flags=re.MULTILINE | re.VERBOSE,
    )

    if removed != 1:
        raise RuntimeError(
            f"{path}: expected one event authority assignment; "
            f"removed {removed}"
        )

    path.write_text(updated)
    print(f"Removed event attribution from {path.name}")

# ------------------------------------------------------------
# Verify.
# ------------------------------------------------------------

for event_name, path in event_locations.items():
    text = path.read_text()

    struct_pattern = re.compile(
        rf"""
        pub\s+struct\s+{re.escape(event_name)}\s*\{{
        (?P<body>.*?)
        \n\}}
        """,
        re.VERBOSE | re.DOTALL,
    )

    match = struct_pattern.search(text)

    if not match:
        raise RuntimeError(
            f"Verification failed: lost {event_name}"
        )

    if re.search(
        r"\bpub\s+authority\s*:",
        match.group("body"),
    ):
        raise RuntimeError(
            f"Verification failed: authority remains in {event_name}"
        )

for path in HANDLERS:
    text = path.read_text()

    if "authority: ctx.accounts.authority.key()" in text:
        raise RuntimeError(
            f"Verification failed in {path}"
        )

print("Release-event verification passed.")
print(f"Backup retained at: {backup}")
