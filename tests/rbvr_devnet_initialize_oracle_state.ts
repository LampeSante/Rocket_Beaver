import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import type { TreasuryRouter } from "../target/types/treasury_router";
import { assert } from "chai";

describe("RBVR Devnet Oracle State Initialization", function () {
  this.timeout(1_000_000);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program =
    anchor.workspace.TreasuryRouter as Program<TreasuryRouter>;

  const authority = provider.wallet.publicKey;

  const TREASURY_PROGRAM_ID = new PublicKey(
    "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3",
  );

  const ORACLE_ADAPTER_PROGRAM_ID = new PublicKey(
    "E5T6hAp6pTE9iZvwMMRaUD7kDC3UkJWiiqi24fJf3sqx",
  );

  const PRICE_FEED_ID = Array<number>(32).fill(7);
  const MAX_PRICE_AGE_SECONDS = new anchor.BN(300);
  const MAX_CONFIDENCE_BPS = 100;

  const [protocolStatePda] =
    PublicKey.findProgramAddressSync(
      [Buffer.from("protocol")],
      TREASURY_PROGRAM_ID,
    );

  const [treasuryStatePda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("treasury"),
        protocolStatePda.toBuffer(),
      ],
      TREASURY_PROGRAM_ID,
    );

  const [founderUsdCapPda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-usd-cap"),
        protocolStatePda.toBuffer(),
      ],
      TREASURY_PROGRAM_ID,
    );

  const [founderPricePda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-price"),
        protocolStatePda.toBuffer(),
      ],
      TREASURY_PROGRAM_ID,
    );

  const [oracleAdapterAuthority] =
    PublicKey.findProgramAddressSync(
      [Buffer.from("rbvr-oracle-authority")],
      ORACLE_ADAPTER_PROGRAM_ID,
    );

  before("confirms Devnet and protocol authority", async () => {
    assert.match(
      provider.connection.rpcEndpoint,
      /devnet/i,
      "Refusing initialization against a non-Devnet RPC",
    );

    const protocol =
      await program.account.protocolState.fetch(
        protocolStatePda,
      );

    assert.equal(
      protocol.authority.toBase58(),
      authority.toBase58(),
      "Connected wallet is not the protocol authority",
    );
  });

  it("initializes the missing Founder USD oracle accounts", async () => {
    const treasury =
      await program.account.treasuryState.fetch(
        treasuryStatePda,
      );

    const capInfo =
      await provider.connection.getAccountInfo(
        founderUsdCapPda,
        "confirmed",
      );

    if (capInfo === null) {
      const signature = await program.methods
        .initializeFounderUsdCap(
          PRICE_FEED_ID,
          MAX_PRICE_AGE_SECONDS,
          MAX_CONFIDENCE_BPS,
        )
        .accountsPartial({
          protocolState: protocolStatePda,
          treasuryState: treasuryStatePda,
          settlementMint: treasury.settlementMint,
          founderUsdCap: founderUsdCapPda,
          authority,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log(`FounderUsdCapState initialized: ${signature}`);
    } else {
      console.log("FounderUsdCapState already exists.");
    }

    const priceInfo =
      await provider.connection.getAccountInfo(
        founderPricePda,
        "confirmed",
      );

    if (priceInfo === null) {
      const signature = await program.methods
        .initializeFounderPrice(
          PRICE_FEED_ID,
          oracleAdapterAuthority,
          MAX_PRICE_AGE_SECONDS,
          MAX_CONFIDENCE_BPS,
        )
        .accountsPartial({
          protocolState: protocolStatePda,
          treasuryState: treasuryStatePda,
          settlementMint: treasury.settlementMint,
          founderPrice: founderPricePda,
          authority,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log(`FounderPriceState initialized: ${signature}`);
    } else {
      console.log("FounderPriceState already exists.");
    }

    const cap =
      await program.account.founderUsdCapState.fetch(
        founderUsdCapPda,
      );

    const price =
      await program.account.founderPriceState.fetch(
        founderPricePda,
      );

    assert.equal(
      cap.annualCapUsdE6.toString(),
      "3000000000000",
    );

    assert.equal(
      cap.periodDuration.toString(),
      "31536000",
    );

    assert.equal(
      price.oracleAdapterAuthority.toBase58(),
      oracleAdapterAuthority.toBase58(),
    );

    console.log(
      JSON.stringify(
        {
          founderUsdCapPda: founderUsdCapPda.toBase58(),
          founderPricePda: founderPricePda.toBase58(),
          settlementMint: treasury.settlementMint.toBase58(),
          annualCapUsdE6: cap.annualCapUsdE6.toString(),
          periodDuration: cap.periodDuration.toString(),
          oracleAdapterAuthority:
            price.oracleAdapterAuthority.toBase58(),
        },
        null,
        2,
      ),
    );
  });
});
