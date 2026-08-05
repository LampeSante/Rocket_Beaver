from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import shutil

ROOT = Path.cwd()
TEST_FILE = ROOT / "tests/rbvr_protocol.ts"

MARKER = "RT-002C — linked-account and PDA substitution assault"
INSERT_BEFORE = '  it("autonomously transfers the reserve allocation"'

if not TEST_FILE.exists():
    raise RuntimeError(f"Missing integration test file: {TEST_FILE}")

text = TEST_FILE.read_text()

if MARKER in text:
    raise RuntimeError("RT-002C tests already exist.")

insert_position = text.find(INSERT_BEFORE)

if insert_position == -1:
    raise RuntimeError(
        'Could not locate the "autonomously transfers the reserve allocation" test.'
    )

required_symbols = [
    "protocolStatePda",
    "protocolConfigPda",
    "treasuryStatePda",
    "founderStatePda",
    "companyStatePda",
    "executionConfigPda",
    "settlementMint",
    "settlementVaultPda",
    "reserveDestination",
    "buybackDestination",
    "liquidityDestination",
    "companyDestination",
    "founderDestination",
    "TOKEN_PROGRAM_ID",
    "treasury",
    "assertAllTreasuryInvariants",
]

for symbol in required_symbols:
    if symbol not in text:
        raise RuntimeError(
            f"Required integration-test symbol not found: {symbol}"
        )

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup_root = ROOT / f".rt002c-backup-{timestamp}"
backup_file = backup_root / TEST_FILE.relative_to(ROOT)

backup_file.parent.mkdir(parents=True, exist_ok=False)
shutil.copy2(TEST_FILE, backup_file)

