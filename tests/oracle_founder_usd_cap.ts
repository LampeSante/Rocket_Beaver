import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";

describe("Oracle → Founder USD Cap integration", () => {
  const provider = anchor.AnchorProvider.env();

  anchor.setProvider(provider);

  const treasuryRouter =
    anchor.workspace.TreasuryRouter as Program;

  const oracleAdapter =
    anchor.workspace.RbvrOracleAdapter as Program;

  it("loads both local programs", async () => {
    assert.isDefined(treasuryRouter);
    assert.isDefined(oracleAdapter);

    assert.equal(
      treasuryRouter.programId.toBase58(),
      "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3",
    );

    assert.equal(
      oracleAdapter.programId.toBase58(),
      "E5T6hAp6pTE9iZvwMMRaUD7kDC3UkJWiiqi24fJf3sqx",
    );
  });

  it.skip("initializes protocol state", async () => {});

  it.skip("initializes Founder USD Cap", async () => {});

  it.skip("initializes Founder Price", async () => {});

  it.skip(
    "submits a verified oracle price through the adapter",
    async () => {},
  );

  it.skip(
    "processes fees using the stored oracle price",
    async () => {},
  );

  it.skip(
    "enforces the US$3M annual Founder cap",
    async () => {},
  );

  it.skip(
    "redirects Founder overflow to Liquidity Growth",
    async () => {},
  );

  it.skip("preserves accounting invariants", async () => {});
});
