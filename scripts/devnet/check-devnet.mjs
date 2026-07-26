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

const balance = await connection.getBalance(wallet.publicKey);

console.log("Network: Solana Devnet");
console.log("Address:", wallet.publicKey.toBase58());
console.log("Balance:", balance / LAMPORTS_PER_SOL, "SOL");
