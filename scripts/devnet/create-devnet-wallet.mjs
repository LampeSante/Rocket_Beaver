import {
  Keypair
} from "@solana/web3.js";

import fs from "node:fs";
import path from "node:path";

const outDir = path.resolve("keys/devnet");
const outFile = path.join(outDir, "deployment-authority.json");

fs.mkdirSync(outDir, { recursive: true });

if (fs.existsSync(outFile)) {
  console.error("REFUSING: Devnet deployment wallet already exists:");
  console.error(outFile);
  process.exit(1);
}

const keypair = Keypair.generate();

fs.writeFileSync(
  outFile,
  JSON.stringify(Array.from(keypair.secretKey))
);

fs.chmodSync(outFile, 0o600);

console.log("==============================================");
console.log("ROCKET BEAVER — DEVNET DEPLOYMENT AUTHORITY");
console.log("==============================================");
console.log("Public address:");
console.log(keypair.publicKey.toBase58());
console.log();
console.log("Private key stored locally at:");
console.log(outFile);
console.log();
console.log("DEVNET ONLY — NEVER SEND REAL SOL TO THIS WALLET.");
