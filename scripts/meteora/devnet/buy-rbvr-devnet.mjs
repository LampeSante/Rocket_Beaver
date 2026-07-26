import {
    Connection,
    PublicKey,
    Keypair,
    LAMPORTS_PER_SOL,
    sendAndConfirmTransaction,
} from "@solana/web3.js";

import BN from "bn.js";
import fs from "fs";

import { DynamicBondingCurveClient } from "@meteora-ag/dynamic-bonding-curve-sdk";

// -----------------------------------------------------
// CONFIG
// -----------------------------------------------------

const RPC = "https://api.devnet.solana.com";

const connection = new Connection(RPC, "confirmed");

const wallet = Keypair.fromSecretKey(
    Uint8Array.from(
        JSON.parse(
            fs.readFileSync(
                "/home/alec_elliott/rocket-beaver-dev/keys/local-deployment.json",
                "utf8"
            )
        )
    )
);

const POOL = new PublicKey(
    "A5p2ryjB6kpywEe3pMY8z9YHSoMTLRzmgyMa1RpcRZXz"
);

// -----------------------------------------------------
// CLIENT
// -----------------------------------------------------

const client = new DynamicBondingCurveClient(
    connection,
    "confirmed"
);

console.log("========================================");
console.log("Rocket Beaver Devnet Buy Test");
console.log("========================================");

console.log("Wallet:", wallet.publicKey.toBase58());

const balance = await connection.getBalance(wallet.publicKey);

console.log(
    "SOL Balance:",
    balance / LAMPORTS_PER_SOL
);

console.log();
console.log("Buying 0.01 SOL worth of RBVR...");
console.log();

// -----------------------------------------------------
// SWAP
// -----------------------------------------------------

const transaction = await client.pool.swap({
    pool: POOL,
    owner: wallet.publicKey,
    payer: wallet.publicKey,

    amountIn: new BN(
        Math.floor(0.01 * LAMPORTS_PER_SOL)
    ),

    minimumAmountOut: new BN(0),

    swapBaseForQuote: false,

    // No referral account for this test purchase
    referralTokenAccount: null,
});

const signature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [wallet]
);

console.log("========================================");
console.log("SUCCESS");
console.log("========================================");
console.log("Transaction:");
console.log(signature);
