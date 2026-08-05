from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import shutil

ROOT = Path.cwd()
TEST_FILE = ROOT / "tests/rbvr_protocol.ts"

MARKER = "RT-002A — initialization assault"

if not TEST_FILE.exists():
    raise RuntimeError(f"Missing integration test file: {TEST_FILE}")

text = TEST_FILE.read_text()

if MARKER in text:
    raise RuntimeError("RT-002A tests already exist.")

required_markers = [
    'it("initializes the locked protocol allocation"',
    'it("initializes the settlement treasury and vault"',
    'it("initializes founder compensation controls"',
    'it("initializes company allocation controls"',
    'it("initializes the execution config"',
    'it("deposits settlement assets into the treasury vault"',
    'it("processes fees using the locked 30/20/20/20/10 model"',
]

for marker in required_markers:
    if marker not in text:
        raise RuntimeError(f"Required insertion marker not found: {marker}")

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup_root = ROOT / f".rt002a-backup-{timestamp}"
backup_file = backup_root / TEST_FILE.relative_to(ROOT)

backup_file.parent.mkdir(parents=True, exist_ok=False)
shutil.copy2(TEST_FILE, backup_file)

print(f"Backup created: {backup_root}")

def insert_before(source: str, marker: str, block: str) -> str:
    position = source.find(marker)

    if position == -1:
        raise RuntimeError(f"Could not locate marker: {marker}")

    line_start = source.rfind("\n", 0, position) + 1
    return source[:line_start] + block.rstrip() + "\n\n" + source[line_start:]

unauthorized_protocol_config = r'''
  // RT-002A — initialization assault
  it("RT-002A rejects protocol-config initialization by an unauthorized signer", async () => {
    const attacker = Keypair.generate();

    const airdropSignature = await provider.connection.requestAirdrop(
      attacker.publicKey,
      1_000_000_000,
    );

    await provider.connection.confirmTransaction(
      airdropSignature,
      "confirmed",
    );

    const protocolBefore =
      await provider.connection.getAccountInfo(protocolStatePda);

    assert.isNotNull(protocolBefore);

    const configBefore =
      await provider.connection.getAccountInfo(protocolConfigPda);

    assert.isNull(
      configBefore,
      "protocol config must not exist before the legitimate initialization",
    );

    let rejected = false;

    try {
      await program.methods
        .initializeProtocolConfig()
        .accountsPartial({
          protocolState: protocolStatePda,
          protocolConfig: protocolConfigPda,
          authority: attacker.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([attacker])
        .rpc();
    } catch {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "an unauthorized signer must not initialize protocol configuration",
    );

    const protocolAfter =
      await provider.connection.getAccountInfo(protocolStatePda);

    const configAfter =
      await provider.connection.getAccountInfo(protocolConfigPda);

    assert.isNotNull(protocolAfter);
    assert.isNull(
      configAfter,
      "rejected unauthorized initialization must not create protocol config",
    );

    assert.isTrue(
      Buffer.from(protocolAfter!.data).equals(
        Buffer.from(protocolBefore!.data),
      ),
      "rejected unauthorized initialization mutated protocol state",
    );
  });
'''

duplicate_protocol_config = r'''
  it("RT-002A rejects repeated protocol-config initialization without mutation", async () => {
    const protocolBefore =
      await provider.connection.getAccountInfo(protocolStatePda);

    const configBefore =
      await provider.connection.getAccountInfo(protocolConfigPda);

    assert.isNotNull(protocolBefore);
    assert.isNotNull(configBefore);

    let rejected = false;

    try {
      await program.methods
        .initializeProtocolConfig()
        .accountsPartial({
          protocolState: protocolStatePda,
          protocolConfig: protocolConfigPda,
          authority,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "canonical protocol configuration must not initialize twice",
    );

    const protocolAfter =
      await provider.connection.getAccountInfo(protocolStatePda);

    const configAfter =
      await provider.connection.getAccountInfo(protocolConfigPda);

    assert.isNotNull(protocolAfter);
    assert.isNotNull(configAfter);

    assert.isTrue(
      Buffer.from(protocolAfter!.data).equals(
        Buffer.from(protocolBefore!.data),
      ),
      "failed protocol-config reinitialization mutated protocol state",
    );

    assert.isTrue(
      Buffer.from(configAfter!.data).equals(
        Buffer.from(configBefore!.data),
      ),
      "failed protocol-config reinitialization mutated configuration",
    );
  });
'''

