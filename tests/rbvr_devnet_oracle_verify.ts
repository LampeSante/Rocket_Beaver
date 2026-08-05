import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
import type { TreasuryRouter } from "../target/types/treasury_router";
import { assert } from "chai";

describe("RBVR Devnet Oracle Verification", function () {
  this.timeout(1_000_000);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const TREASURY_PROGRAM_ID = new PublicKey(
    "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3",
  );

  const ORACLE_ADAPTER_PROGRAM_ID = new PublicKey(
    "E5T6hAp6pTE9iZvwMMRaUD7kDC3UkJWiiqi24fJf3sqx",
  );

  const treasuryRouter =
    anchor.workspace.TreasuryRouter as Program<TreasuryRouter>;

  const [protocolStatePda] =
    PublicKey.findProgramAddressSync(
      [Buffer.from("protocol")],
      TREASURY_PROGRAM_ID,
    );

  const [treasuryStatePda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("treasury"),
        protocolStatePda.toBuffer(),
      ],
      TREASURY_PROGRAM_ID,
    );

  const [founderUsdCapPda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-usd-cap"),
        protocolStatePda.toBuffer(),
      ],
      TREASURY_PROGRAM_ID,
    );

  const [founderPricePda] =
    PublicKey.findProgramAddressSync(
      [
        Buffer.from("founder-price"),
        protocolStatePda.toBuffer(),
      ],
      TREASURY_PROGRAM_ID,
    );

  const [oracleAdapterAuthority, oracleAdapterAuthorityBump] =
    PublicKey.findProgramAddressSync(
      [Buffer.from("rbvr-oracle-authority")],
      ORACLE_ADAPTER_PROGRAM_ID,
    );

  before("confirms the target is Devnet", () => {
    const rpcEndpoint = provider.connection.rpcEndpoint;

    assert.match(
      rpcEndpoint,
      /devnet/i,
      `Refusing oracle verification against non-Devnet RPC: ${rpcEndpoint}`,
    );
  });

  it("confirms Treasury Router is deployed and executable", async () => {
    const accountInfo =
      await provider.connection.getAccountInfo(
        TREASURY_PROGRAM_ID,
        "confirmed",
      );

    assert.isNotNull(
      accountInfo,
      `Treasury Router does not exist on Devnet: ${TREASURY_PROGRAM_ID.toBase58()}`,
    );

    assert.isTrue(
      accountInfo!.executable,
      "Treasury Router account must be executable",
    );
  });

  it("confirms Oracle Adapter is deployed and executable", async () => {
    const accountInfo =
      await provider.connection.getAccountInfo(
        ORACLE_ADAPTER_PROGRAM_ID,
        "confirmed",
      );

    assert.isNotNull(
      accountInfo,
      `Oracle Adapter does not exist on Devnet: ${ORACLE_ADAPTER_PROGRAM_ID.toBase58()}`,
    );

    assert.isTrue(
      accountInfo!.executable,
      "Oracle Adapter account must be executable",
    );
  });

  it("confirms the canonical ProtocolState exists", async () => {
    const accountInfo =
      await provider.connection.getAccountInfo(
        protocolStatePda,
        "confirmed",
      );

    assert.isNotNull(
      accountInfo,
      `ProtocolState does not exist: ${protocolStatePda.toBase58()}`,
    );

    assert.equal(
      accountInfo!.owner.toBase58(),
      TREASURY_PROGRAM_ID.toBase58(),
      "ProtocolState must be owned by Treasury Router",
    );

    const state =
      await treasuryRouter.account.protocolState.fetch(
        protocolStatePda,
      );

    console.log(
      JSON.stringify(
        {
          protocolStatePda: protocolStatePda.toBase58(),
          authority: state.authority.toBase58(),
          version: state.version,
          bump: state.bump,
        },
        null,
        2,
      ),
    );
  });

  it("inspects the Devnet Founder USD oracle accounts", async () => {
    const [
      treasuryInfo,
      founderUsdCapInfo,
      founderPriceInfo,
    ] = await Promise.all([
      provider.connection.getAccountInfo(
        treasuryStatePda,
        "confirmed",
      ),
      provider.connection.getAccountInfo(
        founderUsdCapPda,
        "confirmed",
      ),
      provider.connection.getAccountInfo(
        founderPricePda,
        "confirmed",
      ),
    ]);

    for (const [name, address, info] of [
      ["TreasuryState", treasuryStatePda, treasuryInfo],
      ["FounderUsdCapState", founderUsdCapPda, founderUsdCapInfo],
      ["FounderPriceState", founderPricePda, founderPriceInfo],
    ] as const) {
      if (info !== null) {
        assert.equal(
          info.owner.toBase58(),
          TREASURY_PROGRAM_ID.toBase58(),
          `${name} must be owned by Treasury Router`,
        );
      }

      console.log(
        JSON.stringify(
          {
            account: name,
            address: address.toBase58(),
            exists: info !== null,
            owner: info?.owner.toBase58() ?? null,
            dataLength: info?.data.length ?? null,
          },
          null,
          2,
        ),
      );
    }

    if (founderUsdCapInfo !== null) {
      const cap =
        await treasuryRouter.account.founderUsdCapState.fetch(
          founderUsdCapPda,
        );

      console.log(
        JSON.stringify(
          {
            founderUsdCap: {
              annualCapUsdE6: cap.annualCapUsdE6.toString(),
              earnedCurrentPeriodUsdE6:
                cap.earnedCurrentPeriodUsdE6.toString(),
              lifetimeEarnedUsdE6:
                cap.lifetimeEarnedUsdE6.toString(),
              periodDuration: cap.periodDuration.toString(),
              settlementMint:
                cap.settlementMint.toBase58(),
              enabled: cap.enabled,
            },
          },
          null,
          2,
        ),
      );
    }

    if (founderPriceInfo !== null) {
      const price =
        await treasuryRouter.account.founderPriceState.fetch(
          founderPricePda,
        );

      console.log(
        JSON.stringify(
          {
            founderPrice: {
              oracleAdapterAuthority:
                price.oracleAdapterAuthority.toBase58(),
              expectedOracleAdapterAuthority:
                oracleAdapterAuthority.toBase58(),
              price: price.price.toString(),
              exponent: price.exponent,
              confidence: price.confidence.toString(),
              publishTime: price.publishTime.toString(),
              sequence: price.sequence.toString(),
              enabled: price.enabled,
            },
          },
          null,
          2,
        ),
      );

      assert.equal(
        price.oracleAdapterAuthority.toBase58(),
        oracleAdapterAuthority.toBase58(),
        "FounderPriceState must be locked to the canonical adapter PDA",
      );
    }
  });

  it("prints the verified Devnet program addresses", () => {
    console.log(
      JSON.stringify(
        {
          rpcEndpoint: provider.connection.rpcEndpoint,
          treasuryRouterProgramId:
            TREASURY_PROGRAM_ID.toBase58(),
          oracleAdapterProgramId:
            ORACLE_ADAPTER_PROGRAM_ID.toBase58(),
          oracleAdapterAuthority:
            oracleAdapterAuthority.toBase58(),
          oracleAdapterAuthorityBump,
          protocolStatePda:
            protocolStatePda.toBase58(),
          treasuryStatePda:
            treasuryStatePda.toBase58(),
          founderUsdCapPda:
            founderUsdCapPda.toBase58(),
          founderPricePda:
            founderPricePda.toBase58(),
        },
        null,
        2,
      ),
    );
  });
});
