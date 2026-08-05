from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import shutil

ROOT = Path.cwd()
TEST_FILE = ROOT / "tests/rbvr_protocol.ts"

MARKER = "RT-003 — deposit and token-authority assault"
INSERT_BEFORE = '  it("initializes the execution config"'

if not TEST_FILE.exists():
    raise RuntimeError(f"Missing integration test file: {TEST_FILE}")

text = TEST_FILE.read_text()

if MARKER in text:
    raise RuntimeError("RT-003 tests already exist.")

insert_position = text.find(INSERT_BEFORE)

if insert_position == -1:
    raise RuntimeError(
        'Could not locate the "initializes the execution config" test.'
    )

required_symbols = [
    "Keypair",
    "anchor.BN",
    "createMint",
    "getOrCreateAssociatedTokenAccount",
    "getAccount",
    "mintTo",
    "provider",
    "payer",
    "authority",
    "program",
    "protocolStatePda",
    "treasuryStatePda",
    "settlementMint",
    "settlementVaultPda",
    "sourceTokenAccount",
    "TOKEN_PROGRAM_ID",
    "DEPOSIT_AMOUNT",
    "treasury",
    "assertAllTreasuryInvariants",
]

for symbol in required_symbols:
    if symbol not in text:
        raise RuntimeError(
            f"Required integration-test symbol not found: {symbol}"
        )

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup_root = ROOT / f".rt003-backup-{timestamp}"
backup_file = backup_root / TEST_FILE.relative_to(ROOT)

backup_file.parent.mkdir(parents=True, exist_ok=False)
shutil.copy2(TEST_FILE, backup_file)

