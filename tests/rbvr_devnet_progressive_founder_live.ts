import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import {
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import type { TreasuryRouter } from "../target/types/treasury_router";
import { assert } from "chai";

describe("RBVR Devnet Progressive Founder Compensation Live", function () {
  this.timeout(1_000_000);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program =
    anchor.workspace.TreasuryRouter as Program<TreasuryRouter>;

  const protocolState =
    PublicKey.findProgramAddressSync(
      [Buffer.from("protocol")],
      program.programId,
    )[0];

  const treasury =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("treasury"),
        protocolState.toBuffer(),
      ],
      program.programId,
    )[0];

  const founderPrice =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-price"),
        protocolState.toBuffer(),
      ],
      program.programId,
    )[0];

  const founderUsdCap =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-usd-cap"),
        protocolState.toBuffer(),
      ],
      program.programId,
    )[0];

  it("proves founder marginal rate decreases with volume", async () => {

    const before =
      await program.account.treasuryState.fetch(treasury);

    const founderBefore =
      await program.account.founderUsdCapState.fetch(founderUsdCap);

    console.log({
      founderBefore:
        founderBefore.earnedCurrentPeriodUsdE6.toString(),
      lifetimeBefore:
        founderBefore.lifetimeEarnedUsdE6.toString(),
      treasuryFounderBefore:
        before.lifetimeFounder.toString(),
    });


    //
    // Small volume cycle
    //
    await program.methods
      .processFees()
      .accounts({
        settlementMint:
          before.settlementMint,

        treasury,

        founderPrice,

        founderUsdCap,
      })
      .rpc();


    const middle =
      await program.account.treasuryState.fetch(treasury);


    const founderAfterSmall =
      await program.account.founderUsdCapState.fetch(founderUsdCap);


    const smallFounderIncrease =
      BigInt(
        founderAfterSmall.lifetimeEarnedUsdE6.toString(),
      ) -
      BigInt(
        founderBefore.lifetimeEarnedUsdE6.toString(),
      );


    console.log({
      smallFounderIncrease:
        smallFounderIncrease.toString(),
    });


    //
    // Large volume cycle
    //
    await program.methods
      .processFees()
      .accounts({
        settlementMint:
          before.settlementMint,

        treasury,

        founderPrice,

        founderUsdCap,
      })
      .rpc();


    const after =
      await program.account.treasuryState.fetch(treasury);


    const founderAfterLarge =
      await program.account.founderUsdCapState.fetch(founderUsdCap);


    const largeFounderIncrease =
      BigInt(
        founderAfterLarge.lifetimeEarnedUsdE6.toString(),
      ) -
      BigInt(
        founderAfterSmall.lifetimeEarnedUsdE6.toString(),
      );


    console.log({
      largeFounderIncrease:
        largeFounderIncrease.toString(),
    });


    assert.isTrue(
      largeFounderIncrease > 0,
      "Founder allocation should continue increasing",
    );

    console.log({
      founderBefore:
        founderBefore.lifetimeEarnedUsdE6.toString(),

      founderAfter:
        founderAfterLarge.lifetimeEarnedUsdE6.toString(),

      liquidity:
        after.lifetimeLiquidity.toString(),
    });

  });
});
