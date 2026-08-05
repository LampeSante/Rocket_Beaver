import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import type { TreasuryRouter } from "../target/types/treasury_router";
import { assert } from "chai";

describe("RBVR Devnet Founder Feed Migration", function () {
  this.timeout(1_000_000);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program =
    anchor.workspace.TreasuryRouter as Program<TreasuryRouter>;

  const authority = provider.wallet.publicKey;

  const PROGRAM_ID = new PublicKey(
    "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3",
  );

  const PLACEHOLDER_FEED_ID = Array<number>(32).fill(7);

  const PYTH_USDC_USD_FEED_ID = Array.from(
    Buffer.from(
      "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a",
      "hex",
    ),
  );

  const [protocolStatePda] =
    PublicKey.findProgramAddressSync(
      [Buffer.from("protocol")],
      PROGRAM_ID,
    );

  const [founderUsdCapPda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-usd-cap"),
        protocolStatePda.toBuffer(),
      ],
      PROGRAM_ID,
    );

  const [founderPricePda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-price"),
        protocolStatePda.toBuffer(),
      ],
      PROGRAM_ID,
    );

  before("confirms Devnet and authority", async () => {
    assert.match(
      provider.connection.rpcEndpoint,
      /devnet/i,
      "Refusing migration against non-Devnet RPC",
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

  it("migrates the Founder feed once and rejects replay", async () => {
    const capBefore =
      await program.account.founderUsdCapState.fetch(
        founderUsdCapPda,
      );

    const priceBefore =
      await program.account.founderPriceState.fetch(
        founderPricePda,
      );

    const capFeedBefore =
      Array.from(capBefore.priceFeedId);

    const priceFeedBefore =
      Array.from(priceBefore.priceFeedId);

    assert.deepEqual(
      capFeedBefore,
      priceFeedBefore,
      "Founder accounts must start with matching feed IDs",
    );

    if (
      Buffer.from(capFeedBefore).equals(
        Buffer.from(PLACEHOLDER_FEED_ID),
      )
    ) {
      const signature = await program.methods
        .migrateFounderFeedId(PYTH_USDC_USD_FEED_ID)
        .accountsPartial({
          protocolState: protocolStatePda,
          founderUsdCap: founderUsdCapPda,
          founderPrice: founderPricePda,
          authority,
        })
        .rpc();

      console.log(`Founder feed migration: ${signature}`);
    } else {
      assert.deepEqual(
        capFeedBefore,
        PYTH_USDC_USD_FEED_ID,
        "Existing feed is neither placeholder nor expected USDC/USD",
      );

      console.log("Founder feed was already migrated.");
    }

    const capAfter =
      await program.account.founderUsdCapState.fetch(
        founderUsdCapPda,
      );

    const priceAfter =
      await program.account.founderPriceState.fetch(
        founderPricePda,
      );

    assert.deepEqual(
      Array.from(capAfter.priceFeedId),
      PYTH_USDC_USD_FEED_ID,
    );

    assert.deepEqual(
      Array.from(priceAfter.priceFeedId),
      PYTH_USDC_USD_FEED_ID,
    );

    assert.equal(priceAfter.sequence.toString(), "0");
    assert.equal(
      capAfter.earnedCurrentPeriodUsdE6.toString(),
      "0",
    );
    assert.equal(
      capAfter.lifetimeEarnedUsdE6.toString(),
      "0",
    );

    let replayRejected = false;
    let replayFailure = "";

    try {
      await program.methods
        .migrateFounderFeedId(Array<number>(32).fill(9))
        .accountsPartial({
          protocolState: protocolStatePda,
          founderUsdCap: founderUsdCapPda,
          founderPrice: founderPricePda,
          authority,
        })
        .rpc();
    } catch (error) {
      replayRejected = true;
      replayFailure = String(error);
    }

    assert.isTrue(
      replayRejected,
      "A second feed migration must be permanently rejected",
    );

    assert.match(
      replayFailure,
      /FounderFeedMigrationUnavailable|migration is no longer available|migration/i,
      `Unexpected replay failure: ${replayFailure}`,
    );

    const capFinal =
      await program.account.founderUsdCapState.fetch(
        founderUsdCapPda,
      );

    const priceFinal =
      await program.account.founderPriceState.fetch(
        founderPricePda,
      );

    assert.deepEqual(
      Array.from(capFinal.priceFeedId),
      PYTH_USDC_USD_FEED_ID,
    );

    assert.deepEqual(
      Array.from(priceFinal.priceFeedId),
      PYTH_USDC_USD_FEED_ID,
    );

    console.log(
      JSON.stringify(
        {
          founderUsdCapPda: founderUsdCapPda.toBase58(),
          founderPricePda: founderPricePda.toBase58(),
          feedIdHex:
            Buffer.from(PYTH_USDC_USD_FEED_ID).toString("hex"),
          sequence: priceFinal.sequence.toString(),
          earnedCurrentPeriodUsdE6:
            capFinal.earnedCurrentPeriodUsdE6.toString(),
          lifetimeEarnedUsdE6:
            capFinal.lifetimeEarnedUsdE6.toString(),
          replayRejected,
        },
        null,
        2,
      ),
    );
  });
});
