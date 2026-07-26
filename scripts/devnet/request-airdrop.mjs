import {
  Connection,
  Keypair,
  clusterApiUrl,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";

import fs from "node:fs";

const secret = Uint8Array.from(
  JSON.parse(
    fs.readFileSync(
      "keys/devnet/deployment-authority.json",
      "utf8"
    )
  )
);

const wallet = Keypair.fromSecretKey(secret);

const connection = new Connection(
  clusterApiUrl("devnet"),
  "confirmed"
);

console.log("==============================================");
console.log("ROCKET BEAVER — DEVNET FUNDING");
console.log("==============================================");
console.log("Wallet:", wallet.publicKey.toBase58());

const before = await connection.getBalance(wallet.publicKey);
console.log("Current balance:", before / LAMPORTS_PER_SOL, "SOL");

if (before >= 0.5 * LAMPORTS_PER_SOL) {
  console.log();
  console.log("PASS: Wallet already has enough Devnet SOL to continue.");
  process.exit(0);
}

const amounts = [0.5, 0.25, 0.1];

for (const amount of amounts) {

  try {

    console.log();
    console.log(`Trying ${amount} Devnet SOL airdrop...`);

    const signature = await connection.requestAirdrop(
      wallet.publicKey,
      amount * LAMPORTS_PER_SOL
    );

    console.log("Transaction submitted:", signature);
    console.log("Waiting for confirmation...");

    const latest = await connection.getLatestBlockhash();

    await connection.confirmTransaction(
      {
        signature,
        blockhash: latest.blockhash,
        lastValidBlockHeight: latest.lastValidBlockHeight
      },
      "confirmed"
    );

    const balance = await connection.getBalance(wallet.publicKey);

    console.log();
    console.log("SUCCESS");
    console.log("Balance:", balance / LAMPORTS_PER_SOL, "SOL");

    process.exit(0);

  } catch (error) {

    console.log(`Failed at ${amount} SOL.`);
    console.log(
      error?.message ||
      String(error)
    );

  }
}

console.log();
console.log("==============================================");
console.log("PUBLIC DEVNET FAUCET CURRENTLY UNAVAILABLE");
console.log("==============================================");
console.log();
console.log("Your wallet is valid.");
console.log("Do NOT recreate it.");
console.log();
console.log("Public address:");
console.log(wallet.publicKey.toBase58());
console.log();
console.log("Use an external Solana Devnet faucet if necessary.");
console.log("Never send real SOL to this Devnet-only deployment wallet.");

process.exit(1);
