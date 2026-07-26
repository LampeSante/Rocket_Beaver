import {
    Connection,
    PublicKey,
    Keypair,
    LAMPORTS_PER_SOL,
    sendAndConfirmTransaction,
} from "@solana/web3.js";

import {
    getAssociatedTokenAddress,
    getAccount,
    getMint,
} from "@solana/spl-token";

import { DynamicBondingCurveClient } from "@meteora-ag/dynamic-bonding-curve-sdk";
import BN from "bn.js";
import fs from "fs";

// -----------------------------------------------------
// CONFIG
// -----------------------------------------------------

const RPC = "https://api.devnet.solana.com";
const SELL_AMOUNT_RBVR = "100";

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

const RBVR_MINT = new PublicKey(
    "9L9PUQ8Pqi5n11MRhP42zNbQJwbGCNy673QqxPc46XE6"
);

// -----------------------------------------------------
// CLIENT AND BALANCES
// -----------------------------------------------------

const client = new DynamicBondingCurveClient(
    connection,
    "confirmed"
);

console.log("========================================");
console.log("Rocket Beaver Devnet Sell Test");
console.log("========================================");
console.log("Wallet:", wallet.publicKey.toBase58());

const mintInfo = await getMint(connection, RBVR_MINT);
const tokenDecimals = mintInfo.decimals;

const tokenAccountAddress = await getAssociatedTokenAddress(
    RBVR_MINT,
    wallet.publicKey
);

const tokenAccountBefore = await getAccount(
    connection,
    tokenAccountAddress
);

const solBalanceBefore = await connection.getBalance(
    wallet.publicKey
);

const divisor = 10 ** tokenDecimals;

console.log(
    "SOL before:",
    solBalanceBefore / LAMPORTS_PER_SOL
);

console.log(
    "RBVR before:",
    Number(tokenAccountBefore.amount) / divisor
);

// Convert 100 RBVR into raw token units without floating-point errors.
const amountIn = new BN(SELL_AMOUNT_RBVR).mul(
    new BN(10).pow(new BN(tokenDecimals))
);

if (new BN(tokenAccountBefore.amount.toString()).lt(amountIn)) {
    throw new Error(
        `Insufficient RBVR balance. Attempted to sell ${SELL_AMOUNT_RBVR} RBVR.`
    );
}

console.log();
console.log(`Selling ${SELL_AMOUNT_RBVR} RBVR for SOL...`);
console.log();

// -----------------------------------------------------
// SWAP: RBVR → SOL
// -----------------------------------------------------

const transaction = await client.pool.swap({
    pool: POOL,
    owner: wallet.publicKey,
    payer: wallet.publicKey,

    amountIn,

    // Devnet functional test only.
    minimumAmountOut: new BN(0),

    // Base token RBVR → quote token SOL
    swapBaseForQuote: true,

    referralTokenAccount: null,
});

const signature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [wallet],
    {
        commitment: "confirmed",
    }
);

// -----------------------------------------------------
// BALANCES AFTER SELL
// -----------------------------------------------------

const tokenAccountAfter = await getAccount(
    connection,
    tokenAccountAddress
);

const solBalanceAfter = await connection.getBalance(
    wallet.publicKey
);

console.log("========================================");
console.log("SUCCESS");
console.log("========================================");
console.log("Transaction:");
console.log(signature);
console.log();

console.log(
    "RBVR after:",
    Number(tokenAccountAfter.amount) / divisor
);

console.log(
    "SOL after:",
    solBalanceAfter / LAMPORTS_PER_SOL
);

console.log(
    "Approximate SOL balance change:",
    (solBalanceAfter - solBalanceBefore) / LAMPORTS_PER_SOL
);
