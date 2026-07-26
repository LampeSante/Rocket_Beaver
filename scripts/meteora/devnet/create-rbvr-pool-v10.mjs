import fs from "node:fs";
import path from "node:path";

import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

import {
  DynamicBondingCurveClient,
  deriveDbcPoolAddress,
  deriveDbcTokenVaultAddress,
  deriveMintMetadata,
} from "@meteora-ag/dynamic-bonding-curve-sdk";

/* -------------------------------------------------------------------------- */
/* Paths                                                                      */
/* -------------------------------------------------------------------------- */

const PROJECT_ROOT = process.cwd();

const ENV_FILE = path.join(
  PROJECT_ROOT,
  ".localnet/devnet-v10/rbvr-devnet.env",
);

const MINT_KEYPAIR_FILE = path.join(
  PROJECT_ROOT,
  ".localnet/devnet-v10/rbvr-mint-keypair.json",
);

const DEPLOYMENT_FILE = path.join(
  PROJECT_ROOT,
  "deployments/devnet/rbvr-beavernomics-v10.json",
);

/* -------------------------------------------------------------------------- */
/* Helpers                                                                    */
/* -------------------------------------------------------------------------- */

function loadEnvFile(filename) {
  if (!fs.existsSync(filename)) {
    throw new Error(`Environment file not found: ${filename}`);
  }

  const contents = fs.readFileSync(filename, "utf8");

  for (const originalLine of contents.split(/\r?\n/)) {
    const line = originalLine.trim();

    if (!line || line.startsWith("#")) {
      continue;
    }

    const separator = line.indexOf("=");

    if (separator === -1) {
      continue;
    }

    const key = line.slice(0, separator).trim();
    let value = line.slice(separator + 1).trim();

    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }

    if (!(key in process.env)) {
      process.env[key] = value;
    }
  }
}

function requiredEnv(...names) {
  for (const name of names) {
    const value = process.env[name];

    if (value && value.trim()) {
      return value.trim();
    }
  }

  throw new Error(
    `Missing required environment variable. Expected one of: ${names.join(", ")}`,
  );
}