duplicate_treasury = r'''
  it("RT-002A rejects repeated treasury initialization without mutation", async () => {
    const protocolBefore =
      await provider.connection.getAccountInfo(protocolStatePda);

    const treasuryBefore =
      await provider.connection.getAccountInfo(treasuryStatePda);

    const vaultBefore =
      await provider.connection.getAccountInfo(settlementVaultPda);

    assert.isNotNull(protocolBefore);
    assert.isNotNull(treasuryBefore);
    assert.isNotNull(vaultBefore);

    let rejected = false;

    try {
      await program.methods
        .initializeTreasury()
        .accountsPartial({
          protocolState: protocolStatePda,
          treasuryState: treasuryStatePda,
          settlementMint,
          settlementVault: settlementVaultPda,
          authority,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "canonical treasury and vault must not initialize twice",
    );

    const protocolAfter =
      await provider.connection.getAccountInfo(protocolStatePda);

    const treasuryAfter =
      await provider.connection.getAccountInfo(treasuryStatePda);

    const vaultAfter =
      await provider.connection.getAccountInfo(settlementVaultPda);

    assert.isNotNull(protocolAfter);
    assert.isNotNull(treasuryAfter);
    assert.isNotNull(vaultAfter);

    assert.isTrue(
      Buffer.from(protocolAfter!.data).equals(
        Buffer.from(protocolBefore!.data),
      ),
      "failed treasury reinitialization mutated protocol state",
    );

    assert.isTrue(
      Buffer.from(treasuryAfter!.data).equals(
        Buffer.from(treasuryBefore!.data),
      ),
      "failed treasury reinitialization mutated treasury state",
    );

    assert.isTrue(
      Buffer.from(vaultAfter!.data).equals(
        Buffer.from(vaultBefore!.data),
      ),
      "failed treasury reinitialization mutated settlement vault",
    );
  });
'''

duplicate_founder = r'''
  it("RT-002A rejects repeated founder initialization without mutation", async () => {
    const protocolBefore =
      await provider.connection.getAccountInfo(protocolStatePda);

    const founderBefore =
      await provider.connection.getAccountInfo(founderStatePda);

    assert.isNotNull(protocolBefore);
    assert.isNotNull(founderBefore);

    let rejected = false;

    try {
      await program.methods
        .initializeFounder(
          founderRecipientOwner.publicKey,
          FOUNDER_CAP,
          PERIOD_DURATION,
        )
        .accountsPartial({
          protocolState: protocolStatePda,
          founderState: founderStatePda,
          authority,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "canonical founder state must not initialize twice",
    );

    const protocolAfter =
      await provider.connection.getAccountInfo(protocolStatePda);

    const founderAfter =
      await provider.connection.getAccountInfo(founderStatePda);

    assert.isNotNull(protocolAfter);
    assert.isNotNull(founderAfter);

    assert.isTrue(
      Buffer.from(protocolAfter!.data).equals(
        Buffer.from(protocolBefore!.data),
      ),
      "failed founder reinitialization mutated protocol state",
    );

    assert.isTrue(
      Buffer.from(founderAfter!.data).equals(
        Buffer.from(founderBefore!.data),
      ),
      "failed founder reinitialization mutated founder state",
    );
  });
'''

