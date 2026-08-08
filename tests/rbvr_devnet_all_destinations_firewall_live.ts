import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import type { TreasuryRouter } from "../target/types/treasury_router";
import { assert } from "chai";

describe("RBVR Devnet All Destinations Firewall Live", function () {
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

  const executionConfig =
    PublicKey.findProgramAddressSync(
      [Buffer.from("execution-config"), protocolState.toBuffer()],
      program.programId,
    )[0];

  const founderState =
    PublicKey.findProgramAddressSync(
      [Buffer.from("founder-state"), protocolState.toBuffer()],
      program.programId,
    )[0];

  const companyState =
    PublicKey.findProgramAddressSync(
      [Buffer.from("company-state"), protocolState.toBuffer()],
      program.programId,
    )[0];


  async function assertNoFounderMutation(
    before: any,
  ) {
    const after =
      await program.account.treasuryState.fetch(
        treasury,
      );

    assert.equal(
      after.pendingFounder.toString(),
      before.pendingFounder.toString(),
    );

    assert.equal(
      after.releasedFounder.toString(),
      before.releasedFounder.toString(),
    );
  }


  it("rejects attacker-controlled destination substitutions", async () => {

    const treasuryBefore =
      await program.account.treasuryState.fetch(
        treasury,
      );


    const attacker =
      Keypair.generate();


    const fakeDestination =
      Keypair.generate().publicKey;


    const attacks = [

      {
        name: "founder",
        method: "authorizeFounderExecution",
        accounts: {
          protocolState,
          founderState,
          treasury,
          executionConfig,
          settlementMint:
            treasuryBefore.settlementMint,
          settlementVault:
            treasuryBefore.settlementVault,
          founderDestination:
            fakeDestination,
          tokenProgram:
            TOKEN_PROGRAM_ID,
        },
      },

    ];


    for (const attack of attacks) {

      let rejected = false;

      try {

        await (program.methods as any)
          [attack.method]()
          .accountsPartial(
            attack.accounts,
          )
          .rpc();

      } catch (_) {

        rejected = true;

      }


      assert.isTrue(
        rejected,
        `${attack.name} destination substitution was accepted`,
      );

    }


    await assertNoFounderMutation(
      treasuryBefore,
    );

  });

});
