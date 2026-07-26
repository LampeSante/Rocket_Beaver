import fs from "node:fs";

import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

import {
  DynamicBondingCurveClient,
  buildCurveWithMarketCap,
  validateConfigParameters,
  TokenType,
  TokenDecimal,
  TokenAuthorityOption,
  MigrationOption,
  MigrationFeeOption,
  ActivationType,
  CollectFeeMode,
  BaseFeeMode,
  MigratedCollectFeeMode,
  DammV2DynamicFeeMode,
} from "@meteora-ag/dynamic-bonding-curve-sdk";

const RPC_URL =
  process.env.SOLANA_RPC_URL || "https://api.devnet.solana.com";

const SEND = process.env.RBVR_SEND === "YES";

const INITIAL_MARKET_CAP = 10_000;
const MIGRATION_MARKET_CAP = 36_281.179138321986;

const GENESIS = 1_000_000_000;
const OUTSIDE_DBC = 390_000_000;

function loadKeypair(path) {
  if (!path) {
    throw new Error("Missing required keypair path.");
  }

  const raw = JSON.parse(fs.readFileSync(path, "utf8"));

  return Keypair.fromSecretKey(Uint8Array.from(raw));
}

function requirePublicKey(name) {
  const value = process.env[name];

  if (!value) {
    throw new Error(`Missing environment variable: ${name}`);
  }

  return new PublicKey(value);
}

console.log("============================================================");
console.log("ROCKET BEAVER — METEORA DBC DEVNET CONFIG V8");
console.log("============================================================");
console.log();

console.log("RPC:", RPC_URL);
console.log("Mode:", SEND ? "SEND TO DEVNET" : "SIMULATION ONLY");
console.log();

if (!RPC_URL.toLowerCase().includes("devnet")) {
  throw new Error(
    "SAFETY BLOCK: V8 only permits an RPC URL containing 'devnet'."
  );
}

const payerPath = process.env.RBVR_DEVNET_PAYER;

if (!payerPath) {
  throw new Error(
    "Set RBVR_DEVNET_PAYER to a dedicated Devnet-only keypair."
  );
}

const payer = loadKeypair(payerPath);

const feeClaimer = requirePublicKey("RBVR_DEVNET_FEE_CLAIMER");
const leftoverReceiver = requirePublicKey(
  "RBVR_DEVNET_LEFTOVER_RECEIVER"
);
const quoteMint = requirePublicKey("RBVR_DEVNET_QUOTE_MINT");

console.log("Payer:              ", payer.publicKey.toBase58());
console.log("Fee claimer:        ", feeClaimer.toBase58());
console.log("Leftover receiver:  ", leftoverReceiver.toBase58());
console.log("Quote mint:          ", quoteMint.toBase58());
console.log();

if (leftoverReceiver.equals(PublicKey.default)) {
  throw new Error("Leftover receiver cannot be the default public key.");
}

const connection = new Connection(RPC_URL, "confirmed");

const version = await connection.getVersion();

console.log("RPC connection:      PASS");
console.log("Solana core:", version["solana-core"]);
console.log();

const balance = await connection.getBalance(payer.publicKey);

console.log(
  "Payer balance:",
  `${balance / 1_000_000_000} SOL`
);

if (balance === 0) {
  throw new Error(
    "Devnet payer has zero SOL. Fund it before continuing."
  );
}

