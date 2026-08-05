from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import shutil

ROOT = Path.cwd()
TESTS = ROOT / "tests/rbvr_protocol.ts"

MARKER = "RT-005 — hostile Spillway assault"
SUCCESS_TEST = (
    '  it("deploys only Reserve surplus through the permissionless Spillway"'
)

if not TESTS.exists():
    raise RuntimeError(f"Missing integration test file: {TESTS}")

text = TESTS.read_text()

if MARKER in text:
    raise RuntimeError("RT-005 already appears to be installed.")

if SUCCESS_TEST not in text:
    raise RuntimeError(
        "Could not locate the successful Spillway integration test."
    )

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup = ROOT / f".rt005-backup-{timestamp}"
backup_file = backup / TESTS.relative_to(ROOT)

backup_file.parent.mkdir(parents=True, exist_ok=False)
shutil.copy2(TESTS, backup_file)

print(f"Backup created: {backup}")

# Make the cooldown deterministic for all following RT-005 tests.
old_cooldown = """        new anchor.BN(1),
      )
"""

new_cooldown = """        new anchor.BN(86_400),
      )
"""

cooldown_count = text.count(old_cooldown)

if cooldown_count != 1:
    raise RuntimeError(
        "Expected exactly one one-second Reserve Policy cooldown, "
        f"found {cooldown_count}"
    )

text = text.replace(old_cooldown, new_cooldown, 1)

# Insert hostile tests after the successful Spillway test but before
# the final describe-block closure.
close_marker = "\n});"
insertion_point = text.rfind(close_marker)

if insertion_point == -1:
    raise RuntimeError("Could not locate the integration describe closure.")

