from pathlib import Path
from datetime import datetime, timezone
import re
import shutil

ROOT = Path.cwd()

EVENTS = ROOT / "programs/treasury-router/src/events.rs"

HANDLERS = [
    ROOT / "programs/treasury-router/src/instructions/reserve.rs",
    ROOT / "programs/treasury-router/src/instructions/liquidity.rs",
    ROOT / "programs/treasury-router/src/instructions/company.rs",
    ROOT / "programs/treasury-router/src/instructions/founder.rs",
    ROOT / "programs/treasury-router/src/instructions/buyback.rs",
]

FILES = [EVENTS, *HANDLERS]

for path in FILES:
    if not path.exists():
        raise RuntimeError(f"Missing required file: {path}")

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup = ROOT / f".release-event-authority-backup-{timestamp}"
backup.mkdir(parents=True, exist_ok=False)

for source in FILES:
    destination = backup / source.relative_to(ROOT)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)

print(f"Backup created: {backup}")

events = EVENTS.read_text()

event_names = [
    "ReserveExecutionAuthorized",
    "LiquidityExecutionAuthorized",
    "CompanyExecutionAuthorized",
    "FounderExecutionAuthorized",
    "BuybackExecutionAuthorized",
]

for event_name in event_names:
    pattern = re.compile(
        rf"""
        (
            pub[ \t]+struct[ \t]+{re.escape(event_name)}[ \t]*\{{
        )
        (?P<body>.*?)
        (
            \n\}}
        )
        """,
        re.VERBOSE | re.DOTALL,
    )

    match = pattern.search(events)

    if not match:
        raise RuntimeError(f"Could not find event struct {event_name}")

    body = match.group("body")

    updated_body, removed = re.subn(
        r"^[ \t]*pub[ \t]+authority:[ \t]*Pubkey,[ \t]*\n",
        "",
        body,
        count=1,
        flags=re.MULTILINE,
    )

    if removed != 1:
        raise RuntimeError(
            f"{event_name}: expected exactly one authority field; "
            f"removed {removed}"
        )

    events = (
        events[:match.start()]
        + match.group(1)
        + updated_body
        + match.group(3)
        + events[match.end():]
    )

EVENTS.write_text(events)

print("Removed authority fields from five autonomous release events.")

for path in HANDLERS:
    text = path.read_text()

    updated, removed = re.subn(
        r"^[ \t]*authority:[ \t]*ctx\.accounts\.authority\.key\(\),[ \t]*\n",
        "",
        text,
        count=1,
        flags=re.MULTILINE,
    )

    if removed != 1:
        raise RuntimeError(
            f"{path}: expected exactly one event authority assignment; "
            f"removed {removed}"
        )

    path.write_text(updated)

    print(f"Removed event authority assignment from {path.name}")

# Final verification
events = EVENTS.read_text()

for event_name in event_names:
    pattern = re.compile(
        rf"pub[ \t]+struct[ \t]+{re.escape(event_name)}[ \t]*\{{"
        rf"(?P<body>.*?)\n\}}",
        re.DOTALL,
    )

    match = pattern.search(events)

    if not match:
        raise RuntimeError(f"Lost event struct {event_name}")

    if re.search(r"pub\s+authority\s*:", match.group("body")):
        raise RuntimeError(
            f"{event_name} still contains an authority field"
        )

for path in HANDLERS:
    text = path.read_text()

    if "authority: ctx.accounts.authority.key()" in text:
        raise RuntimeError(
            f"{path} still contains the event authority assignment"
        )

print("Release-event verification passed.")
print(f"Backup retained at: {backup}")
