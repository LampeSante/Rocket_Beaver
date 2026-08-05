import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import {
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import type { TreasuryRouter } from "../target/types/treasury_router";
import { assert } from "chai";

describe("RBVR Devnet Process Fees With Live Oracle", function () {
  this.timeout(1_000_000);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program =
    anchor.workspace.TreasuryRouter as Program<TreasuryRouter>;

  const protocolState =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("protocol"),
      ],
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

  const founderState =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-state"),
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

  const founderPrice =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-price"),
        protocolState.toBuffer(),
      ],
      program.programId,
    )[0];

  const settlementVault =
    new PublicKey(
      "3xouK9KYZrWPyv8qvsDPwRtKy3rXvvsP4cQs7BfoPCa6",
    );


  it("processes fees using the live Founder oracle", async () => {

    const capBefore =
      await program.account.founderUsdCapState.fetch(
        founderUsdCap,
      );

    const treasuryBefore =
      await program.account.treasuryState.fetch(
        treasury,
      );


    console.log({
      beforeReceived:
        treasuryBefore.totalFeesReceived.toString(),

      beforeAllocated:
        treasuryBefore.totalFeesAllocated.toString(),

      founderLifetime:
        capBefore.lifetimeEarnedUsdE6.toString(),
    });


    const sourceTokenAccount =
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        provider.wallet.payer,
        capBefore.settlementMint,
        provider.wallet.publicKey,
      );


    const amount =
      new anchor.BN(
        100_000_000_000,
      );


    await mintTo(
      provider.connection,
      provider.wallet.payer,
      capBefore.settlementMint,
      sourceTokenAccount.address,
      provider.wallet.payer,
      BigInt(amount.toString()),
    );


    await program.methods
      .depositSettlement(
        amount,
      )
      .accounts({
        settlementMint:
          capBefore.settlementMint,

        sourceTokenAccount:
          sourceTokenAccount.address,

        settlementVault,

        authority:
          provider.wallet.publicKey,
      })
      .rpc();


    const treasuryDeposited =
      await program.account.treasuryState.fetch(
        treasury,
      );


    console.log({
      afterDepositReceived:
        treasuryDeposited.totalFeesReceived.toString(),

      afterDepositAllocated:
        treasuryDeposited.totalFeesAllocated.toString(),
    });


    await program.methods
      .processFees()
      .accounts({
        settlementMint:
          capBefore.settlementMint,

        founderState,

        founderUsdCap,

        founderPrice,

        companyState:
          PublicKey.findProgramAddressSync(
            [
              Buffer.from("company-state"),
              protocolState.toBuffer(),
            ],
            program.programId,
          )[0],

        settlementVault,
      })
      .rpc();


    const treasuryAfter =
      await program.account.treasuryState.fetch(
        treasury,
      );

    const capAfter =
      await program.account.founderUsdCapState.fetch(
        founderUsdCap,
      );


    console.log({
      finalReceived:
        treasuryAfter.totalFeesReceived.toString(),

      finalAllocated:
        treasuryAfter.totalFeesAllocated.toString(),

      founderLifetime:
        capAfter.lifetimeEarnedUsdE6.toString(),
    });


    assert.isTrue(
      treasuryAfter.totalFeesAllocated.gt(
        treasuryDeposited.totalFeesAllocated,
      ),
    );
  });
});