const params = {
  token: {
    tokenType: TokenType.SPLToken,
    tokenBaseDecimal: TokenDecimal.NINE,
    tokenQuoteDecimal: TokenDecimal.NINE,
    tokenAuthorityOption: TokenAuthorityOption.Immutable,

    totalTokenSupply: GENESIS,
    leftover: OUTSIDE_DBC,
  },

  fee: {
    baseFeeParams: {
      baseFeeMode: BaseFeeMode.FeeSchedulerLinear,

      feeSchedulerParam: {
        startingFeeBps: 100,
        endingFeeBps: 100,
        numberOfPeriod: 0,
        totalDuration: 0,
      },
    },

    dynamicFeeEnabled: false,

    collectFeeMode: CollectFeeMode.QuoteToken,

    creatorTradingFeePercentage: 100,

    poolCreationFee: 0,

    enableFirstSwapWithMinFee: false,
  },

  migration: {
    migrationOption: MigrationOption.MET_DAMM_V2,

    migrationFeeOption: MigrationFeeOption.FixedBps25,

    migrationFee: {
      feePercentage: 0,
      creatorFeePercentage: 0,
    },

    migratedPoolFee: {
      collectFeeMode: MigratedCollectFeeMode.QuoteToken,

      dynamicFee: DammV2DynamicFeeMode.Disabled,

      poolFeeBps: 10,
    },
  },

  liquidityDistribution: {
    partnerPermanentLockedLiquidityPercentage: 0,
    partnerLiquidityPercentage: 0,

    creatorPermanentLockedLiquidityPercentage: 100,
    creatorLiquidityPercentage: 0,
  },

  lockedVesting: {
    totalLockedVestingAmount: 0,
    numberOfVestingPeriod: 0,
    cliffUnlockAmount: 0,
    totalVestingDuration: 0,
    cliffDurationFromMigrationTime: 0,
  },

  activationType: ActivationType.Timestamp,

  initialMarketCap: INITIAL_MARKET_CAP,

  migrationMarketCap: MIGRATION_MARKET_CAP,
};

console.log("Building canonical V7-derived config...");

const config = buildCurveWithMarketCap(params);

console.log("Config build:         PASS");

validateConfigParameters({
  ...config,
  leftoverReceiver,
});

console.log("Offline validation:   PASS");
console.log();

const client =
  DynamicBondingCurveClient.create(connection, "confirmed");

console.log("DBC client:           PASS");
console.log();

console.log("Building createConfig transaction...");

/*
 * Meteora createConfig requires the config account itself
 * as a writable signer.
 *
 * Generate a fresh disposable Devnet config keypair for this run.
 */
const configKeypair = Keypair.generate();

console.log(
  "DBC config account:    ",
  configKeypair.publicKey.toBase58()
);

const tx = await client.partner.createConfig({
  ...config,

  config: configKeypair.publicKey,
  feeClaimer,
  leftoverReceiver,
  quoteMint,
  payer: payer.publicKey,
});

console.log("Transaction build:    PASS");
console.log();

const latest = await connection.getLatestBlockhash("confirmed");

tx.feePayer = payer.publicKey;
tx.recentBlockhash = latest.blockhash;

tx.partialSign(payer, configKeypair);

console.log("Simulating transaction...");

const simulation = await connection.simulateTransaction(tx);

if (simulation.value.err) {
  console.error();
  console.error("SIMULATION: FAIL");
  console.error(
    JSON.stringify(simulation.value.err, null, 2)
  );

  if (simulation.value.logs) {
    console.error();
    console.error("PROGRAM LOGS:");

    for (const line of simulation.value.logs) {
      console.error(line);
    }
  }

  console.error();
  console.error("MAINNET: BLOCKED");

  process.exit(1);
}

console.log("SIMULATION:           PASS");

if (simulation.value.unitsConsumed !== undefined) {
  console.log(
    "Compute units:",
    simulation.value.unitsConsumed
  );
}

console.log();

if (!SEND) {
  console.log("============================================================");
  console.log("RESULT: SIMULATION PASS");
  console.log("============================================================");
  console.log();
  console.log("No transaction was sent.");
  console.log();
  console.log(
    "To explicitly send after reviewing the simulation:"
  );
  console.log();
  console.log(
    "RBVR_SEND=YES node scripts/meteora/devnet/create-dbc-config-v10.mjs"
  );
  console.log();
  console.log("MAINNET: BLOCKED");

  process.exit(0);
}

console.log("Sending transaction to Devnet...");

const signature = await sendAndConfirmTransaction(
  connection,
  tx,
  [payer, configKeypair],
  {
    commitment: "confirmed",
  }
);

console.log();
console.log("DEVNET TRANSACTION:   PASS");
console.log("Signature:", signature);

console.log();
console.log("============================================================");
console.log("RESULT: DEVNET CONFIG CREATION PASS");
console.log("============================================================");
console.log();
console.log("MAINNET: BLOCKED");
