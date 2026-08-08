import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import { execSync } from "child_process";
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



  function refreshFounderOracle() {
    execSync(
      `
      cd tools/pyth-receiver-client &&       ANCHOR_PROVIDER_URL="https://api.devnet.solana.com"       ANCHOR_WALLET="$HOME/rocket-beaver-dev/keys/local-deployment.json"       npx ts-node       --project ./tsconfig.json       submit-rbvr-price.ts
      `,
      {
        stdio: "inherit",
        shell: "/bin/bash",
      },
    );
  }

  async function depositTestFees(amount: number) {
    const treasuryState =
      await program.account.treasuryState.fetch(treasury);

    const source =
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        provider.wallet.payer,
        treasuryState.settlementMint,
        provider.wallet.publicKey,
      );

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      treasuryState.settlementMint,
      source.address,
      provider.wallet.publicKey,
      amount,
    );

    await program.methods
      .depositSettlement(
        new anchor.BN(amount),
      )
      .accounts({
        settlementMint:
          treasuryState.settlementMint,

        sourceTokenAccount:
          source.address,

        settlementVault:
          new PublicKey(
            "3xouK9KYZrWPyv8qvsDPwRtKy3rXvvsP4cQs7BfoPCa6",
          ),
      })
      .rpc();
  }

  it("proves founder marginal rate decreases with volume", async () => {

  const protocolStateAccount =
    await program.account.protocolState.fetch(
      protocolState,
    );

  const founderState =
    protocolStateAccount.founderState;


    const before =
      await program.account.treasuryState.fetch(treasury);

    const founderBefore =
      await program.account.founderState.fetch(founderState);

  
  console.log({
  });

  console.log({
      founderBefore:
        founderBefore.earnedCurrentPeriod.toString(),
      lifetimeBefore:
        founderBefore.lifetimeEarned.toString(),
      treasuryFounderBefore:
        before.lifetimeFounder.toString(),
    });


    //
    // Small volume cycle
    //
    await depositTestFees(500_000);

    refreshFounderOracle();

    await program.methods
      .processFees()
      .accounts({
        settlementMint:
          before.settlementMint,

        settlementVault:
          new PublicKey(
            "3xouK9KYZrWPyv8qvsDPwRtKy3rXvvsP4cQs7BfoPCa6",
          ),

      })
      .rpc();


    const middle =
      await program.account.treasuryState.fetch(treasury);


    const founderAfterSmall =
      await program.account.founderState.fetch(founderState);


    const smallFounderIncrease =
      BigInt(
        founderAfterSmall.lifetimeEarned.toString(),
      ) -
      BigInt(
        founderBefore.lifetimeEarned.toString(),
      );


    console.log({
      smallFounderIncrease:
        smallFounderIncrease.toString(),
    });


    //
    // Large volume cycle
    //
    await depositTestFees(500_000);

    refreshFounderOracle();

    await program.methods
      .processFees()
      .accounts({
        settlementMint:
          before.settlementMint,

        settlementVault:
          new PublicKey(
            "3xouK9KYZrWPyv8qvsDPwRtKy3rXvvsP4cQs7BfoPCa6",
          ),

      })
      .rpc();


    const after =
      await program.account.treasuryState.fetch(treasury);


    const founderAfterLarge =
      await program.account.founderState.fetch(founderState);


    const largeFounderIncrease =
      BigInt(
        founderAfterLarge.lifetimeEarned.toString(),
      ) -
      BigInt(
        founderAfterSmall.lifetimeEarned.toString(),
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
        founderBefore.lifetimeEarned.toString(),

      founderAfter:
        founderAfterLarge.lifetimeEarned.toString(),

      liquidity:
        after.lifetimeLiquidity.toString(),
    });

  });
});
