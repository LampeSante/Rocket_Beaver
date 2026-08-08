import * as anchor from "@coral-xyz/anchor";
import { HermesClient } from "@pythnetwork/hermes-client";
import {
  DEFAULT_RECEIVER_PROGRAM_ID,
  DEFAULT_WORMHOLE_PROGRAM_ID,
  PythSolanaReceiver,
} from "@pythnetwork/pyth-solana-receiver";
import {
  Connection,
  Keypair,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import fs from "fs";
import path from "path";

const DEVNET_RPC = "https://api.devnet.solana.com";

const TREASURY_ROUTER_PROGRAM_ID = new PublicKey(
  "5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3",
);

const ORACLE_ADAPTER_PROGRAM_ID = new PublicKey(
  "E5T6hAp6pTE9iZvwMMRaUD7kDC3UkJWiiqi24fJf3sqx",
);

const USDC_USD_FEED_ID =
  "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";

const MAXIMUM_AGE_SECONDS = 300n;
const MAXIMUM_CONFIDENCE_BPS = 100;

type IdlAccount = {
  name: string;
  writable?: boolean;
  signer?: boolean;
};

type IdlInstruction = {
  name: string;
  discriminator: number[];
  accounts: IdlAccount[];
};

type AdapterIdl = {
  address?: string;
  instructions?: IdlInstruction[];
};

function loadKeypair(keypairPath: string): Keypair {
  const bytes = JSON.parse(
    fs.readFileSync(keypairPath, "utf8"),
  ) as number[];

  return Keypair.fromSecretKey(Uint8Array.from(bytes));
}

function normalizeName(name: string): string {
  return name.replace(/_/g, "").toLowerCase();
}

function encodeU64(value: bigint): Buffer {
  const output = Buffer.alloc(8);
  output.writeBigUInt64LE(value);
  return output;
}

function encodeU16(value: number): Buffer {
  const output = Buffer.alloc(2);
  output.writeUInt16LE(value);
  return output;
}

function findInstruction(
  idl: AdapterIdl,
  expectedName: string,
): IdlInstruction {
  const instruction = (idl.instructions ?? []).find(
    ({ name }) =>
      normalizeName(name) === normalizeName(expectedName),
  );

  if (!instruction) {
    throw new Error(
      `Instruction ${expectedName} is missing from the adapter IDL`,
    );
  }

  return instruction;
}

function buildAdapterInstruction(args: {
  adapterIdl: AdapterIdl;
  priceUpdate: PublicKey;
  oracleAdapterAuthority: PublicKey;
  protocolState: PublicKey;
  founderPrice: PublicKey;
}): TransactionInstruction {
  const instruction = findInstruction(
    args.adapterIdl,
    "verify_and_submit_founder_price",
  );

  const accountMap = new Map<string, PublicKey>([
    [
      normalizeName("price_update"),
      args.priceUpdate,
    ],
    [
      normalizeName("oracle_adapter_authority"),
      args.oracleAdapterAuthority,
    ],
    [
      normalizeName("protocol_state"),
      args.protocolState,
    ],
    [
      normalizeName("founder_price"),
      args.founderPrice,
    ],
    [
      normalizeName("treasury_router_program"),
      TREASURY_ROUTER_PROGRAM_ID,
    ],
  ]);

  const keys = instruction.accounts.map((account) => {
    const publicKey = accountMap.get(
      normalizeName(account.name),
    );

    if (!publicKey) {
      throw new Error(
        `No account mapping for adapter account: ${account.name}`,
      );
    }

    return {
      pubkey: publicKey,
      isSigner: account.signer === true,
      isWritable: account.writable === true,
    };
  });

  const feedIdBytes = Buffer.from(
    USDC_USD_FEED_ID,
    "hex",
  );

  if (feedIdBytes.length !== 32) {
    throw new Error("USDC/USD feed ID must be exactly 32 bytes");
  }

  const data = Buffer.concat([
    Buffer.from(instruction.discriminator),
    feedIdBytes,
    encodeU64(MAXIMUM_AGE_SECONDS),
    encodeU16(MAXIMUM_CONFIDENCE_BPS),
  ]);

  return new TransactionInstruction({
    programId: ORACLE_ADAPTER_PROGRAM_ID,
    keys,
    data,
  });
}

async function main(): Promise<void> {
  const walletPath =
    process.env.ANCHOR_WALLET ??
    path.join(
      process.env.HOME ?? "",
      "rocket-beaver-dev/keys/local-deployment.json",
    );

  if (!walletPath || !fs.existsSync(walletPath)) {
    throw new Error(
      `Devnet wallet was not found: ${walletPath}`,
    );
  }

  const payer = loadKeypair(walletPath);
  const wallet = new anchor.Wallet(payer);

  const connection = new Connection(
    process.env.ANCHOR_PROVIDER_URL ?? DEVNET_RPC,
    "confirmed",
  );

  if (!/devnet/i.test(connection.rpcEndpoint)) {
    throw new Error(
      `Refusing live oracle submission against non-Devnet RPC: ${connection.rpcEndpoint}`,
    );
  }

  const provider = new anchor.AnchorProvider(
    connection,
    wallet,
    {
      commitment: "confirmed",
      preflightCommitment: "confirmed",
    },
  );

  const receiver = new PythSolanaReceiver({
    connection,
    wallet,
    receiverProgramId: DEFAULT_RECEIVER_PROGRAM_ID,
    wormholeProgramId: DEFAULT_WORMHOLE_PROGRAM_ID,
  });

  const repositoryRoot = path.resolve(
    __dirname,
    "../..",
  );

  const adapterIdlPath = path.join(
    repositoryRoot,
    "oracle-adapter/target/idl/rbvr_oracle_adapter.json",
  );

  if (!fs.existsSync(adapterIdlPath)) {
    throw new Error(
      `Oracle Adapter IDL was not found: ${adapterIdlPath}`,
    );
  }

  const adapterIdl = JSON.parse(
    fs.readFileSync(adapterIdlPath, "utf8"),
  ) as AdapterIdl;

  const [
    protocolState,
  ] = PublicKey.findProgramAddressSync(
    [Buffer.from("protocol")],
    TREASURY_ROUTER_PROGRAM_ID,
  );

  const [
    founderPrice,
  ] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("founder-price"),
      protocolState.toBuffer(),
    ],
    TREASURY_ROUTER_PROGRAM_ID,
  );

  const [
    oracleAdapterAuthority,
  ] = PublicKey.findProgramAddressSync(
    [Buffer.from("rbvr-oracle-authority")],
    ORACLE_ADAPTER_PROGRAM_ID,
  );

  const founderPriceBefore =
    await connection.getAccountInfo(
      founderPrice,
      "confirmed",
    );

  if (!founderPriceBefore) {
    throw new Error(
      `FounderPriceState does not exist: ${founderPrice.toBase58()}`,
    );
  }

  const hermes = new HermesClient(
    "https://hermes.pyth.network",
  );

  const update = await hermes.getLatestPriceUpdates(
    [`0x${USDC_USD_FEED_ID}`],
    {
      encoding: "base64",
      parsed: true,
    },
  );

  const binaryUpdates = update.binary.data;

  if (!Array.isArray(binaryUpdates) || binaryUpdates.length === 0) {
    throw new Error(
      "Hermes returned no binary USDC/USD update",
    );
  }

  const parsedPrice = update.parsed?.[0];

  if (!parsedPrice) {
    throw new Error(
      "Hermes returned no parsed USDC/USD price",
    );
  }

  const returnedFeedId = parsedPrice.id
    .replace(/^0x/i, "")
    .toLowerCase();

  if (returnedFeedId !== USDC_USD_FEED_ID) {
    throw new Error(
      `Hermes returned the wrong feed: ${returnedFeedId}`,
    );
  }

  if (BigInt(parsedPrice.price.price) <= 0n) {
    throw new Error("Hermes price must be positive");
  }

  const builder = receiver.newTransactionBuilder({
    closeUpdateAccounts: true,
  });

  // Fully verified Receiver path.
  await builder.addPostPriceUpdates(binaryUpdates);

  const priceUpdateAccount =
    builder.getPriceUpdateAccount(
      `0x${USDC_USD_FEED_ID}`,
    );

  await builder.addPriceConsumerInstructions(
    async () => [
      {
        instruction: buildAdapterInstruction({
          adapterIdl,
          priceUpdate: priceUpdateAccount,
          oracleAdapterAuthority,
          protocolState,
          founderPrice,
        }),
        signers: [],
      },
    ],
  );

  const transactions =
    await builder.buildVersionedTransactions({
      computeUnitPriceMicroLamports: 10_000,
      tightComputeBudget: true,
    });

  if (transactions.length === 0) {
    throw new Error(
      "Pyth transaction builder returned no transactions",
    );
  }

  console.log(
    JSON.stringify(
      {
        rpcEndpoint: connection.rpcEndpoint,
        payer: payer.publicKey.toBase58(),
        receiverProgramId:
          DEFAULT_RECEIVER_PROGRAM_ID.toBase58(),
        wormholeProgramId:
          DEFAULT_WORMHOLE_PROGRAM_ID.toBase58(),
        oracleAdapterProgramId:
          ORACLE_ADAPTER_PROGRAM_ID.toBase58(),
        oracleAdapterAuthority:
          oracleAdapterAuthority.toBase58(),
        protocolState: protocolState.toBase58(),
        founderPrice: founderPrice.toBase58(),
        priceUpdateAccount:
          priceUpdateAccount.toBase58(),
        hermes: {
          feedId: returnedFeedId,
          price: parsedPrice.price.price,
          exponent: parsedPrice.price.expo,
          confidence: parsedPrice.price.conf,
          publishTime: parsedPrice.price.publish_time,
          binaryUpdateCount: binaryUpdates.length,
        },
        transactionCount: transactions.length,
      },
      null,
      2,
    ),
  );

  const signatures = await provider.sendAll(
    transactions,
    {
      commitment: "confirmed",
      skipPreflight: false,
    },
  );

  console.log(
    JSON.stringify(
      {
        signatures,
      },
      null,
      2,
    ),
  );
}

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