function readKeypair(filename) {
  if (!fs.existsSync(filename)) {
    throw new Error(`Keypair file not found: ${filename}`);
  }

  const secret = JSON.parse(fs.readFileSync(filename, "utf8"));

  if (!Array.isArray(secret)) {
    throw new Error(`Invalid keypair JSON: ${filename}`);
  }

  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

function loadOrCreateMintKeypair(filename) {
  if (fs.existsSync(filename)) {
    const keypair = readKeypair(filename);

    console.log(`Mint keypair reused: ${keypair.publicKey.toBase58()}`);
    return keypair;
  }

  const keypair = Keypair.generate();

  fs.writeFileSync(
    filename,
    JSON.stringify(Array.from(keypair.secretKey)),
    {
      encoding: "utf8",
      mode: 0o600,
      flag: "wx",
    },
  );

  fs.chmodSync(filename, 0o600);

  console.log(`Mint keypair created: ${keypair.publicKey.toBase58()}`);
  console.log(`Saved securely to: ${filename}`);

  return keypair;
}

function normalizeTransaction(result) {
  if (result instanceof Transaction) {
    return result;
  }

  if (result?.transaction instanceof Transaction) {
    return result.transaction;
  }

  if (result?.tx instanceof Transaction) {
    return result.tx;
  }

  throw new Error(
    `Unexpected createPool() result. Returned keys: ${
      result && typeof result === "object"
        ? Object.keys(result).join(", ")
        : typeof result
    }`,
  );
}

function printSimulationLogs(logs = []) {
  if (!logs.length) {
    console.log("No simulation logs returned.");
    return;
  }

  console.log("\nSimulation logs:");
  console.log("------------------------------------------------------------");

  for (const line of logs) {
    console.log(line);
  }

  console.log("------------------------------------------------------------");
}

/* -------------------------------------------------------------------------- */
/* Main                                                                       */
/* -------------------------------------------------------------------------- */

async function main() {
  console.log("============================================================");
  console.log(" Rocket Beaver — Meteora DBC Devnet Deployment V10");
  console.log("============================================================");

  loadEnvFile(ENV_FILE);

  if (fs.existsSync(DEPLOYMENT_FILE)) {
    throw new Error(
      [
        `Deployment manifest already exists:`,
        DEPLOYMENT_FILE,
        "",
        "The script stopped to prevent an accidental duplicate deployment.",
      ].join("\n"),
    );
  }

  const rpcUrl = requiredEnv(
    "RBVR_RPC_URL",
    "SOLANA_RPC_URL",
    "RPC_URL",
  );

  const payerPath = requiredEnv(
    "RBVR_PAYER_KEYPAIR",
    "RBVR_PAYER_PATH",
    "SOLANA_KEYPAIR",
    "PAYER_KEYPAIR",
  );

  const configAddress = requiredEnv(
    "RBVR_DEVNET_CONFIG",
    "RBVR_CONFIG",
    "DBC_CONFIG",
  );

  const metadataUri =
    process.env.RBVR_METADATA_URI?.trim() ||
    "https://rocketbeavertoken.com/token.json";

  const tokenName =
    process.env.RBVR_NAME?.trim() || "Rocket Beaver";

  const tokenSymbol =
    process.env.RBVR_SYMBOL?.trim() || "RBVR";

  const shouldSend =
    (process.env.RBVR_SEND || "NO").toUpperCase() === "YES";

  const commitment = "confirmed";
  const connection = new Connection(rpcUrl, commitment);

  const payer = readKeypair(
    path.isAbsolute(payerPath)
      ? payerPath
      : path.resolve(PROJECT_ROOT, payerPath),
  );

  const config = new PublicKey(configAddress);
  const baseMint = loadOrCreateMintKeypair(MINT_KEYPAIR_FILE);

  console.log("\nDeployment parameters");
  console.log("------------------------------------------------------------");
  console.log(`RPC:          ${rpcUrl}`);
  console.log(`Payer:        ${payer.publicKey.toBase58()}`);
  console.log(`Config:       ${config.toBase58()}`);
  console.log(`Base mint:    ${baseMint.publicKey.toBase58()}`);
  console.log(`Name:         ${tokenName}`);
  console.log(`Symbol:       ${tokenSymbol}`);
  console.log(`Metadata URI: ${metadataUri}`);
  console.log(`Send enabled: ${shouldSend ? "YES" : "NO — simulation only"}`);
  console.log("------------------------------------------------------------");

  const genesisHash = await connection.getGenesisHash();
  const payerBalance = await connection.getBalance(payer.publicKey);

  console.log(`Genesis hash: ${genesisHash}`);
  console.log(`Payer balance: ${payerBalance / 1_000_000_000} SOL`);

  if (payerBalance === 0) {
    throw new Error("The payer has no SOL.");
  }

  const configAccount = await connection.getAccountInfo(config);

  if (!configAccount) {
    throw new Error(
      `DBC config account does not exist on this RPC: ${config.toBase58()}`,
    );
  }

  console.log(`Config owner: ${configAccount.owner.toBase58()}`);
  console.log(`Config size:  ${configAccount.data.length} bytes`);

  const existingMintAccount = await connection.getAccountInfo(
    baseMint.publicKey,
  );

  if (existingMintAccount) {
    throw new Error(
      [
        `The persistent mint address already exists on-chain:`,
        baseMint.publicKey.toBase58(),
        "",
        "No transaction was created. Check whether a previous deployment",
        "succeeded before deleting or replacing any local files.",
      ].join("\n"),
    );
  }

  const client = new DynamicBondingCurveClient(
    connection,
    commitment,
  );

  /*
   * We derive the addresses after reading the quote mint from the config
   * through the SDK state service. The config object is also used internally
   * by creator.createPool().
   */
  const poolConfig = await client.state.getPoolConfig(config);

  if (!poolConfig) {
    throw new Error("The SDK could not decode the DBC config account.");
  }

  const quoteMintValue =
    poolConfig.quoteMint ||
    poolConfig.account?.quoteMint ||
    poolConfig.data?.quoteMint;

  if (!quoteMintValue) {
    throw new Error(
      `Could not determine quoteMint from the decoded config.`,
    );
  }

  const quoteMint = new PublicKey(quoteMintValue);

  const derivedPool = deriveDbcPoolAddress(
    quoteMint,
    baseMint.publicKey,
    config,
  );

  const baseVault = deriveDbcTokenVaultAddress(
    derivedPool,
    baseMint.publicKey,
  );

  const quoteVault = deriveDbcTokenVaultAddress(
    derivedPool,
    quoteMint,
  );

  const metadata = deriveMintMetadata(baseMint.publicKey);

  console.log("\nDerived addresses");
  console.log("------------------------------------------------------------");
  console.log(`Quote mint:    ${quoteMint.toBase58()}`);
  console.log(`Pool:          ${derivedPool.toBase58()}`);
  console.log(`Base vault:    ${baseVault.toBase58()}`);
  console.log(`Quote vault:   ${quoteVault.toBase58()}`);
  console.log(`Metadata PDA:  ${metadata.toBase58()}`);
  console.log("------------------------------------------------------------");

  const existingPool = await connection.getAccountInfo(derivedPool);

  if (existingPool) {
    throw new Error(
      `Derived pool already exists: ${derivedPool.toBase58()}`,
    );
  }

  console.log("\nBuilding pool-creation transaction...");

  const createPoolResult = await client.creator.createPool({
    name: tokenName,
    symbol: tokenSymbol,
    uri: metadataUri,
    payer: payer.publicKey,
    poolCreator: payer.publicKey,
    config,
    baseMint: baseMint.publicKey,
  });

  const transaction = normalizeTransaction(createPoolResult);

  const latestBlockhash = await connection.getLatestBlockhash(
    commitment,
  );

  transaction.feePayer = payer.publicKey;
  transaction.recentBlockhash = latestBlockhash.blockhash;
  transaction.partialSign(payer, baseMint);

  console.log(`Instructions: ${transaction.instructions.length}`);
  console.log("Simulating transaction...");

  const simulation = await connection.simulateTransaction(
    transaction,
    [payer, baseMint],
  );

  printSimulationLogs(simulation.value.logs);

  if (simulation.value.err) {
    console.error(
      "\nSimulation error:",
      JSON.stringify(simulation.value.err, null, 2),
    );

    throw new Error(
      "Simulation failed. Nothing was sent to Devnet.",
    );
  }

  console.log("\nSIMULATION PASS");

  if (simulation.value.unitsConsumed !== undefined) {
    console.log(
      `Compute units consumed: ${simulation.value.unitsConsumed}`,
    );
  }

  if (!shouldSend) {
    console.log("\nDRY RUN COMPLETE — nothing was broadcast.");
    console.log(
      "After reviewing the output, send with:",
    );
    console.log(
      "RBVR_SEND=YES node scripts/meteora/devnet/create-rbvr-pool-v10.mjs",
    );

    return;
  }

  console.log("\nRBVR_SEND=YES detected.");
  console.log("Broadcasting transaction to Devnet...");

  const signature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [payer, baseMint],
    {
      commitment,
      preflightCommitment: commitment,
      skipPreflight: false,
      maxRetries: 5,
    },
  );

  const confirmation = await connection.confirmTransaction(
    {
      signature,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    },
    "finalized",
  );

  if (confirmation.value.err) {
    throw new Error(
      `Transaction confirmation failed: ${JSON.stringify(
        confirmation.value.err,
      )}`,
    );
  }

  const manifest = {
    project: "Rocket Beaver",
    symbol: tokenSymbol,
    network: "devnet",
    rpcUrl,
    name: tokenName,
    metadataUri,
    payer: payer.publicKey.toBase58(),
    poolCreator: payer.publicKey.toBase58(),
    config: config.toBase58(),
    baseMint: baseMint.publicKey.toBase58(),
    quoteMint: quoteMint.toBase58(),
    pool: derivedPool.toBase58(),
    baseVault: baseVault.toBase58(),
    quoteVault: quoteVault.toBase58(),
    metadata: metadata.toBase58(),
    transactionSignature: signature,
    createdAt: new Date().toISOString(),
  };

  fs.mkdirSync(path.dirname(DEPLOYMENT_FILE), {
    recursive: true,
  });

  fs.writeFileSync(
    DEPLOYMENT_FILE,
    `${JSON.stringify(manifest, null, 2)}\n`,
    {
      encoding: "utf8",
      mode: 0o644,
      flag: "wx",
    },
  );

  console.log("\nDEVNET DEPLOYMENT PASS");
  console.log("------------------------------------------------------------");
  console.log(`Signature: ${signature}`);
  console.log(`Mint:      ${baseMint.publicKey.toBase58()}`);
  console.log(`Pool:      ${derivedPool.toBase58()}`);
  console.log(`Manifest:  ${DEPLOYMENT_FILE}`);
  console.log("------------------------------------------------------------");
}

main().catch((error) => {
  console.error("\nV10 DEPLOYMENT ERROR");
  console.error(
    error instanceof Error ? error.stack || error.message : error,
  );
  process.exitCode = 1;
});
