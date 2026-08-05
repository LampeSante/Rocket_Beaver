import * as anchor from "@anchor-lang/core";
import { PublicKey } from "@solana/web3.js";
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

  it("prints the verified Devnet program addresses", () => {
    console.log(
      JSON.stringify(
        {
          rpcEndpoint: provider.connection.rpcEndpoint,
          treasuryRouterProgramId:
            TREASURY_PROGRAM_ID.toBase58(),
          oracleAdapterProgramId:
            ORACLE_ADAPTER_PROGRAM_ID.toBase58(),
        },
        null,
        2,
      ),
    );
  });
});
