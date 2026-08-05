from pathlib import Path
from datetime import datetime, timezone
import re
import shutil

ROOT = Path.cwd()
TESTS = ROOT / "tests/rbvr_protocol.ts"

if not TESTS.exists():
    raise RuntimeError(f"Missing test file: {TESTS}")

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup = ROOT / f".double-initialize-test-backup-{timestamp}"
backup.mkdir(parents=True, exist_ok=False)

destination = backup / TESTS.relative_to(ROOT)
destination.parent.mkdir(parents=True, exist_ok=True)
shutil.copy2(TESTS, destination)

print(f"Backup created: {backup}")

text = TESTS.read_text()

if "rejects a second protocol initialization" in text:
    raise RuntimeError(
        "The double-initialization test already appears to exist"
    )

anchor = re.compile(
    r'''
    (
      \s*it\("initializes the protocol state",\s*async\s*\(\)\s*=>\s*\{
      .*?
      \n\s*\}\);
    )
    ''',
    re.VERBOSE | re.DOTALL,
)

match = anchor.search(text)

if not match:
    raise RuntimeError(
        'Could not locate the "initializes the protocol state" test'
    )

new_test = r'''

  it("rejects a second protocol initialization", async () => {
    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .initialize()
        .accountsPartial({
          protocolState: protocolStatePda,
          authority,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch (error) {
      rejected = true;
      failureText =
        error instanceof Error
          ? `${error.message}\n${error.stack ?? ""}`
          : String(error);
    }

    assert.isTrue(
      rejected,
      "canonical protocol state must not be initialized twice",
    );

    assert.match(
      failureText,
      /already in use|already initialized|custom program error|0x0/i,
      "second initialization should fail because the canonical PDA already exists",
    );

    const state =
      await program.account.protocolState.fetch(protocolStatePda);

    assert.equal(
      state.authority.toBase58(),
      authority.toBase58(),
      "failed reinitialization must not alter protocol authority",
    );

    assert.equal(
      state.version,
      1,
      "failed reinitialization must not alter protocol version",
    );

    assert.isFalse(
      state.paused,
      "failed reinitialization must not alter protocol pause state",
    );
  });
'''

updated = (
    text[:match.end()]
    + new_test
    + text[match.end():]
)

TESTS.write_text(updated)

verification = TESTS.read_text()

required_fragments = [
    'it("rejects a second protocol initialization"',
    "canonical protocol state must not be initialized twice",
    "failed reinitialization must not alter protocol authority",
    "failed reinitialization must not alter protocol version",
    "failed reinitialization must not alter protocol pause state",
]

for fragment in required_fragments:
    if fragment not in verification:
        raise RuntimeError(
            f"Verification failed; missing fragment: {fragment}"
        )

print("Added explicit canonical protocol reinitialization test.")
print("Verification passed.")
print(f"Backup retained at: {backup}")
