from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import shutil

ROOT = Path.cwd()
TEST_FILE = ROOT / "tests/rbvr_protocol.ts"

TEST_TITLE = 'it("initializes the protocol state"'
NEXT_TEST_TITLE = '  it("initializes the locked protocol allocation"'
NEW_TEST_TITLE = 'it("rejects a second protocol initialization"'

if not TEST_FILE.exists():
    raise RuntimeError(f"Missing test file: {TEST_FILE}")

text = TEST_FILE.read_text()

if NEW_TEST_TITLE in text:
    raise RuntimeError(
        "The duplicate protocol initialization test already exists."
    )

original_test_start = text.find(TEST_TITLE)

if original_test_start == -1:
    raise RuntimeError(
        'Could not locate: it("initializes the protocol state"'
    )

next_test_start = text.find(NEXT_TEST_TITLE, original_test_start)

if next_test_start == -1:
    raise RuntimeError(
        'Could not locate the following allocation initialization test.'
    )

original_test = text[original_test_start:next_test_start]

transaction_start = original_test.find("await program.methods")

if transaction_start == -1:
    raise RuntimeError(
        "Could not locate the initialization transaction in the test."
    )

rpc_marker = ".rpc();"
transaction_end = original_test.find(rpc_marker, transaction_start)

if transaction_end == -1:
    raise RuntimeError(
        "Could not locate the end of the initialization transaction."
    )

transaction_end += len(rpc_marker)
initialize_transaction = original_test[transaction_start:transaction_end]

# Preserve the transaction exactly as used by the passing initialization test,
# while indenting it inside the new try block.
transaction_lines = initialize_transaction.splitlines()
indented_transaction = "\n".join(
    "      " + line.lstrip() if index == 0 else "      " + line
    for index, line in enumerate(transaction_lines)
)

new_test = f'''  it("rejects a second protocol initialization", async () => {{
    let rejected = false;
    let failureText = "";

    try {{
{indented_transaction}
    }} catch (error) {{
      rejected = true;
      failureText =
        error instanceof Error
          ? `${{error.message}}\\n${{error.stack ?? ""}}`
          : String(error);
    }}

    assert.isTrue(
      rejected,
      "canonical protocol state must not be initialized twice",
    );

    assert.isNotEmpty(
      failureText,
      "the rejected initialization should return an error",
    );

    const protocolStateAfter =
      await program.account.protocolState.fetch(protocolStatePda);

    assert.equal(
      protocolStateAfter.authority.toBase58(),
      authority.toBase58(),
      "failed reinitialization must not alter protocol authority",
    );

    assert.equal(
      protocolStateAfter.version,
      1,
      "failed reinitialization must not alter protocol version",
    );

    assert.isFalse(
      protocolStateAfter.paused,
      "failed reinitialization must not alter protocol pause state",
    );
  }});

'''

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup_root = ROOT / f".double-initialize-test-backup-{timestamp}"
backup_file = backup_root / TEST_FILE.relative_to(ROOT)

backup_file.parent.mkdir(parents=True, exist_ok=False)
shutil.copy2(TEST_FILE, backup_file)

updated = text[:next_test_start] + new_test + text[next_test_start:]
TEST_FILE.write_text(updated)

verification = TEST_FILE.read_text()

required = [
    NEW_TEST_TITLE,
    "canonical protocol state must not be initialized twice",
    "failed reinitialization must not alter protocol authority",
    "failed reinitialization must not alter protocol version",
    "failed reinitialization must not alter protocol pause state",
]

for fragment in required:
    if fragment not in verification:
        shutil.copy2(backup_file, TEST_FILE)
        raise RuntimeError(
            f"Patch verification failed; restored backup. Missing: {fragment}"
        )

print(f"Backup created: {backup_root}")
print("Duplicate initialization test inserted successfully.")
print()
print("Copied initialization transaction:")
print("----------------------------------")
print(initialize_transaction)
