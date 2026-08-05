from pathlib import Path

path = Path("tests/rbvr_protocol.ts")
text = path.read_text()

# Remove caller-selected release amounts everywhere.
for method in (
    "authorizeReserveExecution",
    "authorizeLiquidityExecution",
    "authorizeCompanyExecution",
    "authorizeFounderExecution",
    "authorizeBuybackExecution",
):
    text = text.replace(
        f".{method}(EXECUTION_AMOUNT)",
        f".{method}()",
    )

start_marker = '  it("authorizes and transfers the reserve allocation"'
end_marker = '  it("preserves final global accounting integrity"'

start = text.find(start_marker)
if start == -1:
    raise RuntimeError("Could not find the reserve execution test")

final_start = text.find(end_marker, start)
if final_start == -1:
    raise RuntimeError("Could not find the final accounting test")

# Find the end of the final test immediately before the describe block closes.
end = text.find("\n  });\n});", final_start)
if end == -1:
    raise RuntimeError("Could not find the end of the execution test block")

end += len("\n  });")

replacement = '''  it("autonomously transfers the reserve allocation", async () => {
    const destinationBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );
    const treasuryBefore = await treasury();

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
        authority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const destinationAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );
    const treasuryAfter = await treasury();

    const released = destinationAfter.amount - destinationBefore.amount;

    assert(released > 0n, "reserve release must be positive");

    assert.equal(
      treasuryBefore.pendingReserve
        .sub(treasuryAfter.pendingReserve)
        .toString(),
      released.toString(),
    );

    assert.equal(
      treasuryAfter.releasedReserve
        .sub(treasuryBefore.releasedReserve)
        .toString(),
      released.toString(),
    );

    await assertAllTreasuryInvariants();
  });

  it("autonomously transfers the liquidity allocation", async () => {
    const destinationBefore = await getAccount(
      provider.connection,
      liquidityDestination,
    );
    const treasuryBefore = await treasury();

    await program.methods
      .authorizeLiquidityExecution()
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
        authority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const destinationAfter = await getAccount(
      provider.connection,
      liquidityDestination,
    );
    const treasuryAfter = await treasury();

    const released = destinationAfter.amount - destinationBefore.amount;

    assert(released > 0n, "liquidity release must be positive");

    assert.equal(
      treasuryBefore.pendingLiquidity
        .sub(treasuryAfter.pendingLiquidity)
        .toString(),
      released.toString(),
    );

    assert.equal(
      treasuryAfter.releasedLiquidity
        .sub(treasuryBefore.releasedLiquidity)
        .toString(),
      released.toString(),
    );

    await assertAllTreasuryInvariants();
  });

  it("autonomously transfers the company allocation", async () => {
    const destinationBefore = await getAccount(
      provider.connection,
      companyDestination,
    );
    const treasuryBefore = await treasury();
    const companyBefore =
      await program.account.companyState.fetch(companyStatePda);

    await program.methods
      .authorizeCompanyExecution()
      .accountsPartial({
        protocolState: protocolStatePda,
        companyState: companyStatePda,
        treasury: treasuryStatePda,
        executionConfig: executionConfigPda,
        settlementMint,
        settlementVault: settlementVaultPda,
        companyDestination,
        authority: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const destinationAfter = await getAccount(
      provider.connection,
      companyDestination,
    );
    const treasuryAfter = await treasury();
    const companyAfter =
      await program.account.companyState.fetch(companyStatePda);

    const released = destinationAfter.amount - destinationBefore.amount;

    assert(released > 0n, "company release must be positive");

    assert.equal(
      treasuryBefore.pendingCompany
        .sub(treasuryAfter.pendingCompany)
        .toString(),
      released.toString(),
    );

    assert.equal(
      treasuryAfter.releasedCompany
        .sub(treasuryBefore.releasedCompany)
        .toString(),
      released.toString(),
    );

    assert.equal(
      companyAfter.spentCurrentPeriod.toString(),
      companyBefore.spentCurrentPeriod.toString(),
      "release must not change company period-cap accounting",
    );

    assert.equal(
      companyAfter.lifetimeSpent.toString(),
      companyBefore.lifetimeSpent.toString(),
      "release must not change company lifetime accounting",
    );

    await assertAllTreasuryInvariants();
  });

  it("autonomously transfers founder compensation", async () => {
    const destinationBefore = await getAccount(
      provider.connection,
      founderDestination,
    );
    const treasuryBefore = await treasury();
    const founderBefore =
      await program.account.founderState.fetch(founderStatePda);

    await program.methods
      .authorizeFounderExecution()
      .accountsPartial({
        protocolState: protocolStatePda,
        founderState: founderStatePda,
        treasury: treasuryStatePda,
        executionConfig: executionConfigPda,
        settlementMint,
        settlementVault: settlementVaultPda,
        founderDestination,
        authority: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const destinationAfter = await getAccount(
      provider.connection,
      founderDestination,
    );
    const treasuryAfter = await treasury();
    const founderAfter =
      await program.account.founderState.fetch(founderStatePda);

    const released = destinationAfter.amount - destinationBefore.amount;

    assert(released > 0n, "founder release must be positive");

    assert.equal(
      treasuryBefore.pendingFounder
        .sub(treasuryAfter.pendingFounder)
        .toString(),
      released.toString(),
    );

    assert.equal(
      treasuryAfter.releasedFounder
        .sub(treasuryBefore.releasedFounder)
        .toString(),
      released.toString(),
    );

    assert.equal(
      founderAfter.earnedCurrentPeriod.toString(),
      founderBefore.earnedCurrentPeriod.toString(),
      "release must not change founder period-cap accounting",
    );

    assert.equal(
      founderAfter.lifetimeEarned.toString(),
      founderBefore.lifetimeEarned.toString(),
      "release must not change founder lifetime accounting",
    );

    await assertAllTreasuryInvariants();
  });

  it("autonomously transfers the buyback allocation", async () => {
    const destinationBefore = await getAccount(
      provider.connection,
      buybackDestination,
    );
    const treasuryBefore = await treasury();

    await program.methods
      .authorizeBuybackExecution()
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
        authority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const destinationAfter = await getAccount(
      provider.connection,
      buybackDestination,
    );
    const treasuryAfter = await treasury();

    const released = destinationAfter.amount - destinationBefore.amount;

    assert(released > 0n, "buyback release must be positive");

    assert.equal(
      treasuryBefore.pendingBuybackBurn
        .sub(treasuryAfter.pendingBuybackBurn)
        .toString(),
      released.toString(),
    );

    assert.equal(
      treasuryAfter.releasedBuybackBurn
        .sub(treasuryBefore.releasedBuybackBurn)
        .toString(),
      released.toString(),
    );

    await assertAllTreasuryInvariants();
  });

  it("preserves final global accounting integrity", async () => {
    const state = await treasury();

    const totalPending = state.pendingReserve
      .add(state.pendingBuybackBurn)
      .add(state.pendingLiquidity)
      .add(state.pendingCompany)
      .add(state.pendingFounder);

    const totalReleased = state.releasedReserve
      .add(state.releasedBuybackBurn)
      .add(state.releasedLiquidity)
      .add(state.releasedCompany)
      .add(state.releasedFounder);

    assertBn(
      totalPending.add(totalReleased),
      state.totalFeesAllocated,
    );

    const vault = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    assert.equal(
      vault.amount.toString(),
      totalPending.toString(),
      "vault balance must equal total pending accounting",
    );

    assert(
      totalReleased.gt(new anchor.BN(0)),
      "autonomous execution must release settlement assets",
    );

    assert(
      number(state.processingEpoch) >= 1,
      "processing epoch must advance",
    );

    await assertAllTreasuryInvariants();
  });'''

path.write_text(text[:start] + replacement + text[end:])

print("Autonomous integration tests patched successfully.")