attack_tests = r'''
  // RT-005 — hostile Spillway assault
  it("RT-005A rejects a substituted Reserve Policy without mutation", async () => {
    const [reservePolicyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("reserve-policy"), treasuryStatePda.toBuffer()],
      PROGRAM_ID,
    );

    const reserveBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const liquidityBefore = await getAccount(
      provider.connection,
      liquidityDestination,
    );

    const policyBefore =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .spillwayRelease()
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,
          executionConfig: executionConfigPda,

          // Hostile substitution:
          // an unrelated program PDA is supplied where the canonical
          // ReservePolicy PDA is required.
          reservePolicy: executionConfigPda,

          settlementMint,
          reserveVault: reserveDestination,
          liquidityDestination,
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

    assert.isTrue(
      rejected,
      "A substituted Reserve Policy must be rejected",
    );

    assert.match(
      failureText,
      /ConstraintSeeds|AccountDiscriminatorMismatch|ReservePolicy|InvalidReservePolicyLinkage|account discriminator/i,
      `Unexpected Reserve Policy substitution rejection: ${failureText}`,
    );

    const reserveAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const liquidityAfter = await getAccount(
      provider.connection,
      liquidityDestination,
    );

    const policyAfter =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    assert.equal(
      reserveAfter.amount.toString(),
      reserveBefore.amount.toString(),
      "Rejected policy substitution changed the Reserve Vault",
    );

    assert.equal(
      liquidityAfter.amount.toString(),
      liquidityBefore.amount.toString(),
      "Rejected policy substitution changed Liquidity Growth",
    );

    assert.equal(
      policyAfter.lastDeployedAt.toString(),
      policyBefore.lastDeployedAt.toString(),
      "Rejected policy substitution changed the deployment timestamp",
    );

    assert.equal(
      policyAfter.lifetimeDeployed.toString(),
      policyBefore.lifetimeDeployed.toString(),
      "Rejected policy substitution changed lifetime deployment accounting",
    );
  });

  it("RT-005B rejects a substituted Reserve Vault without mutation", async () => {
    const [reservePolicyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("reserve-policy"), treasuryStatePda.toBuffer()],
      PROGRAM_ID,
    );

    const reserveBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const settlementBefore = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    const liquidityBefore = await getAccount(
      provider.connection,
      liquidityDestination,
    );

    const policyBefore =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .spillwayRelease()
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,
          executionConfig: executionConfigPda,
          reservePolicy: reservePolicyPda,
          settlementMint,

          // Hostile substitution:
          // the settlement vault is supplied in place of the canonical
          // Reserve Vault PDA.
          reserveVault: settlementVaultPda,

          liquidityDestination,
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

    assert.isTrue(
      rejected,
      "A substituted Reserve Vault must be rejected",
    );

    assert.match(
      failureText,
      /ConstraintSeeds|InvalidSettlementVault|reserve vault|seeds constraint/i,
      `Unexpected Reserve Vault substitution rejection: ${failureText}`,
    );

    const reserveAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const settlementAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    const liquidityAfter = await getAccount(
      provider.connection,
      liquidityDestination,
    );

    const policyAfter =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    assert.equal(
      reserveAfter.amount.toString(),
      reserveBefore.amount.toString(),
      "Rejected vault substitution changed the canonical Reserve Vault",
    );

    assert.equal(
      settlementAfter.amount.toString(),
      settlementBefore.amount.toString(),
      "Rejected vault substitution changed the settlement vault",
    );

    assert.equal(
      liquidityAfter.amount.toString(),
      liquidityBefore.amount.toString(),
      "Rejected vault substitution changed Liquidity Growth",
    );

    assert.equal(
      policyAfter.lastDeployedAt.toString(),
      policyBefore.lastDeployedAt.toString(),
      "Rejected vault substitution changed the deployment timestamp",
    );

    assert.equal(
      policyAfter.lifetimeDeployed.toString(),
      policyBefore.lifetimeDeployed.toString(),
      "Rejected vault substitution changed lifetime deployment accounting",
    );
  });

  it("RT-005C rejects a substituted Spillway destination without mutation", async () => {
    const [reservePolicyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("reserve-policy"), treasuryStatePda.toBuffer()],
      PROGRAM_ID,
    );

    const reserveBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const liquidityBefore = await getAccount(
      provider.connection,
      liquidityDestination,
    );

    const buybackBefore = await getAccount(
      provider.connection,
      buybackDestination,
    );

    const policyBefore =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .spillwayRelease()
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,
          executionConfig: executionConfigPda,
          reservePolicy: reservePolicyPda,
          settlementMint,
          reserveVault: reserveDestination,

          // Hostile substitution:
          // a valid settlement-token account is supplied, but it is not
          // the immutable Liquidity Growth destination.
          liquidityDestination: buybackDestination,

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

    assert.isTrue(
      rejected,
      "A substituted Spillway destination must be rejected",
    );

    assert.match(
      failureText,
      /InvalidSpillwayDestination|constraint was violated|liquidity destination/i,
      `Unexpected destination substitution rejection: ${failureText}`,
    );

    const reserveAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const liquidityAfter = await getAccount(
      provider.connection,
      liquidityDestination,
    );

    const buybackAfter = await getAccount(
      provider.connection,
      buybackDestination,
    );

    const policyAfter =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    assert.equal(
      reserveAfter.amount.toString(),
      reserveBefore.amount.toString(),
      "Rejected destination substitution changed the Reserve Vault",
    );

    assert.equal(
      liquidityAfter.amount.toString(),
      liquidityBefore.amount.toString(),
      "Rejected destination substitution changed Liquidity Growth",
    );

    assert.equal(
      buybackAfter.amount.toString(),
      buybackBefore.amount.toString(),
      "Rejected destination substitution transferred funds to Buyback",
    );

    assert.equal(
      policyAfter.lastDeployedAt.toString(),
      policyBefore.lastDeployedAt.toString(),
      "Rejected destination substitution changed the deployment timestamp",
    );

    assert.equal(
      policyAfter.lifetimeDeployed.toString(),
      policyBefore.lifetimeDeployed.toString(),
      "Rejected destination substitution changed lifetime deployment accounting",
    );
  });

  it("RT-005D rejects an immediate Spillway replay without mutation", async () => {
    const [reservePolicyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("reserve-policy"), treasuryStatePda.toBuffer()],
      PROGRAM_ID,
    );

    const reserveBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const liquidityBefore = await getAccount(
      provider.connection,
      liquidityDestination,
    );

    const policyBefore =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    assert.isAbove(
      Number(policyBefore.lastDeployedAt),
      0,
      "The successful Spillway test must execute before replay testing",
    );

    let rejected = false;
    let failureText = "";

    try {
      await program.methods
        .spillwayRelease()
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,
          executionConfig: executionConfigPda,
          reservePolicy: reservePolicyPda,
          settlementMint,
          reserveVault: reserveDestination,
          liquidityDestination,
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

    assert.isTrue(
      rejected,
      "An immediate Spillway replay must be rejected",
    );

    assert.match(
      failureText,
      /ReserveDeploymentCooldownActive|cooldown is still active|cooldown/i,
      `Unexpected Spillway replay rejection: ${failureText}`,
    );

    const reserveAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const liquidityAfter = await getAccount(
      provider.connection,
      liquidityDestination,
    );

    const policyAfter =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    assert.equal(
      reserveAfter.amount.toString(),
      reserveBefore.amount.toString(),
      "Rejected replay changed the Reserve Vault",
    );

    assert.equal(
      liquidityAfter.amount.toString(),
      liquidityBefore.amount.toString(),
      "Rejected replay changed Liquidity Growth",
    );

    assert.equal(
      policyAfter.lastDeployedAt.toString(),
      policyBefore.lastDeployedAt.toString(),
      "Rejected replay changed the deployment timestamp",
    );

    assert.equal(
      policyAfter.lifetimeDeployed.toString(),
      policyBefore.lifetimeDeployed.toString(),
      "Rejected replay changed lifetime deployment accounting",
    );
  });
'''

updated = (
    text[:insertion_point]
    + "\n"
    + attack_tests.rstrip()
    + text[insertion_point:]
)

TESTS.write_text(updated)

verification = TESTS.read_text()

required = [
    MARKER,
    "RT-005A rejects a substituted Reserve Policy without mutation",
    "RT-005B rejects a substituted Reserve Vault without mutation",
    "RT-005C rejects a substituted Spillway destination without mutation",
    "RT-005D rejects an immediate Spillway replay without mutation",
    "new anchor.BN(86_400)",
]

for fragment in required:
    if fragment not in verification:
        shutil.copy2(backup_file, TESTS)
        raise RuntimeError(
            f"RT-005 verification failed. Missing: {fragment}. "
            "Original test file restored."
        )

print("RT-005 Spillway assault tests inserted.")
print("New hostile integration tests: 4")
print("Expected integration total: 42")
