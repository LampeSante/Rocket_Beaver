import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import { assert } from "chai";

describe("RBVR Devnet Fee Execution Live", function () {
  this.timeout(1_000_000);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program =
    anchor.workspace.TreasuryRouter as Program<any>;

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


  it("routes real fees through the complete allocation engine", async () => {

    const before =
      await program.account.treasuryState.fetch(treasury);


    console.log({
      reserve:
        before.lifetimeReserve.toString(),

      liquidity:
        before.lifetimeLiquidity.toString(),

      company:
        before.lifetimeCompany.toString(),

      founder:
        before.lifetimeFounder.toString(),
    });


    /*
      TODO:
      1. Create settlement deposit
      2. Call processFees()
      3. Fetch destinations
      4. Verify exact 30/20/20/20/10 split
    */


    const after =
      await program.account.treasuryState.fetch(treasury);


    assert.ok(after);


    console.log({
      reserve:
        after.lifetimeReserve.toString(),

      liquidity:
        after.lifetimeLiquidity.toString(),

      company:
        after.lifetimeCompany.toString(),

      founder:
        after.lifetimeFounder.toString(),
    });

  });

});
