from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import shutil

ROOT = Path.cwd()
TEST_FILE = ROOT / "tests/rbvr_protocol.ts"

MARKER = "RT-002B — account substitution assault"
INSERT_BEFORE = 'it("autonomously transfers the reserve allocation"'

if not TEST_FILE.exists():
    raise RuntimeError(f"Missing integration test file: {TEST_FILE}")

text = TEST_FILE.read_text()

if MARKER in text:
    raise RuntimeError("RT-002B tests already exist.")

position = text.find(INSERT_BEFORE)

if position == -1:
    raise RuntimeError(
        'Could not locate the reserve execution test insertion point.'
    )

line_start = text.rfind("\n", 0, position) + 1

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup_root = ROOT / f".rt002b-backup-{timestamp}"
backup_file = backup_root / TEST_FILE.relative_to(ROOT)

backup_file.parent.mkdir(parents=True, exist_ok=False)
shutil.copy2(TEST_FILE, backup_file)

print(f"Backup created: {backup_root}")

block = r'''
  // RT-002B — account substitution assault
  it("RT-002B rejects a fake settlement mint without mutation", async () => {
    const fakeMint = await createMint(
      provider.connection,
      payer,
      authority,
      null,
      6,
    );

    const treasuryBefore =
      await provider.connection.getAccountInfo(treasuryStatePda);

    const vaultBefore =
      await provider.connection.getAccountInfo(settlementVaultPda);

    const destinationBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    assert.isNotNull(treasuryBefore);
    assert.isNotNull(vaultBefore);

    let rejected = false;

    try {
      await program.methods
        .authorizeReserveExecution()
        .accountsPartial({
          protocolState: protocolStatePda,
          protocolConfig: protocolConfigPda,
          treasury: treasuryStatePda,
          founderState: founderStatePda,
          companyState: companyStatePda,
          settlementMint: fakeMint,
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
    } catch {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "reserve execution must reject a substituted settlement mint",
    );

    const treasuryAfter =
      await provider.connection.getAccountInfo(treasuryStatePda);

    const vaultAfter =
      await provider.connection.getAccountInfo(settlementVaultPda);

    const destinationAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    assert.isNotNull(treasuryAfter);
    assert.isNotNull(vaultAfter);

    assert.isTrue(
      Buffer.from(treasuryAfter!.data).equals(
        Buffer.from(treasuryBefore!.data),
      ),
      "fake-mint attack mutated treasury state",
    );

    assert.isTrue(
      Buffer.from(vaultAfter!.data).equals(
        Buffer.from(vaultBefore!.data),
      ),
      "fake-mint attack mutated the settlement vault",
    );

    assert.equal(
      destinationAfter.amount,
      destinationBefore.amount,
      "fake-mint attack transferred reserve funds",
    );
  });

  it("RT-002B rejects a fake treasury vault without mutation", async () => {
    const attacker = Keypair.generate();

    const fakeVault = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        attacker.publicKey,
      )
    ).address;

    const treasuryBefore =
      await provider.connection.getAccountInfo(treasuryStatePda);

    const legitimateVaultBefore =
      await provider.connection.getAccountInfo(settlementVaultPda);

    const fakeVaultBefore = await getAccount(
      provider.connection,
      fakeVault,
    );

    const destinationBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    assert.isNotNull(treasuryBefore);
    assert.isNotNull(legitimateVaultBefore);

    let rejected = false;

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
          settlementVault: fakeVault,
          reserveDestination,
          buybackDestination,
          liquidityDestination,
          companyDestination,
          founderDestination,
          executionConfig: executionConfigPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
    } catch {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "reserve execution must reject an attacker-controlled vault",
    );

    const treasuryAfter =
      await provider.connection.getAccountInfo(treasuryStatePda);

    const legitimateVaultAfter =
      await provider.connection.getAccountInfo(settlementVaultPda);

    const fakeVaultAfter = await getAccount(
      provider.connection,
      fakeVault,
    );

    const destinationAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    assert.isNotNull(treasuryAfter);
    assert.isNotNull(legitimateVaultAfter);

    assert.isTrue(
      Buffer.from(treasuryAfter!.data).equals(
        Buffer.from(treasuryBefore!.data),
      ),
      "fake-vault attack mutated treasury state",
    );

    assert.isTrue(
      Buffer.from(legitimateVaultAfter!.data).equals(
        Buffer.from(legitimateVaultBefore!.data),
      ),
      "fake-vault attack mutated the legitimate vault",
    );

    assert.equal(
      fakeVaultAfter.amount,
      fakeVaultBefore.amount,
      "fake-vault attack transferred funds into the attacker vault",
    );

    assert.equal(
      destinationAfter.amount,
      destinationBefore.amount,
      "fake-vault attack changed the reserve destination balance",
    );
  });

  it("RT-002B rejects an attacker-controlled reserve destination without mutation", async () => {
    const attacker = Keypair.generate();

    const attackerDestination = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        attacker.publicKey,
      )
    ).address;

    const treasuryBefore =
      await provider.connection.getAccountInfo(treasuryStatePda);

    const vaultBefore =
      await provider.connection.getAccountInfo(settlementVaultPda);

    const legitimateDestinationBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const attackerDestinationBefore = await getAccount(
      provider.connection,
      attackerDestination,
    );

    assert.isNotNull(treasuryBefore);
    assert.isNotNull(vaultBefore);

    let rejected = false;

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
          reserveDestination: attackerDestination,
          buybackDestination,
          liquidityDestination,
          companyDestination,
          founderDestination,
          executionConfig: executionConfigPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
    } catch {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "reserve execution must reject a substituted destination",
    );

    const treasuryAfter =
      await provider.connection.getAccountInfo(treasuryStatePda);

    const vaultAfter =
      await provider.connection.getAccountInfo(settlementVaultPda);

    const legitimateDestinationAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const attackerDestinationAfter = await getAccount(
      provider.connection,
      attackerDestination,
    );

    assert.isNotNull(treasuryAfter);
    assert.isNotNull(vaultAfter);

    assert.isTrue(
      Buffer.from(treasuryAfter!.data).equals(
        Buffer.from(treasuryBefore!.data),
      ),
      "destination-substitution attack mutated treasury state",
    );

    assert.isTrue(
      Buffer.from(vaultAfter!.data).equals(
        Buffer.from(vaultBefore!.data),
      ),
      "destination-substitution attack mutated the settlement vault",
    );

    assert.equal(
      legitimateDestinationAfter.amount,
      legitimateDestinationBefore.amount,
      "destination-substitution attack changed the legitimate destination",
    );

    assert.equal(
      attackerDestinationAfter.amount,
      attackerDestinationBefore.amount,
      "destination-substitution attack transferred funds to the attacker",
    );
  });

  it("RT-002B rejects a substituted token program without mutation", async () => {
    const treasuryBefore =
      await provider.connection.getAccountInfo(treasuryStatePda);

    const vaultBefore =
      await provider.connection.getAccountInfo(settlementVaultPda);

    const destinationBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    assert.isNotNull(treasuryBefore);
    assert.isNotNull(vaultBefore);

    let rejected = false;

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
          executionConfig: executionConfigPda,
          tokenProgram: SystemProgram.programId,
        })
        .rpc();
    } catch {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "reserve execution must reject a substituted token program",
    );

    const treasuryAfter =
      await provider.connection.getAccountInfo(treasuryStatePda);

    const vaultAfter =
      await provider.connection.getAccountInfo(settlementVaultPda);

    const destinationAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    assert.isNotNull(treasuryAfter);
    assert.isNotNull(vaultAfter);

    assert.isTrue(
      Buffer.from(treasuryAfter!.data).equals(
        Buffer.from(treasuryBefore!.data),
      ),
      "token-program substitution mutated treasury state",
    );

    assert.isTrue(
      Buffer.from(vaultAfter!.data).equals(
        Buffer.from(vaultBefore!.data),
      ),
      "token-program substitution mutated the settlement vault",
    );

    assert.equal(
      destinationAfter.amount,
      destinationBefore.amount,
      "token-program substitution transferred reserve funds",
    );
  });
'''

try:
    updated = text[:line_start] + block.rstrip() + "\n\n" + text[line_start:]
    TEST_FILE.write_text(updated)

    verification = TEST_FILE.read_text()

    required = [
        MARKER,
        "RT-002B rejects a fake settlement mint without mutation",
        "RT-002B rejects a fake treasury vault without mutation",
        "RT-002B rejects an attacker-controlled reserve destination without mutation",
        "RT-002B rejects a substituted token program without mutation",
    ]

    for fragment in required:
        if fragment not in verification:
            shutil.copy2(backup_file, TEST_FILE)
            raise RuntimeError(
                f"Patch verification failed; backup restored. Missing: {fragment}"
            )

except Exception:
    shutil.copy2(backup_file, TEST_FILE)
    raise

print("RT-002B account-substitution attacks inserted.")
print("Attack tests added: 4")
print("Expected integration test total: 27")