duplicate_company = r'''
  it("RT-002A rejects repeated company initialization without mutation", async () => {
    const protocolBefore =
      await provider.connection.getAccountInfo(protocolStatePda);

    const companyBefore =
      await provider.connection.getAccountInfo(companyStatePda);

    assert.isNotNull(protocolBefore);
    assert.isNotNull(companyBefore);

    let rejected = false;

    try {
      await program.methods
        .initializeCompany(
          companyRecipientOwner.publicKey,
          COMPANY_CAP,
          PERIOD_DURATION,
        )
        .accountsPartial({
          protocolState: protocolStatePda,
          companyState: companyStatePda,
          authority,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "canonical company state must not initialize twice",
    );

    const protocolAfter =
      await provider.connection.getAccountInfo(protocolStatePda);

    const companyAfter =
      await provider.connection.getAccountInfo(companyStatePda);

    assert.isNotNull(protocolAfter);
    assert.isNotNull(companyAfter);

    assert.isTrue(
      Buffer.from(protocolAfter!.data).equals(
        Buffer.from(protocolBefore!.data),
      ),
      "failed company reinitialization mutated protocol state",
    );

    assert.isTrue(
      Buffer.from(companyAfter!.data).equals(
        Buffer.from(companyBefore!.data),
      ),
      "failed company reinitialization mutated company state",
    );
  });
'''

duplicate_execution_config = r'''
  it("RT-002A rejects repeated execution-config initialization without mutation", async () => {
    const protocolBefore =
      await provider.connection.getAccountInfo(protocolStatePda);

    const executionBefore =
      await provider.connection.getAccountInfo(executionConfigPda);

    assert.isNotNull(protocolBefore);
    assert.isNotNull(executionBefore);

    let rejected = false;

    try {
      await program.methods
        .initializeExecutionConfig()
        .accountsPartial({
          protocolState: protocolStatePda,
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
          authority,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "canonical execution configuration must not initialize twice",
    );

    const protocolAfter =
      await provider.connection.getAccountInfo(protocolStatePda);

    const executionAfter =
      await provider.connection.getAccountInfo(executionConfigPda);

    assert.isNotNull(protocolAfter);
    assert.isNotNull(executionAfter);

    assert.isTrue(
      Buffer.from(protocolAfter!.data).equals(
        Buffer.from(protocolBefore!.data),
      ),
      "failed execution-config reinitialization mutated protocol state",
    );

    assert.isTrue(
      Buffer.from(executionAfter!.data).equals(
        Buffer.from(executionBefore!.data),
      ),
      "failed execution-config reinitialization mutated execution config",
    );
  });
'''

try:
    # Before legitimate protocol config initialization.
    text = insert_before(
        text,
        'it("initializes the locked protocol allocation"',
        unauthorized_protocol_config,
    )

    # After legitimate protocol config, before treasury initialization.
    text = insert_before(
        text,
        'it("initializes the settlement treasury and vault"',
        duplicate_protocol_config,
    )

    # After legitimate treasury, before founder initialization.
    text = insert_before(
        text,
        'it("initializes founder compensation controls"',
        duplicate_treasury,
    )

    # After legitimate founder, before company initialization.
    text = insert_before(
        text,
        'it("initializes company allocation controls"',
        duplicate_founder,
    )

    # After legitimate company, before deposit.
    text = insert_before(
        text,
        'it("deposits settlement assets into the treasury vault"',
        duplicate_company,
    )

    # After legitimate execution config, before process fees.
    text = insert_before(
        text,
        'it("processes fees using the locked 30/20/20/20/10 model"',
        duplicate_execution_config,
    )

    TEST_FILE.write_text(text)

    verification = TEST_FILE.read_text()

    required = [
        MARKER,
        "RT-002A rejects protocol-config initialization by an unauthorized signer",
        "RT-002A rejects repeated protocol-config initialization without mutation",
        "RT-002A rejects repeated treasury initialization without mutation",
        "RT-002A rejects repeated founder initialization without mutation",
        "RT-002A rejects repeated company initialization without mutation",
        "RT-002A rejects repeated execution-config initialization without mutation",
    ]

    for fragment in required:
        if fragment not in verification:
            shutil.copy2(backup_file, TEST_FILE)
            raise RuntimeError(
                f"Verification failed; test file restored. Missing: {fragment}"
            )

except Exception:
    shutil.copy2(backup_file, TEST_FILE)
    raise

print("RT-002A malicious initialization tests inserted.")
print("Attack tests added: 6")
print("Expected integration test total: 23")
