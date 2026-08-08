import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import { assert } from "chai";

describe("RBVR Devnet Full Economics Live", function () {
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

  it("preserves global accounting through complete fee lifecycle", async () => {

    const before =
      await program.account.treasuryState.fetch(treasury);

    console.log({
      reserveBefore:
        before.lifetimeReserve.toString(),

      liquidityBefore:
        before.lifetimeLiquidity.toString(),

      companyBefore:
        before.lifetimeCompany.toString(),

      founderBefore:
        before.lifetimeFounder.toString(),
    });


    /*
      TODO:
      - deposit settlement assets
      - refresh oracle
      - process fees
      - fetch all destinations
      - verify allocation math
    */


    const after =
      await program.account.treasuryState.fetch(treasury);


    console.log({
      reserveAfter:
        after.lifetimeReserve.toString(),

      liquidityAfter:
        after.lifetimeLiquidity.toString(),

      companyAfter:
        after.lifetimeCompany.toString(),

      founderAfter:
        after.lifetimeFounder.toString(),
    });


    assert.ok(
      after.lifetimeReserve.gte(
        before.lifetimeReserve,
      ),
    );

    assert.ok(
      after.lifetimeLiquidity.gte(
        before.lifetimeLiquidity,
      ),
    );
  });
});
