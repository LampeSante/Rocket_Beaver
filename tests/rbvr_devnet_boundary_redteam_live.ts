import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import {
  getAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import type { TreasuryRouter } from "../target/types/treasury_router";
import { assert } from "chai";

describe("RBVR Devnet Boundary Red Team Live", function () {
  this.timeout(1_000_000);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program =
    anchor.workspace.TreasuryRouter as Program<TreasuryRouter>;

  const PROGRAM_ID = new PublicKey(
    "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3"
  );

  const protocolStatePda =
    PublicKey.findProgramAddressSync(
      [Buffer.from("protocol")],
      PROGRAM_ID
    )[0];

  const treasuryStatePda =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("treasury"),
        protocolStatePda.toBuffer(),
      ],
      PROGRAM_ID
    )[0];

  const settlementVaultPda =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("treasury-vault"),
        treasuryStatePda.toBuffer(),
      ],
      PROGRAM_ID
    )[0];


  async function treasury() {
    return program.account.treasuryState.fetch(
      treasuryStatePda
    );
  }


  it("rejects zero-value deposits without mutation", async () => {

    const before =
      await treasury();

    let rejected = false;

    try {
      await program.methods
        .depositSettlement(
          new anchor.BN(0)
        )
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,
          settlementVault: settlementVaultPda,
        })
        .rpc();

    } catch {
      rejected = true;
    }

    assert.equal(
      rejected,
      true,
      "zero deposit must fail"
    );

    const after =
      await treasury();

    assert.equal(
      after.totalFeesReceived.toString(),
      before.totalFeesReceived.toString()
    );

  });


  it("rejects fee replay without mutation", async () => {

    const before =
      await treasury();

    let rejected = false;

    try {

      await program.methods
        .processFees()
        .accountsPartial({
          protocolState: protocolStatePda,
          treasury: treasuryStatePda,
          settlementVault: settlementVaultPda,
        })
        .rpc();

    } catch {
      rejected = true;
    }


    assert.equal(
      rejected,
      true,
      "fee replay must fail"
    );


    const after =
      await treasury();


    assert.equal(
      after.totalFeesAllocated.toString(),
      before.totalFeesAllocated.toString()
    );

  });


  it("preserves treasury invariants after attacks", async () => {

    const state =
      await treasury();


    const total =
      state.lifetimeReserve
        .add(state.lifetimeBuybackBurn)
        .add(state.lifetimeLiquidity)
        .add(state.lifetimeCompany)
        .add(state.lifetimeFounder);


    assert.equal(
      total.toString(),
      state.totalFeesAllocated.toString()
    );

  });

});
