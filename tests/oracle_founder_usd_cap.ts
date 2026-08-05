import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import type { TreasuryRouter } from "../target/types/treasury_router";
import type { RbvrOracleAdapter } from "../target/types/rbvr_oracle_adapter";
import {
  createMint,
  getAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { assert } from "chai";

describe("Oracle → Founder USD Cap integration", () => {
  const provider = anchor.AnchorProvider.env();

  anchor.setProvider(provider);

  const treasuryRouter =
    anchor.workspace.TreasuryRouter as Program<TreasuryRouter>;

  const oracleAdapter =
    anchor.workspace.RbvrOracleAdapter as Program<RbvrOracleAdapter>;

  const authority = provider.wallet.publicKey;

  const TREASURY_ROUTER_PROGRAM_ID = new PublicKey(
    "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3",
  );

  const [protocolStatePda] =
    PublicKey.findProgramAddressSync(
      [Buffer.from("protocol")],
      TREASURY_ROUTER_PROGRAM_ID,
    );

  const [treasuryStatePda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("treasury"),
        protocolStatePda.toBuffer(),
      ],
      TREASURY_ROUTER_PROGRAM_ID,
    );

  const [settlementVaultPda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("treasury-vault"),
        treasuryStatePda.toBuffer(),
      ],
      TREASURY_ROUTER_PROGRAM_ID,
    );

  let settlementMint: PublicKey;

  const FOUNDER_PRICE_FEED_ID = Array<number>(32).fill(7);
  const FOUNDER_MAX_PRICE_AGE_SECONDS = new anchor.BN(300);
  const FOUNDER_MAX_CONFIDENCE_BPS = 100;
  const FOUNDER_ANNUAL_CAP_USD_E6 =
    new anchor.BN(3_000_000_000_000);
  const FOUNDER_ANNUAL_PERIOD_SECONDS =
    new anchor.BN(31_536_000);

  const [founderUsdCapPda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-usd-cap"),
        protocolStatePda.toBuffer(),
      ],
      TREASURY_ROUTER_PROGRAM_ID,
    );

  const ORACLE_ADAPTER_PROGRAM_ID = new PublicKey(
    "E5T6hAp6pTE9iZvwMMRaUD7kDC3UkJWiiqi24fJf3sqx",
  );

  const [oracleAdapterAuthority] =
    PublicKey.findProgramAddressSync(
      [Buffer.from("rbvr-oracle-authority")],
      ORACLE_ADAPTER_PROGRAM_ID,
    );

  const [founderPricePda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-price"),
        protocolStatePda.toBuffer(),
      ],
      TREASURY_ROUTER_PROGRAM_ID,
    );

  it("loads both local programs", async () => {
    assert.isDefined(treasuryRouter);
    assert.isDefined(oracleAdapter);

    assert.equal(
      treasuryRouter.programId.toBase58(),
      "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3",
    );

    assert.equal(
      oracleAdapter.programId.toBase58(),
      "E5T6hAp6pTE9iZvwMMRaUD7kDC3UkJWiiqi24fJf3sqx",
    );
  });

  it("initializes protocol state", async () => {
    await treasuryRouter.methods
      .initialize()
      .accountsPartial({
        protocolState: protocolStatePda,
        authority,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const state =
      await treasuryRouter.account.protocolState.fetch(
        protocolStatePda,
      );

    assert.equal(
      state.authority.toBase58(),
      authority.toBase58(),
      "Protocol authority must equal the provider wallet",
    );

    assert.equal(
      state.bump,
      PublicKey.findProgramAddressSync(
        [Buffer.from("protocol")],
        TREASURY_ROUTER_PROGRAM_ID,
      )[1],
      "Stored ProtocolState bump must be canonical",
    );
  });

  it("initializes the settlement Treasury", async () => {
    const payer =
      (provider.wallet as anchor.Wallet).payer;

    settlementMint = await createMint(
      provider.connection,
      payer,
      authority,
      null,
      6,
    );

    await treasuryRouter.methods
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

    const treasury =
      await treasuryRouter.account.treasuryState.fetch(
        treasuryStatePda,
      );

    assert.equal(
      treasury.protocol.toBase58(),
      protocolStatePda.toBase58(),
    );

    assert.equal(
      treasury.settlementMint.toBase58(),
      settlementMint.toBase58(),
    );

    assert.equal(
      treasury.settlementVault.toBase58(),
      settlementVaultPda.toBase58(),
    );

    const vault = await getAccount(
      provider.connection,
      settlementVaultPda,
    );

    assert.equal(
      vault.mint.toBase58(),
      settlementMint.toBase58(),
    );

    assert.equal(
      vault.owner.toBase58(),
      treasuryStatePda.toBase58(),
    );
  });

  it("initializes Founder USD Cap", async () => {
    await treasuryRouter.methods
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

    const state =
      await treasuryRouter.account.founderUsdCapState.fetch(
        founderUsdCapPda,
      );

    assert.equal(
      state.protocol.toBase58(),
      protocolStatePda.toBase58(),
    );

    assert.equal(
      state.settlementMint.toBase58(),
      settlementMint.toBase58(),
    );

    assert.deepEqual(
      Array.from(state.priceFeedId),
      FOUNDER_PRICE_FEED_ID,
    );

    assert.equal(
      state.annualCapUsdE6.toString(),
      FOUNDER_ANNUAL_CAP_USD_E6.toString(),
    );

    assert.equal(
      state.periodDuration.toString(),
      FOUNDER_ANNUAL_PERIOD_SECONDS.toString(),
    );

    assert.equal(
      state.earnedCurrentPeriodUsdE6.toString(),
      "0",
    );

    assert.equal(
      state.lifetimeEarnedUsdE6.toString(),
      "0",
    );

    assert.isTrue(state.enabled);
  });

  it("initializes Founder Price", async () => {
    await treasuryRouter.methods
      .initializeFounderPrice(
        FOUNDER_PRICE_FEED_ID,
        oracleAdapterAuthority,
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

    const state =
      await treasuryRouter.account.founderPriceState.fetch(
        founderPricePda,
      );

    assert.equal(
      state.protocol.toBase58(),
      protocolStatePda.toBase58(),
    );

    assert.equal(
      state.settlementMint.toBase58(),
      settlementMint.toBase58(),
    );

    assert.equal(
      state.oracleAdapterAuthority.toBase58(),
      oracleAdapterAuthority.toBase58(),
    );

    assert.deepEqual(
      Array.from(state.priceFeedId),
      FOUNDER_PRICE_FEED_ID,
    );

    assert.equal(
      state.maxPriceAgeSeconds.toString(),
      FOUNDER_MAX_PRICE_AGE_SECONDS.toString(),
    );

    assert.equal(
      state.maxConfidenceBps,
      FOUNDER_MAX_CONFIDENCE_BPS,
    );

    assert.equal(state.sequence.toString(), "0");
    assert.isTrue(state.enabled);
  });

  it.skip(
    "submits a verified oracle price through the adapter",
    async () => {},
  );

  it.skip(
    "processes fees using the stored oracle price",
    async () => {},
  );

  it.skip(
    "enforces the US$3M annual Founder cap",
    async () => {},
  );

  it.skip(
    "redirects Founder overflow to Liquidity Growth",
    async () => {},
  );

  it.skip("preserves accounting invariants", async () => {});
});
