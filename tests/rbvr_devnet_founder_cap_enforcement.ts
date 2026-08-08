import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import {
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import type { TreasuryRouter } from "../target/types/treasury_router";
import { assert } from "chai";

describe("RBVR Devnet Founder USD Cap Enforcement", function () {
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

  const founderState =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-state"),
        protocolState.toBuffer(),
      ],
      program.programId,
    )[0];

  const companyState =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("company-state"),
        protocolState.toBuffer(),
      ],
      program.programId,
    )[0];

  const settlementVault =
    new PublicKey(
      "3xouK9KYZrWPyv8qvsDPwRtKy3rXvvsP4cQs7BfoPCa6",
    );


  it("enforces the Founder USD cap and redirects overflow", async () => {

    const capBefore =
      await program.account.founderUsdCapState.fetch(
        founderUsdCap,
      );

    const treasuryBefore =
      await program.account.treasuryState.fetch(
        treasury,
      );


    console.log({
      founderBefore:
        capBefore.lifetimeEarnedUsdE6.toString(),

      liquidityBefore:
        treasuryBefore.lifetimeLiquidity.toString(),
    });


    const source =
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        provider.wallet.payer,
        capBefore.settlementMint,
        provider.wallet.publicKey,
      );


    // Large deposit intended to exceed remaining founder cap
    const amount =
      new anchor.BN(
        500_000_000_000_000,
      );


    await mintTo(
      provider.connection,
      provider.wallet.payer,
      capBefore.settlementMint,
      source.address,
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
          source.address,

        settlementVault,

      })
      .rpc();


    await program.methods
      .processFees()
      .accounts({
        settlementMint:
          capBefore.settlementMint,





        settlementVault,
      })
      .rpc();


    const capAfter =
      await program.account.founderUsdCapState.fetch(
        founderUsdCap,
      );

    const treasuryAfter =
      await program.account.treasuryState.fetch(
        treasury,
      );


    console.log({
      founderAfter:
        capAfter.lifetimeEarnedUsdE6.toString(),

      liquidityAfter:
        treasuryAfter.lifetimeLiquidity.toString(),
    });


    const founderIncrease =
      capAfter.lifetimeEarnedUsdE6.sub(
        capBefore.lifetimeEarnedUsdE6,
      );

    const liquidityIncrease =
      treasuryAfter.lifetimeLiquidity.sub(
        treasuryBefore.lifetimeLiquidity,
      );


    console.log({
      founderIncrease:
        founderIncrease.toString(),

      liquidityIncrease:
        liquidityIncrease.toString(),

      finalFounderUsd:
        capAfter.lifetimeEarnedUsdE6.toString(),
    });


    assert.isTrue(
      capAfter.lifetimeEarnedUsdE6.lte(
        new anchor.BN("3000000000000"),
      ),
      "Founder USD lifetime cap exceeded",
    );


    assert.isTrue(
      liquidityIncrease.gt(new anchor.BN(0)),
      "Founder overflow did not increase liquidity",
    );


    assert.equal(
      capAfter.lifetimeEarnedUsdE6.toString(),
      "3000000000000",
      "Founder cap was not exhausted",
    );
  });
});
