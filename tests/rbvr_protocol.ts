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

  const FOUNDER_PRICE_FEED_ID = Array<number>(32).fill(7);
  const FOUNDER_MAX_PRICE_AGE_SECONDS = new anchor.BN(300);
  const FOUNDER_MAX_CONFIDENCE_BPS = 100;

  const ORACLE_ADAPTER_PROGRAM_ID = new PublicKey(
    "E5T6hAp6pTE9iZvwMMRaUD7kDC3UkJWiiqi24fJf3sqx",
  );

  const [ORACLE_ADAPTER_AUTHORITY] =
    PublicKey.findProgramAddressSync(
      [Buffer.from("rbvr-oracle-authority")],
      ORACLE_ADAPTER_PROGRAM_ID,
    );

  let settlementMint: PublicKey;
  let sourceTokenAccount: PublicKey;

  let protocolStatePda: PublicKey;
  let protocolConfigPda: PublicKey;
  let treasuryStatePda: PublicKey;
  let settlementVaultPda: PublicKey;
  let founderStatePda: PublicKey;
  let founderUsdCapPda: PublicKey;
  let founderPricePda: PublicKey;
  let companyStatePda: PublicKey;
  let executionConfigPda: PublicKey;

  let founderDestination: PublicKey;
  let companyDestination: PublicKey;
  let reserveDestination: PublicKey;
  let liquidityDestination: PublicKey;
  let buybackDestination: PublicKey;

  const founderRecipientOwner = Keypair.generate();
  const companyRecipientOwner = Keypair.generate();
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

    [reserveDestination] = PublicKey.findProgramAddressSync(
      [Buffer.from("reserve-vault"), treasuryStatePda.toBuffer()],
      PROGRAM_ID,
    );

    [founderStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("founder-state"), protocolStatePda.toBuffer()],
      PROGRAM_ID,
    );

    [founderUsdCapPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("founder-usd-cap"), protocolStatePda.toBuffer()],
      PROGRAM_ID,
    );

    [founderPricePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("founder-price"), protocolStatePda.toBuffer()],
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

    assert.equal(
      reserveDestination.toBase58(),
      PublicKey.findProgramAddressSync(
        [Buffer.from("reserve-vault"), treasuryStatePda.toBuffer()],
        PROGRAM_ID,
      )[0].toBase58(),
      "Reserve Vault must use the canonical PDA",
    );
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

  it("initializes the Founder USD annual cap", async () => {
    await program.methods
      .initializeFounderUsdCap(
        FOUNDER_PRICE_FEED_ID,
        FOUNDER_MAX_PRICE_AGE_SECONDS,
        FOUNDER_MAX_CONFIDENCE_BPS,
      )
      .accountsPartial({
        protocolState: protocolStatePda,
        treasuryState: treasuryStatePda,
        settlementMint,
        founderUsdCap: founderUsdCapPda,
        authority,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const founderUsdCap =
      await program.account.founderUsdCapState.fetch(
        founderUsdCapPda,
      );

    assert.equal(
      founderUsdCap.protocol.toBase58(),
      protocolStatePda.toBase58(),
    );

    assert.equal(
      founderUsdCap.settlementMint.toBase58(),
      settlementMint.toBase58(),
    );

    assert.deepEqual(
      Array.from(founderUsdCap.priceFeedId),
      FOUNDER_PRICE_FEED_ID,
    );

    assertBn(
      founderUsdCap.annualCapUsdE6,
      new anchor.BN(3_000_000_000_000),
    );

    assertBn(
      founderUsdCap.periodDuration,
      new anchor.BN(31_536_000),
    );

    assertBn(founderUsdCap.earnedCurrentPeriodUsdE6, 0);
    assertBn(founderUsdCap.lifetimeEarnedUsdE6, 0);
    assert.equal(founderUsdCap.enabled, true);
  });

  it("initializes the Founder verified-price account", async () => {
    await program.methods
      .initializeFounderPrice(
        FOUNDER_PRICE_FEED_ID,
        authority,
        FOUNDER_MAX_PRICE_AGE_SECONDS,
        FOUNDER_MAX_CONFIDENCE_BPS,
      )
      .accountsPartial({
        protocolState: protocolStatePda,
        treasuryState: treasuryStatePda,
        settlementMint,
        founderPrice: founderPricePda,
        authority,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const founderPrice =
      await program.account.founderPriceState.fetch(
        founderPricePda,
      );

    assert.equal(
      founderPrice.protocol.toBase58(),
      protocolStatePda.toBase58(),
    );

    assert.equal(
      founderPrice.settlementMint.toBase58(),
      settlementMint.toBase58(),
    );

    assert.equal(
      founderPrice.oracleAdapterAuthority.toBase58(),
      authority.toBase58(),
    );

    assert.deepEqual(
      Array.from(founderPrice.priceFeedId),
      FOUNDER_PRICE_FEED_ID,
    );

    assertBn(
      founderPrice.maxPriceAgeSeconds,
      FOUNDER_MAX_PRICE_AGE_SECONDS,
    );

    assert.equal(
      founderPrice.maxConfidenceBps,
      FOUNDER_MAX_CONFIDENCE_BPS,
    );

    assert.equal(founderPrice.enabled, true);
    assertBn(founderPrice.sequence, 0);
  });

  it("submits a valid Founder price for local fee tests", async () => {
    const slot = await provider.connection.getSlot();
    const blockTime =
      await provider.connection.getBlockTime(slot);

    const publishTime = new anchor.BN(
      blockTime ?? Math.floor(Date.now() / 1_000),
    );

    await program.methods
      .submitFounderPrice(
        FOUNDER_PRICE_FEED_ID,
        new anchor.BN(1_000_000),
        -6,
        new anchor.BN(1_000),
        publishTime,
      )
      .accountsPartial({
        protocolState: protocolStatePda,
        founderPrice: founderPricePda,
        oracleAdapterAuthority: authority,
      })
      .rpc();

    const founderPrice =
      await program.account.founderPriceState.fetch(
        founderPricePda,
      );

    assertBn(founderPrice.price, 1_000_000);
    assert.equal(founderPrice.exponent, -6);
    assertBn(founderPrice.confidence, 1_000);
    assertBn(founderPrice.publishTime, publishTime);
    assertBn(founderPrice.sequence, 1);
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
        tokenProgram: TOKEN_PROGRAM_ID,
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

    const reserveVault = await getAccount(
      provider.connection,
      reserveDestination,
    );

    assert.equal(
      reserveVault.mint.toBase58(),
      settlementMint.toBase58(),
      "Reserve Vault must use the settlement mint",
    );

    assert.equal(
      reserveVault.owner.toBase58(),
      treasuryStatePda.toBase58(),
      "TreasuryState must be the Reserve Vault token authority",
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
          tokenProgram: TOKEN_PROGRAM_ID,
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

  it("processes fees using the locked 30/20/20/20/10 model", async () => {
    await program.methods
      .processFees()
      .accountsPartial({
        settlementMint,
        protocolState: protocolStatePda,
        protocolConfig: protocolConfigPda,
        treasury: treasuryStatePda,
        founderState: founderStatePda,
        founderUsdCap: founderUsdCapPda,
        founderPrice: founderPricePda,
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
          settlementMint,
          protocolState: protocolStatePda,
          protocolConfig: protocolConfigPda,
          treasury: treasuryStatePda,
          founderState: founderStatePda,
          founderUsdCap: founderUsdCapPda,
          founderPrice: founderPricePda,
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


  it("RT-006A rejects substituted founder destination without mutation", async () => {
    const attackerDestination = Keypair.generate();

    const founderBefore = await getAccount(
      provider.connection,
      founderDestination,
    );

    const treasuryBefore = await treasury();

    let rejected = false;

    try {
      await program.methods
        .authorizeFounderExecution()
        .accountsPartial({
          protocolState: protocolStatePda,
          founderState: founderStatePda,
          treasury: treasuryStatePda,
          executionConfig: executionConfigPda,
          settlementMint,
          settlementVault: settlementVaultPda,
          founderDestination: attackerDestination.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
    } catch (e) {
      rejected = true;
    }

    assert.isTrue(
      rejected,
      "attacker founder destination must be rejected",
    );

    const founderAfter = await getAccount(
      provider.connection,
      founderDestination,
    );

    const treasuryAfter = await treasury();

    assert.equal(
      founderAfter.amount.toString(),
      founderBefore.amount.toString(),
      "founder destination changed after rejected attack",
    );

    assert.equal(
      treasuryAfter.pendingFounder.toString(),
      treasuryBefore.pendingFounder.toString(),
      "treasury accounting changed after rejected attack",
    );
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

  it("deploys only Reserve surplus through the permissionless Spillway", async () => {
    const [reservePolicyPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("reserve-policy"),
        treasuryStatePda.toBuffer(),
      ],
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

    assert.isTrue(
      reserveBefore.amount > 100_000n,
      "Reserve Vault must contain enough value to test a protected floor",
    );

    /*
     * Lock the absolute floor exactly 100,000 base units beneath the
     * current Reserve balance. With a 50% deployment rate, the first
     * Spillway execution must deploy exactly 50,000 units.
     *
     * The liquidity-floor rate is zero in this integration test so the
     * expected amount depends only on the absolute immutable floor.
     */
    const minimumFloor = reserveBefore.amount - 100_000n;

    await program.methods
      .initializeReservePolicy(
        new anchor.BN(minimumFloor.toString()),
        0,
        5_000,
        new anchor.BN(86_400),
      )
      .accountsPartial({
        protocolState: protocolStatePda,
        treasury: treasuryStatePda,
        reservePolicy: reservePolicyPda,
        authority,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const policyBefore =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    assert.equal(
      policyBefore.minimumReserveFloor.toString(),
      minimumFloor.toString(),
    );

    assert.equal(policyBefore.liquidityFloorBps, 0);
    assert.equal(policyBefore.surplusDeploymentBps, 5_000);
    assert.equal(policyBefore.lifetimeDeployed.toString(), "0");

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

    const expectedDeployment = 50_000n;

    assert.equal(
      (reserveBefore.amount - reserveAfter.amount).toString(),
      expectedDeployment.toString(),
      "Spillway must debit only the policy-approved surplus amount",
    );

    assert.equal(
      (liquidityAfter.amount - liquidityBefore.amount).toString(),
      expectedDeployment.toString(),
      "Spillway must send the approved amount to Liquidity Growth",
    );

    assert.isAtLeast(
      Number(reserveAfter.amount),
      Number(minimumFloor),
      "Spillway must never breach the protected Reserve floor",
    );

    assert.equal(
      policyAfter.lifetimeDeployed.toString(),
      expectedDeployment.toString(),
    );

    assert.isAbove(
      Number(policyAfter.lastDeployedAt),
      0,
      "Successful Spillway execution must record its timestamp",
    );
  });

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
});
