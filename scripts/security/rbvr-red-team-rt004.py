from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import shutil

ROOT = Path.cwd()
TEST_FILE = ROOT / "tests/rbvr_protocol.ts"

MARKER = "RT-004 — fee-processing and accounting assault"
TARGET_TEST = '  it("processes fees using the locked 30/20/20/20/10 model"'

if not TEST_FILE.exists():
    raise RuntimeError(f"Missing integration test file: {TEST_FILE}")

text = TEST_FILE.read_text()

if MARKER in text:
    raise RuntimeError("RT-004 integration test already exists.")

start = text.find(TARGET_TEST)

if start == -1:
    raise RuntimeError(
        'Could not locate the normal "processes fees" integration test.'
    )

# Find the next top-level integration test after the normal process-fees test.
next_test = text.find('\n  it("', start + len(TARGET_TEST))

if next_test == -1:
    raise RuntimeError(
        "Could not locate the insertion point after the process-fees test."
    )

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup_root = ROOT / f".rt004-backup-{timestamp}"
backup_file = backup_root / TEST_FILE.relative_to(ROOT)

backup_file.parent.mkdir(parents=True, exist_ok=False)
shutil.copy2(TEST_FILE, backup_file)

attack_test = r'''
  // RT-004 — fee-processing and accounting assault
  it("RT-004 rejects fee-processing replay without mutating accounting", async () => {
    const treasuryBefore = await treasury();

    const founderBefore =
      await program.account.founderState.fetch(founderStatePda);

    const companyBefore =
      await program.account.companyState.fetch(companyStatePda);

    const vaultBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    let rejected = false;
    let failureText = "";

    try {
      /*
       * The complete vault balance was already processed by the preceding
       * successful processFees call. Replaying processFees without depositing
       * new settlement assets must fail closed.
       */
      await program.methods
        .processFees()
        .accountsPartial({
          protocolState: protocolStatePda,
          protocolConfig: protocolConfigPda,
          treasury: treasuryStatePda,
          founderState: founderStatePda,
          companyState: companyStatePda,
          settlementVault: settlementVaultPda,
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
      "Replaying processFees without new funds must be rejected",
    );

    assert.match(
      failureText,
      /NoUnprocessedFees|no unprocessed fees/i,
      `Unexpected replay rejection: ${failureText}`,
    );

    const treasuryAfter = await treasury();

    const founderAfter =
      await program.account.founderState.fetch(founderStatePda);

    const companyAfter =
      await program.account.companyState.fetch(companyStatePda);

    const vaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    const treasuryFields = [
      "totalFeesReceived",
      "totalFeesAllocated",
      "pendingReserve",
      "pendingBuybackBurn",
      "pendingLiquidity",
      "pendingCompany",
      "pendingFounder",
      "lifetimeReserve",
      "lifetimeBuybackBurn",
      "lifetimeLiquidity",
      "lifetimeCompany",
      "lifetimeFounder",
      "releasedReserve",
      "releasedBuybackBurn",
      "releasedLiquidity",
      "releasedCompany",
      "releasedFounder",
      "processingEpoch",
    ] as const;

    for (const field of treasuryFields) {
      assert.equal(
        treasuryAfter[field].toString(),
        treasuryBefore[field].toString(),
        `Rejected fee replay mutated treasury.${field}`,
      );
    }

    const founderFields = [
      "earnedCurrentPeriod",
      "lifetimeEarned",
      "periodStartedAt",
    ] as const;

    for (const field of founderFields) {
      assert.equal(
        founderAfter[field].toString(),
        founderBefore[field].toString(),
        `Rejected fee replay mutated founderState.${field}`,
      );
    }

    const companyFields = [
      "spentCurrentPeriod",
      "lifetimeSpent",
      "periodStartedAt",
    ] as const;

    for (const field of companyFields) {
      assert.equal(
        companyAfter[field].toString(),
        companyBefore[field].toString(),
        `Rejected fee replay mutated companyState.${field}`,
      );
    }

    assert.equal(
      vaultAfter.amount.toString(),
      vaultBefore.amount.toString(),
      "Rejected fee replay changed the settlement-vault balance",
    );

    await assertAllTreasuryInvariants();
  });
'''

updated = text[:next_test] + "\n" + attack_test.rstrip() + text[next_test:]
TEST_FILE.write_text(updated)

verification = TEST_FILE.read_text()

required = [
    MARKER,
    "RT-004 rejects fee-processing replay without mutating accounting",
    "NoUnprocessedFees",
    "Rejected fee replay mutated treasury",
    "Rejected fee replay changed the settlement-vault balance",
]

for fragment in required:
    if fragment not in verification:
        shutil.copy2(backup_file, TEST_FILE)
        raise RuntimeError(
            f"RT-004 verification failed. Missing: {fragment}. "
            "Original test file restored."
        )

print(f"Backup created: {backup_root}")
print("RT-004 fee-processing replay attack inserted.")
print("New integration attacks: 1")
print("Expected integration test total: 37")
