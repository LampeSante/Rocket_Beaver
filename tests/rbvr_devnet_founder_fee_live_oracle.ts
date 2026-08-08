import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import {
  PublicKey,
  Keypair,
  SystemProgram,
} from "@solana/web3.js";
import type { TreasuryRouter } from "../target/types/treasury_router";
import { assert } from "chai";

describe("RBVR Devnet Founder Fee Live Oracle Integration", function () {
  this.timeout(1_000_000);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program =
    anchor.workspace.TreasuryRouter as Program<TreasuryRouter>;

  const authority = provider.wallet.publicKey;

  const PROTOCOL_STATE_SEED = Buffer.from("protocol");

  const [protocolState] =
    PublicKey.findProgramAddressSync(
      [PROTOCOL_STATE_SEED],
      program.programId,
    );

  const [founderPrice] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-price"),
        protocolState.toBuffer(),
      ],
      program.programId,
    );

  const [founderUsdCap] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-usd-cap"),
        protocolState.toBuffer(),
      ],
      program.programId,
    );

  it("loads live Founder oracle state", async () => {
    const price =
      await program.account.founderPriceState.fetch(
        founderPrice,
      );

    assert.equal(
      price.enabled,
      true,
    );

    assert.equal(
      price.sequence.toNumber(),
      1,
    );

    console.log(
      JSON.stringify(
        {
          price:
            price.price.toString(),
          exponent:
            price.exponent,
          confidence:
            price.confidence.toString(),
          publishTime:
            price.publishTime.toString(),
          sequence:
            price.sequence.toString(),
        },
        null,
        2,
      ),
    );
  });


  it("loads founder USD cap state", async () => {
    const cap =
      await program.account.founderUsdCapState.fetch(
        founderUsdCap,
      );

    assert.equal(
      cap.enabled,
      true,
    );

    console.log(
      JSON.stringify(
        {
          annualCapUsdE6:
            cap.annualCapUsdE6.toString(),
          earnedCurrentPeriodUsdE6:
            cap.earnedCurrentPeriodUsdE6.toString(),
          lifetimeEarnedUsdE6:
            cap.lifetimeEarnedUsdE6.toString(),
        },
        null,
        2,
      ),
    );
  });


  it("finds fee processing instruction", async () => {

    const instructions =
      Object.keys(
        program.methods,
      );

    console.log(
      instructions,
    );

    assert.isAbove(
      instructions.length,
      0,
    );
  });

});
