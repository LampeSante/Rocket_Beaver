import * as anchor from "@anchor-lang/core";
import { assert } from "chai";
import { PublicKey } from "@solana/web3.js";
import { readFileSync } from "node:fs";

const idl = JSON.parse(
  readFileSync("target/idl/treasury_router.json", "utf8"),
) as anchor.Idl;

const PROGRAM_ID = new PublicKey(
  "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3",
);

const [EXECUTION_CONFIG_PDA] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("execution-config"),
    new PublicKey("6gm3ErHmKPvtEncQo9UVVM73R8BJKDugtevwAzZFSmn4").toBuffer(),
  ],
  PROGRAM_ID,
);

/*
 * These are the locked PDAs created during the successful first Devnet run.
 * They are read-only verification targets. This suite never initializes them.
 */
const LOCKED_ACCOUNTS = [
  {
    label: "Execution Configuration",
    address: EXECUTION_CONFIG_PDA.toBase58(),
  },
  {
    label: "Protocol State",
    address: "6gm3ErHmKPvtEncQo9UVVM73R8BJKDugtevwAzZFSmn4",
  },
  {
    label: "Protocol Configuration",
    address: "45ykMagNoeSXhEDWmBHPeR9yHvB6CTju2EmrnwQQ8NVG",
  },
  {
    label: "Settlement Treasury",
    address: "4JNbTpsaVk2yedE8stQxNZboNwJzLj3cfxgsGvZdn9iW",
  },
  {
    label: "Founder Compensation Controls",
    address: "G6FcSoxrM68e21uYz9AWghC4BRnL13WU7FgxwpNmNVWh",
  },
  {
    label: "Company Allocation Controls",
    address: "7irjZBduNKmnNsMPSDXZyJR2AyFT11iVMfCcBfTDWrBn",
  },
  {
    label: "Reserve Policy",
    address: "GgYfq8aFwgp8H57zgCZn4qrikLs3yUEBapjUndrcvir4",
  },
] as const;

function printable(value: unknown): unknown {
  if (typeof value === "bigint") {
    return value.toString();
  }

  if (value instanceof PublicKey) {
    return value.toBase58();
  }

  if (anchor.BN.isBN(value)) {
    return value.toString();
  }

  if (Buffer.isBuffer(value)) {
    return value.toString("hex");
  }

  if (Array.isArray(value)) {
    return value.map(printable);
  }

  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, printable(item)]),
    );
  }

  return value;
}

function identifyAccountType(data: Buffer): string | null {
  const idlAccounts = (idl as any).accounts ?? [];

  for (const account of idlAccounts) {
    const discriminator = account.discriminator;

    if (
      Array.isArray(discriminator) &&
      discriminator.length === 8 &&
      data.subarray(0, 8).equals(Buffer.from(discriminator))
    ) {
      return account.name;
    }
  }

  return null;
}

describe("RBVR Treasury Router — persistent Devnet verification", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const coder = new anchor.BorshAccountsCoder(idl as anchor.Idl);

  before("confirms the target is Devnet", async () => {
    const rpcEndpoint = connection.rpcEndpoint;

    assert.match(
      rpcEndpoint,
      /devnet/i,
      `Refusing live-state verification against non-Devnet RPC: ${rpcEndpoint}`,
    );

    console.log(`RPC: ${rpcEndpoint}`);
    console.log(`Verifier wallet: ${provider.wallet.publicKey.toBase58()}`);
  });

  it("confirms the deployed program is executable and owned by an upgradeable loader", async () => {
    const programInfo = await connection.getAccountInfo(
      PROGRAM_ID,
      "confirmed",
    );

    assert.isNotNull(programInfo, "deployed program account does not exist");
    assert.isTrue(
      programInfo!.executable,
      "deployed program account is not executable",
    );

    assert.equal(
      programInfo!.owner.toBase58(),
      "BPFLoaderUpgradeab1e11111111111111111111111",
      "program is not owned by the expected upgradeable loader",
    );

    console.log(`Program: ${PROGRAM_ID.toBase58()}`);
    console.log(`Program data length: ${programInfo!.data.length} bytes`);
  });

  it("confirms every locked protocol PDA exists and is program-owned", async () => {
    const addresses = LOCKED_ACCOUNTS.map(
      ({ address }) => new PublicKey(address),
    );

    const accountInfos = await connection.getMultipleAccountsInfo(
      addresses,
      "confirmed",
    );

    for (let index = 0; index < LOCKED_ACCOUNTS.length; index += 1) {
      const expected = LOCKED_ACCOUNTS[index];
      const info = accountInfos[index];

      assert.isNotNull(
        info,
        `${expected.label} is missing at ${expected.address}`,
      );

      assert.equal(
        info!.owner.toBase58(),
        PROGRAM_ID.toBase58(),
        `${expected.label} is not owned by the RBVR program`,
      );

      assert.isAbove(
        info!.data.length,
        8,
        `${expected.label} does not contain a complete Anchor account`,
      );

      const accountType = identifyAccountType(info!.data);

      assert.isNotNull(
        accountType,
        `${expected.label} has an unknown Anchor discriminator`,
      );

      console.log(
        `${expected.label}: ${expected.address} | ${accountType} | ${info!.data.length} bytes`,
      );
    }
  });

  for (const expected of LOCKED_ACCOUNTS) {
    it(`decodes ${expected.label}`, async () => {
      const address = new PublicKey(expected.address);
      const info = await connection.getAccountInfo(address, "confirmed");

      assert.isNotNull(info, `${expected.label} is missing`);

      const accountType = identifyAccountType(info!.data);

      assert.isNotNull(
        accountType,
        `${expected.label} discriminator is not present in the current IDL`,
      );

      const decoded = coder.decode(accountType!, info!.data);

      assert.isObject(
        decoded,
        `${expected.label} could not be decoded using ${accountType}`,
      );

      console.log(`\n${expected.label}`);
      console.log(
        JSON.stringify(
          {
            address: expected.address,
            accountType,
            state: printable(decoded),
          },
          null,
          2,
        ),
      );
    });
  }

  it("confirms all locked PDA addresses are unique", () => {
    const addresses = LOCKED_ACCOUNTS.map(({ address }) => address);
    const uniqueAddresses = new Set(addresses);

    assert.equal(
      uniqueAddresses.size,
      addresses.length,
      "two locked protocol roles use the same PDA",
    );
  });

  it("confirms the verifier wallet still has Devnet transaction capacity", async () => {
    const balance = await connection.getBalance(
      provider.wallet.publicKey,
      "confirmed",
    );

    assert.isAbove(
      balance,
      50_000_000,
      "verifier wallet has less than 0.05 SOL",
    );

    console.log(
      `Verifier balance: ${balance / anchor.web3.LAMPORTS_PER_SOL} SOL`,
    );
  });
});