attack_tests = r'''
  // RT-002C — linked-account and PDA substitution assault

  it("RT-002C rejects a substituted protocol-config PDA without mutation", async () => {
    const treasuryBefore = await treasury();

    const vaultBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .processFees()
        .accountsPartial({
          protocolState: protocolStatePda,

          // Hostile substitution:
          // execution_config is supplied where protocol_config is required.
          protocolConfig: executionConfigPda,

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

    assert.equal(
      rejected,
      true,
      "substituted protocol-config PDA must be rejected",
    );

    assert.notEqual(
      failureText.length,
      0,
      "substituted protocol-config PDA should return an error",
    );

    const treasuryAfter = await treasury();

    const vaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    assert.equal(
      treasuryAfter.totalFeesReceived.toString(),
      treasuryBefore.totalFeesReceived.toString(),
      "failed protocol-config substitution changed total received fees",
    );

    assert.equal(
      treasuryAfter.totalFeesAllocated.toString(),
      treasuryBefore.totalFeesAllocated.toString(),
      "failed protocol-config substitution changed total allocated fees",
    );

    assert.equal(
      treasuryAfter.processingEpoch.toString(),
      treasuryBefore.processingEpoch.toString(),
      "failed protocol-config substitution changed processing epoch",
    );

    assert.equal(
      treasuryAfter.pendingReserve.toString(),
      treasuryBefore.pendingReserve.toString(),
      "failed protocol-config substitution changed reserve accounting",
    );

    assert.equal(
      treasuryAfter.pendingBuybackBurn.toString(),
      treasuryBefore.pendingBuybackBurn.toString(),
      "failed protocol-config substitution changed buyback accounting",
    );

    assert.equal(
      treasuryAfter.pendingLiquidity.toString(),
      treasuryBefore.pendingLiquidity.toString(),
      "failed protocol-config substitution changed liquidity accounting",
    );

    assert.equal(
      treasuryAfter.pendingCompany.toString(),
      treasuryBefore.pendingCompany.toString(),
      "failed protocol-config substitution changed company accounting",
    );

    assert.equal(
      treasuryAfter.pendingFounder.toString(),
      treasuryBefore.pendingFounder.toString(),
      "failed protocol-config substitution changed founder accounting",
    );

    assert.equal(
      vaultAfter.amount,
      vaultBefore.amount,
      "failed protocol-config substitution changed vault balance",
    );

    await assertAllTreasuryInvariants();
  });

  it("RT-002C rejects swapped founder and company PDAs without mutation", async () => {
    const treasuryBefore = await treasury();

    const vaultBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .processFees()
        .accountsPartial({
          protocolState: protocolStatePda,
          protocolConfig: protocolConfigPda,
          treasury: treasuryStatePda,

          // Hostile linked-account substitution:
          // canonical founder and company PDAs are deliberately swapped.
          founderState: companyStatePda,
          companyState: founderStatePda,

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

    assert.equal(
      rejected,
      true,
      "swapped founder/company PDAs must be rejected",
    );

    assert.notEqual(
      failureText.length,
      0,
      "swapped founder/company PDAs should return an error",
    );

    const treasuryAfter = await treasury();

    const vaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    assert.equal(
      treasuryAfter.totalFeesReceived.toString(),
      treasuryBefore.totalFeesReceived.toString(),
      "failed founder/company substitution changed received fees",
    );

    assert.equal(
      treasuryAfter.totalFeesAllocated.toString(),
      treasuryBefore.totalFeesAllocated.toString(),
      "failed founder/company substitution changed allocated fees",
    );

    assert.equal(
      treasuryAfter.processingEpoch.toString(),
      treasuryBefore.processingEpoch.toString(),
      "failed founder/company substitution changed processing epoch",
    );

    assert.equal(
      treasuryAfter.pendingReserve.toString(),
      treasuryBefore.pendingReserve.toString(),
      "failed founder/company substitution changed reserve accounting",
    );

    assert.equal(
      treasuryAfter.pendingLiquidity.toString(),
      treasuryBefore.pendingLiquidity.toString(),
      "failed founder/company substitution changed liquidity accounting",
    );

    assert.equal(
      treasuryAfter.pendingCompany.toString(),
      treasuryBefore.pendingCompany.toString(),
      "failed founder/company substitution changed company accounting",
    );

    assert.equal(
      treasuryAfter.pendingFounder.toString(),
      treasuryBefore.pendingFounder.toString(),
      "failed founder/company substitution changed founder accounting",
    );

    assert.equal(
      vaultAfter.amount,
      vaultBefore.amount,
      "failed founder/company substitution changed vault balance",
    );

    await assertAllTreasuryInvariants();
  });

  it("RT-002C rejects a substituted execution-config PDA without token movement", async () => {
    const treasuryBefore = await treasury();

    const vaultBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    const reserveBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .authorizeReserveExecution()
        .accountsPartial({
          protocolState: protocolStatePda,
          protocolConfig: protocolConfigPda,
          treasury: treasuryStatePda,
          founderState: founderStatePda,
          companyState: companyStatePda,
          settlementMint,
          settlementVault: settlementVaultPda,
          reserveDestination,
          buybackDestination,
          liquidityDestination,
          companyDestination,
          founderDestination,

          // Hostile substitution:
          // protocol_config is supplied where execution_config is required.
          executionConfig: protocolConfigPda,

          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
    } catch (error) {
      rejected = true;
      failureText =
        error instanceof Error
          ? `${error.message}\n${error.stack ?? ""}`
          : String(error);
    }

    assert.equal(
      rejected,
      true,
      "substituted execution-config PDA must be rejected",
    );

    assert.notEqual(
      failureText.length,
      0,
      "substituted execution-config PDA should return an error",
    );

    const treasuryAfter = await treasury();

    const vaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    const reserveAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    assert.equal(
      treasuryAfter.pendingReserve.toString(),
      treasuryBefore.pendingReserve.toString(),
      "failed execution-config substitution changed pending reserve",
    );

    assert.equal(
      treasuryAfter.releasedReserve.toString(),
      treasuryBefore.releasedReserve.toString(),
      "failed execution-config substitution changed released reserve",
    );

    assert.equal(
      treasuryAfter.processingEpoch.toString(),
      treasuryBefore.processingEpoch.toString(),
      "failed execution-config substitution changed processing epoch",
    );

    assert.equal(
      vaultAfter.amount,
      vaultBefore.amount,
      "failed execution-config substitution changed vault balance",
    );

    assert.equal(
      reserveAfter.amount,
      reserveBefore.amount,
      "failed execution-config substitution transferred reserve tokens",
    );

    await assertAllTreasuryInvariants();
  });

  it("RT-002C rejects a substituted protocol-state PDA without token movement", async () => {
    const treasuryBefore = await treasury();

    const vaultBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    const reserveBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .authorizeReserveExecution()
        .accountsPartial({
          // Hostile substitution:
          // execution_config is supplied where canonical protocol_state
          // is required.
          protocolState: executionConfigPda,

          protocolConfig: protocolConfigPda,
          treasury: treasuryStatePda,
          founderState: founderStatePda,
          companyState: companyStatePda,
          settlementMint,
          settlementVault: settlementVaultPda,
          reserveDestination,
          buybackDestination,
          liquidityDestination,
          companyDestination,
          founderDestination,
          executionConfig: executionConfigPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
    } catch (error) {
      rejected = true;
      failureText =
        error instanceof Error
          ? `${error.message}\n${error.stack ?? ""}`
          : String(error);
    }

    assert.equal(
      rejected,
      true,
      "substituted protocol-state PDA must be rejected",
    );

    assert.notEqual(
      failureText.length,
      0,
      "substituted protocol-state PDA should return an error",
    );

    const treasuryAfter = await treasury();

    const vaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    const reserveAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    assert.equal(
      treasuryAfter.pendingReserve.toString(),
      treasuryBefore.pendingReserve.toString(),
      "failed protocol-state substitution changed pending reserve",
    );

    assert.equal(
      treasuryAfter.releasedReserve.toString(),
      treasuryBefore.releasedReserve.toString(),
      "failed protocol-state substitution changed released reserve",
    );

    assert.equal(
      treasuryAfter.totalFeesAllocated.toString(),
      treasuryBefore.totalFeesAllocated.toString(),
      "failed protocol-state substitution changed allocated fees",
    );

    assert.equal(
      vaultAfter.amount,
      vaultBefore.amount,
      "failed protocol-state substitution changed vault balance",
    );

    assert.equal(
      reserveAfter.amount,
      reserveBefore.amount,
      "failed protocol-state substitution transferred reserve tokens",
    );

    await assertAllTreasuryInvariants();
  });

'''

updated = text[:insert_position] + attack_tests + text[insert_position:]
TEST_FILE.write_text(updated)

verification = TEST_FILE.read_text()

required_fragments = [
    MARKER,
    "RT-002C rejects a substituted protocol-config PDA without mutation",
    "RT-002C rejects swapped founder and company PDAs without mutation",
    "RT-002C rejects a substituted execution-config PDA without token movement",
    "RT-002C rejects a substituted protocol-state PDA without token movement",
]

for fragment in required_fragments:
    if fragment not in verification:
        shutil.copy2(backup_file, TEST_FILE)
        raise RuntimeError(
            f"Patch verification failed; backup restored. Missing: {fragment}"
        )

print(f"Backup created: {backup_root}")
print("RT-002C linked-account and PDA substitution attacks inserted.")
print("Attack tests added: 4")
print("Expected integration test total: 31")
