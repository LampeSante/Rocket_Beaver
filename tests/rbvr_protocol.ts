import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import {
  createMint,
  getAccount,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import type { TreasuryRouter } from "../target/types/treasury_router";

import { assert } from "chai";
describe("RBVR Treasury Router — protocol integration", function () {
  this.timeout(1_000_000);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TreasuryRouter as Program<TreasuryRouter>;

  const authority = provider.wallet.publicKey;
  const payer = (provider.wallet as anchor.Wallet).payer;

  const PROGRAM_ID = new PublicKey(
    "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3",
  );

  const DEPOSIT_AMOUNT = new anchor.BN(1_000_000);
  const PROCESS_AMOUNT = new anchor.BN(1_000_000);
  const EXECUTION_AMOUNT = new anchor.BN(10_000);

  const FOUNDER_CAP = new anchor.BN(1_000_000_000);
  const COMPANY_CAP = new anchor.BN(1_000_000_000);
  const PERIOD_DURATION = new anchor.BN(30 * 24 * 60 * 60);

  let settlementMint: PublicKey;
  let sourceTokenAccount: PublicKey;

  let protocolStatePda: PublicKey;
  let protocolConfigPda: PublicKey;
  let treasuryStatePda: PublicKey;
  let settlementVaultPda: PublicKey;
  let founderStatePda: PublicKey;
  let companyStatePda: PublicKey;
  let executionConfigPda: PublicKey;

  let founderDestination: PublicKey;
  let companyDestination: PublicKey;
  let reserveDestination: PublicKey;
  let liquidityDestination: PublicKey;
  let buybackDestination: PublicKey;

  const founderRecipientOwner = Keypair.generate();
  const companyRecipientOwner = Keypair.generate();
  const reserveRecipientOwner = Keypair.generate();
  const liquidityRecipientOwner = Keypair.generate();
  const buybackRecipientOwner = Keypair.generate();

  function number(value: anchor.BN): number {
    return Number(value.toString());
  }

  function assertBn(
    actual: anchor.BN,
    expected: anchor.BN | number,
    message: string = "",
  ): void {
    const expectedBn =
      expected instanceof anchor.BN ? expected : new anchor.BN(expected);

    assert.equal(actual.toString(), expectedBn.toString(), message);
  }

  function derivePdas(): void {
    [protocolStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("protocol")],
      PROGRAM_ID,
    );

    [protocolConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("protocol-config"), protocolStatePda.toBuffer()],
      PROGRAM_ID,
    );

    [treasuryStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury"), protocolStatePda.toBuffer()],
      PROGRAM_ID,
    );

    [settlementVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury-vault"), treasuryStatePda.toBuffer()],
      PROGRAM_ID,
    );

    [founderStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("founder-state"), protocolStatePda.toBuffer()],
      PROGRAM_ID,
    );

    [companyStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("company-state"), protocolStatePda.toBuffer()],
      PROGRAM_ID,
    );

    [executionConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("execution-config"), protocolStatePda.toBuffer()],
      PROGRAM_ID,
    );
  }

  async function treasury() {
    return program.account.treasuryState.fetch(treasuryStatePda);
  }

  function assertBucketInvariant(
    pending: anchor.BN,
    released: anchor.BN,
    lifetime: anchor.BN,
    bucket: string,
  ): void {
    assertBn(
      pending.add(released),
      lifetime,
      `${bucket}: pending + released must equal lifetime`,
    );
  }

  async function assertAllTreasuryInvariants(): Promise<void> {
    const state = await treasury();

    assertBucketInvariant(
      state.pendingReserve,
      state.releasedReserve,
      state.lifetimeReserve,
      "reserve",
    );

    assertBucketInvariant(
      state.pendingBuybackBurn,
      state.releasedBuybackBurn,
      state.lifetimeBuybackBurn,
      "buyback",
    );

    assertBucketInvariant(
      state.pendingLiquidity,
      state.releasedLiquidity,
      state.lifetimeLiquidity,
      "liquidity",
    );

    assertBucketInvariant(
      state.pendingCompany,
      state.releasedCompany,
      state.lifetimeCompany,
      "company",
    );

    assertBucketInvariant(
      state.pendingFounder,
      state.releasedFounder,
      state.lifetimeFounder,
      "founder",
    );

    const lifetimeTotal = state.lifetimeReserve
      .add(state.lifetimeBuybackBurn)
      .add(state.lifetimeLiquidity)
      .add(state.lifetimeCompany)
      .add(state.lifetimeFounder);

    assertBn(
      lifetimeTotal,
      state.totalFeesAllocated,
      "sum of lifetime buckets must equal total fees allocated",
    );

    assert(
      state.totalFeesAllocated.lte(state.totalFeesReceived),
      "allocated fees cannot exceed received fees",
    );
  }

  before(async () => {
    assert.equal(
      program.programId.toBase58(),
      PROGRAM_ID.toBase58(),
      "workspace program ID does not match locked RBVR program ID",
    );

    derivePdas();

    settlementMint = await createMint(
      provider.connection,
      payer,
      authority,
      null,
      6,
    );

    const sourceAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      settlementMint,
      authority,
    );

    sourceTokenAccount = sourceAta.address;

    founderDestination = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        founderRecipientOwner.publicKey,
      )
    ).address;

    companyDestination = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        companyRecipientOwner.publicKey,
      )
    ).address;

    reserveDestination = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        reserveRecipientOwner.publicKey,
      )
    ).address;

    liquidityDestination = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        liquidityRecipientOwner.publicKey,
      )
    ).address;

    buybackDestination = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        buybackRecipientOwner.publicKey,
      )
    ).address;

    await mintTo(
      provider.connection,
      payer,
      settlementMint,
      sourceTokenAccount,
      authority,
      BigInt(DEPOSIT_AMOUNT.toString()),
    );
  });

  it("derives every locked protocol PDA", async () => {
    assert.equal(
      protocolStatePda.toBase58(),
      PublicKey.findProgramAddressSync(
        [Buffer.from("protocol")],
        PROGRAM_ID,
      )[0].toBase58(),
    );

    assert.notEqual(protocolConfigPda.toBase58(), treasuryStatePda.toBase58());

    assert.notEqual(founderStatePda.toBase58(), companyStatePda.toBase58());
  });

  it("initializes the protocol state", async () => {
    await program.methods
      .initialize()
      .accountsPartial({
        protocolState: protocolStatePda,
        authority,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const state = await program.account.protocolState.fetch(protocolStatePda);

    assert.equal(state.authority.toBase58(), authority.toBase58());

    assert.equal(state.paused, false);
  });

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
  });

  it("initializes the locked protocol allocation", async () => {
    await program.methods
      .initializeProtocolConfig()
      .accountsPartial({
        protocolState: protocolStatePda,
        protocolConfig: protocolConfigPda,
        authority,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const config =
      await program.account.protocolConfig.fetch(protocolConfigPda);

    assert.equal(config.reserveBps, 3000);
    assert.equal(config.buybackBurnBps, 2000);
    assert.equal(config.liquidityBps, 2000);
    assert.equal(config.companyBps, 2000);
    assert.equal(config.founderBps, 1000);

    assert.equal(
      config.reserveBps +
        config.buybackBurnBps +
        config.liquidityBps +
        config.companyBps +
        config.founderBps,
      10_000,
    );

    const protocol =
      await program.account.protocolState.fetch(protocolStatePda);

    assert.equal(
      protocol.protocolConfig.toBase58(),
      protocolConfigPda.toBase58(),
    );
  });

  it("initializes the settlement treasury and vault", async () => {
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

    const state = await treasury();

    assert.equal(state.protocol.toBase58(), protocolStatePda.toBase58());

    assert.equal(state.settlementMint.toBase58(), settlementMint.toBase58());

    assert.equal(
      state.settlementVault.toBase58(),
      settlementVaultPda.toBase58(),
    );

    assertBn(state.totalFeesReceived, 0);
    assertBn(state.totalFeesAllocated, 0);

    const vault = await getAccount(provider.connection, settlementVaultPda);

    assert.equal(vault.owner.toBase58(), treasuryStatePda.toBase58());
  });

  it("initializes founder compensation controls", async () => {
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

    const founder = await program.account.founderState.fetch(founderStatePda);

    assert.equal(
      founder.recipient.toBase58(),
      founderRecipientOwner.publicKey.toBase58(),
      "founder recipient must be the owner of the destination token account",
    );

    assertBn(founder.periodCap, FOUNDER_CAP);
    assertBn(founder.periodDuration, PERIOD_DURATION);
    assert.equal(founder.enabled, true);
  });

  it("initializes company allocation controls", async () => {
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

    const company = await program.account.companyState.fetch(companyStatePda);

    assert.equal(
      company.recipient.toBase58(),
      companyRecipientOwner.publicKey.toBase58(),
      "company recipient must be the owner of the destination token account",
    );

    assertBn(company.periodCap, COMPANY_CAP);
    assertBn(company.periodDuration, PERIOD_DURATION);
    assert.equal(company.enabled, true);
  });

  it("deposits settlement assets into the treasury vault", async () => {
    const sourceBefore = await getAccount(
      provider.connection,
      sourceTokenAccount,
    );

    await program.methods
      .depositSettlement(DEPOSIT_AMOUNT)
      .accountsPartial({
        protocolState: protocolStatePda,
        treasury: treasuryStatePda,
        settlementMint,
        sourceTokenAccount,
        settlementVault: settlementVaultPda,
        authority,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const sourceAfter = await getAccount(
      provider.connection,
      sourceTokenAccount,
    );

    const vaultAfter = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    assert.equal(
      sourceBefore.amount - sourceAfter.amount,
      BigInt(DEPOSIT_AMOUNT.toString()),
    );

    assert.equal(vaultAfter.amount, BigInt(DEPOSIT_AMOUNT.toString()));

    /*
     * depositSettlement transfers settlement tokens into the vault.
     * Fee-receipt accounting is recorded by processFees, not by the
     * deposit instruction itself.
     */
  });

  it("initializes the execution config", async () => {
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

    const config =
      await program.account.executionConfig.fetch(executionConfigPda);

    assert.equal(config.protocolState.toBase58(), protocolStatePda.toBase58());

    assert.equal(config.settlementMint.toBase58(), settlementMint.toBase58());

    assert.equal(
      config.reserveDestination.toBase58(),
      reserveDestination.toBase58(),
    );

    assert.equal(
      config.buybackDestination.toBase58(),
      buybackDestination.toBase58(),
    );

    assert.equal(
      config.liquidityDestination.toBase58(),
      liquidityDestination.toBase58(),
    );

    assert.equal(
      config.companyDestination.toBase58(),
      companyDestination.toBase58(),
    );

    assert.equal(
      config.founderDestination.toBase58(),
      founderDestination.toBase58(),
    );

    assert.equal(config.version, 1);
  });

  it("processes fees using the locked 30/20/20/20/10 model", async () => {
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

    const state = await treasury();

    assertBn(state.totalFeesAllocated, 1_000_000);

    assertBn(state.pendingReserve, 300_000);
    assertBn(state.pendingBuybackBurn, 200_000);
    assertBn(state.pendingLiquidity, 200_000);
    assertBn(state.pendingCompany, 200_000);
    assertBn(state.pendingFounder, 100_000);

    assertBn(state.lifetimeReserve, 300_000);
    assertBn(state.lifetimeBuybackBurn, 200_000);
    assertBn(state.lifetimeLiquidity, 200_000);
    assertBn(state.lifetimeCompany, 200_000);
    assertBn(state.lifetimeFounder, 100_000);

    assertBn(state.releasedReserve, 0);
    assertBn(state.releasedBuybackBurn, 0);
    assertBn(state.releasedLiquidity, 0);
    assertBn(state.releasedCompany, 0);
    assertBn(state.releasedFounder, 0);

    assertBn(state.processingEpoch, 1);

    await assertAllTreasuryInvariants();
  });

  it("rejects duplicate execution destinations through the Integrity Firewall", async () => {
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

          // Deliberate architecture violation:
          // Reserve and Buyback point to the same token account.
          reserveDestination,
          buybackDestination: reserveDestination,

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

    assert.isTrue(rejected, "Duplicate destinations must be rejected");

    assert.match(
      failureText,
      /IntegrityFirewallViolation|Integrity Firewall|integrity firewall/i,
      `Unexpected rejection: ${failureText}`,
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
      vaultAfter.amount.toString(),
      vaultBefore.amount.toString(),
      "Failed firewall transaction changed the treasury vault balance",
    );

    assert.equal(
      reserveAfter.amount.toString(),
      reserveBefore.amount.toString(),
      "Failed firewall transaction changed the reserve destination balance",
    );

    assert.equal(
      treasuryAfter.pendingReserve.toString(),
      treasuryBefore.pendingReserve.toString(),
      "Failed firewall transaction changed pending reserve accounting",
    );

    assert.equal(
      treasuryAfter.releasedReserve.toString(),
      treasuryBefore.releasedReserve.toString(),
      "Failed firewall transaction changed released reserve accounting",
    );

    await assertAllTreasuryInvariants();
  });

  it("autonomously transfers the reserve allocation", async () => {
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
  });
});