attack_tests = r'''
  // RT-003 — deposit and token-authority assault

  const assertDepositAccountingUnchanged = async (
    treasuryBefore: Awaited<ReturnType<typeof treasury>>,
    context: string,
  ) => {
    const treasuryAfter = await treasury();

    assert.equal(
      treasuryAfter.totalFeesReceived.toString(),
      treasuryBefore.totalFeesReceived.toString(),
      `${context} changed total received fees`,
    );

    assert.equal(
      treasuryAfter.totalFeesAllocated.toString(),
      treasuryBefore.totalFeesAllocated.toString(),
      `${context} changed total allocated fees`,
    );

    assert.equal(
      treasuryAfter.processingEpoch.toString(),
      treasuryBefore.processingEpoch.toString(),
      `${context} changed processing epoch`,
    );

    assert.equal(
      treasuryAfter.pendingReserve.toString(),
      treasuryBefore.pendingReserve.toString(),
      `${context} changed pending reserve`,
    );

    assert.equal(
      treasuryAfter.pendingBuybackBurn.toString(),
      treasuryBefore.pendingBuybackBurn.toString(),
      `${context} changed pending buyback`,
    );

    assert.equal(
      treasuryAfter.pendingLiquidity.toString(),
      treasuryBefore.pendingLiquidity.toString(),
      `${context} changed pending liquidity`,
    );

    assert.equal(
      treasuryAfter.pendingCompany.toString(),
      treasuryBefore.pendingCompany.toString(),
      `${context} changed pending company`,
    );

    assert.equal(
      treasuryAfter.pendingFounder.toString(),
      treasuryBefore.pendingFounder.toString(),
      `${context} changed pending founder`,
    );

    await assertAllTreasuryInvariants();
  };

  it("RT-003 rejects a source token account owned by another signer", async () => {
    const attacker = Keypair.generate();

    const attackerSource = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        attacker.publicKey,
      )
    ).address;

    await mintTo(
      provider.connection,
      payer,
      settlementMint,
      attackerSource,
      authority,
      BigInt(DEPOSIT_AMOUNT.toString()),
    );

    const treasuryBefore = await treasury();

    const attackerBefore = await getAccount(
      provider.connection,
      attackerSource,
    );

    const vaultBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .depositSettlement(DEPOSIT_AMOUNT)
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,
          settlementMint,
          sourceTokenAccount: attackerSource,
          settlementVault: settlementVaultPda,

          // The supplied signer does not own attackerSource.
          authority: payer.publicKey,

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
      "deposit from an account owned by another signer must be rejected",
    );

    assert.notEqual(
      failureText.length,
      0,
      "wrong-owner deposit should return an error",
    );

    const attackerAfter = await getAccount(
      provider.connection,
      attackerSource,
    );

    const vaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    assert.equal(
      attackerAfter.amount,
      attackerBefore.amount,
      "wrong-owner deposit removed tokens from attacker source",
    );

    assert.equal(
      vaultAfter.amount,
      vaultBefore.amount,
      "wrong-owner deposit added tokens to treasury vault",
    );

    await assertDepositAccountingUnchanged(
      treasuryBefore,
      "wrong-owner deposit",
    );
  });

  it("RT-003 rejects a source token account using the wrong mint", async () => {
    const fakeMint = await createMint(
      provider.connection,
      payer,
      authority,
      null,
      6,
    );

    const fakeSource = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        fakeMint,
        payer.publicKey,
      )
    ).address;

    await mintTo(
      provider.connection,
      payer,
      fakeMint,
      fakeSource,
      authority,
      BigInt(DEPOSIT_AMOUNT.toString()),
    );

    const treasuryBefore = await treasury();

    const fakeSourceBefore = await getAccount(
      provider.connection,
      fakeSource,
    );

    const vaultBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .depositSettlement(DEPOSIT_AMOUNT)
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,

          // The canonical mint is supplied, but fakeSource belongs to fakeMint.
          settlementMint,
          sourceTokenAccount: fakeSource,

          settlementVault: settlementVaultPda,
          authority: payer.publicKey,
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
      "deposit from a token account using the wrong mint must be rejected",
    );

    assert.notEqual(
      failureText.length,
      0,
      "wrong-mint source deposit should return an error",
    );

    const fakeSourceAfter = await getAccount(
      provider.connection,
      fakeSource,
    );

    const vaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    assert.equal(
      fakeSourceAfter.amount,
      fakeSourceBefore.amount,
      "wrong-mint deposit removed fake tokens",
    );

    assert.equal(
      vaultAfter.amount,
      vaultBefore.amount,
      "wrong-mint deposit changed canonical vault balance",
    );

    await assertDepositAccountingUnchanged(
      treasuryBefore,
      "wrong-mint source deposit",
    );
  });

  it("RT-003 rejects a substituted settlement vault", async () => {
    const attacker = Keypair.generate();

    const fakeVault = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        attacker.publicKey,
      )
    ).address;

    const treasuryBefore = await treasury();

    const sourceBefore = await getAccount(
      provider.connection,
      sourceTokenAccount,
    );

    const canonicalVaultBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    const fakeVaultBefore = await getAccount(
      provider.connection,
      fakeVault,
    );

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .depositSettlement(DEPOSIT_AMOUNT)
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,
          settlementMint,
          sourceTokenAccount,

          // Valid SPL token account, but not the canonical treasury vault.
          settlementVault: fakeVault,

          authority: payer.publicKey,
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
      "substituted settlement vault must be rejected",
    );

    assert.notEqual(
      failureText.length,
      0,
      "substituted-vault deposit should return an error",
    );

    const sourceAfter = await getAccount(
      provider.connection,
      sourceTokenAccount,
    );

    const canonicalVaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    const fakeVaultAfter = await getAccount(
      provider.connection,
      fakeVault,
    );

    assert.equal(
      sourceAfter.amount,
      sourceBefore.amount,
      "substituted-vault deposit removed source tokens",
    );

    assert.equal(
      canonicalVaultAfter.amount,
      canonicalVaultBefore.amount,
      "substituted-vault deposit changed canonical vault",
    );

    assert.equal(
      fakeVaultAfter.amount,
      fakeVaultBefore.amount,
      "substituted-vault deposit transferred tokens to attacker vault",
    );

    await assertDepositAccountingUnchanged(
      treasuryBefore,
      "substituted-vault deposit",
    );
  });

  it("RT-003 rejects a zero-value deposit without mutation", async () => {
    const zeroAmount = new anchor.BN(0);

    const treasuryBefore = await treasury();

    const sourceBefore = await getAccount(
      provider.connection,
      sourceTokenAccount,
    );

    const vaultBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .depositSettlement(zeroAmount)
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,
          settlementMint,
          sourceTokenAccount,
          settlementVault: settlementVaultPda,
          authority: payer.publicKey,
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
      "zero-value deposit must be rejected",
    );

    assert.notEqual(
      failureText.length,
      0,
      "zero-value deposit should return an error",
    );

    const sourceAfter = await getAccount(
      provider.connection,
      sourceTokenAccount,
    );

    const vaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    assert.equal(
      sourceAfter.amount,
      sourceBefore.amount,
      "zero-value deposit changed source balance",
    );

    assert.equal(
      vaultAfter.amount,
      vaultBefore.amount,
      "zero-value deposit changed vault balance",
    );

    await assertDepositAccountingUnchanged(
      treasuryBefore,
      "zero-value deposit",
    );
  });

  it("RT-003 rejects a deposit exceeding the source balance", async () => {
    const sourceBefore = await getAccount(
      provider.connection,
      sourceTokenAccount,
    );

    const excessiveAmount = new anchor.BN(
      (sourceBefore.amount + 1n).toString(),
    );

    const treasuryBefore = await treasury();

    const vaultBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .depositSettlement(excessiveAmount)
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,
          settlementMint,
          sourceTokenAccount,
          settlementVault: settlementVaultPda,
          authority: payer.publicKey,
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
      "deposit exceeding source balance must be rejected",
    );

    assert.notEqual(
      failureText.length,
      0,
      "insufficient-balance deposit should return an error",
    );

    const sourceAfter = await getAccount(
      provider.connection,
      sourceTokenAccount,
    );

    const vaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    assert.equal(
      sourceAfter.amount,
      sourceBefore.amount,
      "insufficient-balance deposit changed source balance",
    );

    assert.equal(
      vaultAfter.amount,
      vaultBefore.amount,
      "insufficient-balance deposit changed vault balance",
    );

    await assertDepositAccountingUnchanged(
      treasuryBefore,
      "insufficient-balance deposit",
    );
  });

'''

updated = text[:insert_position] + attack_tests + text[insert_position:]
TEST_FILE.write_text(updated)

verification = TEST_FILE.read_text()

required_fragments = [
    MARKER,
    "RT-003 rejects a source token account owned by another signer",
    "RT-003 rejects a source token account using the wrong mint",
    "RT-003 rejects a substituted settlement vault",
    "RT-003 rejects a zero-value deposit without mutation",
    "RT-003 rejects a deposit exceeding the source balance",
]

for fragment in required_fragments:
    if fragment not in verification:
        shutil.copy2(backup_file, TEST_FILE)
        raise RuntimeError(
            f"Patch verification failed; backup restored. Missing: {fragment}"
        )

print(f"Backup created: {backup_root}")
print("RT-003 deposit and token-authority attacks inserted.")
print("Attack tests added: 5")
print("Expected integration test total: 36")
