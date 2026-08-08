import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { getAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import type { TreasuryRouter } from "../target/types/treasury_router";
import { assert } from "chai";

describe("RBVR Devnet Destination Firewall Live", function () {
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
      [Buffer.from("treasury"), protocolState.toBuffer()],
      program.programId,
    )[0];

  const founderState =
    PublicKey.findProgramAddressSync(
      [Buffer.from("founder-state"), protocolState.toBuffer()],
      program.programId,
    )[0];

  const executionConfig =
    PublicKey.findProgramAddressSync(
      [Buffer.from("execution-config"), protocolState.toBuffer()],
      program.programId,
    )[0];

  it("rejects attacker-controlled founder destination without mutation", async () => {

    const treasuryBefore =
      await program.account.treasuryState.fetch(treasury);


    const founderStateBefore =
      await program.account.founderState.fetch(founderState);


    const treasuryState =
      await program.account.treasuryState.fetch(treasury);


    const attacker =
      Keypair.generate();


    const attackerDestination =
      await anchor.utils.token.associatedAddress({
        mint: treasuryState.settlementMint,
        owner: attacker.publicKey,
      });


    let rejected = false;
    let errorText = "";


    try {

      await program.methods
        .authorizeFounderExecution()
        .accountsPartial({

          protocolState,

          founderState,

          treasury,

          executionConfig,

          settlementMint:
            treasuryState.settlementMint,

          settlementVault:
            treasuryState.settlementVault,

          founderDestination:
            attackerDestination,

          tokenProgram:
            TOKEN_PROGRAM_ID,

        })
        .rpc();

    } catch (e) {

      rejected = true;

      errorText =
        e instanceof Error
          ? e.message
          : String(e);
    }


    assert.isTrue(
      rejected,
      "attacker destination must be rejected",
    );


    const treasuryAfter =
      await program.account.treasuryState.fetch(treasury);


    const founderStateAfter =
      await program.account.founderState.fetch(founderState);


    assert.equal(
      treasuryAfter.pendingFounder.toString(),
      treasuryBefore.pendingFounder.toString(),
      "pending founder changed after rejected attack",
    );


    assert.equal(
      treasuryAfter.releasedFounder.toString(),
      treasuryBefore.releasedFounder.toString(),
      "released founder changed after rejected attack",
    );


    assert.equal(
      founderStateAfter.lifetimeEarned.toString(),
      founderStateBefore.lifetimeEarned.toString(),
      "founder accounting mutated after rejected attack",
    );


    assert.match(
      errorText,
      /Unauthorized|Constraint|Invalid|Account/i,
      "unexpected rejection reason",
    );

  });

});
