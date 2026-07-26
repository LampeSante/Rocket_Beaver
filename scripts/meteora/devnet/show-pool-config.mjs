import util from "node:util";
import { Connection, PublicKey } from "@solana/web3.js";
import {
  DynamicBondingCurveClient,
} from "@meteora-ag/dynamic-bonding-curve-sdk";

const RPC = "https://api.devnet.solana.com";

const POOL = new PublicKey(
  "A5p2ryjB6kpywEe3pMY8z9YHSoMTLRzmgyMa1RpcRZXz"
);

const EXPECTED_CONFIG = new PublicKey(
  "BcazQs7EibXsm1QJ3HPj72tfn4L2qCiysVmUo7D8RU5E"
);

const connection = new Connection(RPC, "confirmed");
const client = new DynamicBondingCurveClient(connection, "confirmed");

function printable(value) {
  return JSON.parse(
    JSON.stringify(value, (_, item) => {
      if (
        item &&
        typeof item === "object" &&
        typeof item.toBase58 === "function"
      ) {
        return item.toBase58();
      }

      if (
        item &&
        typeof item === "object" &&
        typeof item.toString === "function" &&
        item.constructor?.name === "BN"
      ) {
        return item.toString();
      }

      if (typeof item === "bigint") {
        return item.toString();
      }

      return item;
    })
  );
}

console.log("============================================================");
console.log("RBVR METEORA DBC INSPECTION");
console.log("============================================================");

const pool = await client.state.getPool(POOL);

if (!pool) {
  throw new Error(`Pool not found: ${POOL.toBase58()}`);
}

const poolState = pool.poolState ?? pool;
const configAddress = new PublicKey(poolState.config);

console.log("\nPOOL ADDRESS");
console.log(POOL.toBase58());

console.log("\nCONFIG FROM POOL");
console.log(configAddress.toBase58());

console.log("\nEXPECTED CONFIG");
console.log(EXPECTED_CONFIG.toBase58());

if (!configAddress.equals(EXPECTED_CONFIG)) {
  throw new Error("Pool config does not match the deployment manifest.");
}

const config = await client.state.getPoolConfig(configAddress);

if (!config) {
  throw new Error(`Config not found: ${configAddress.toBase58()}`);
}

const feeMetrics = await client.state.getPoolFeeMetrics(POOL);
const feeBreakdown = await client.state.getPoolFeeBreakdown(POOL);

console.log("\nPOOL STATE");
console.log(
  util.inspect(printable(pool), {
    depth: null,
    colors: false,
  })
);

console.log("\nPOOL CONFIG");
console.log(
  util.inspect(printable(config), {
    depth: null,
    colors: false,
  })
);

console.log("\nFEE METRICS");
console.log(
  util.inspect(printable(feeMetrics), {
    depth: null,
    colors: false,
  })
);

console.log("\nFEE BREAKDOWN");
console.log(
  util.inspect(printable(feeBreakdown), {
    depth: null,
    colors: false,
  })
);

console.log("\nKEY BEAVERNOMICS FIELDS");
console.log({
  creatorTradingFeePercentage:
    config.creatorTradingFeePercentage?.toString?.() ??
    config.creatorTradingFeePercentage,

  collectFeeMode:
    config.collectFeeMode?.toString?.() ??
    config.collectFeeMode,

  baseFee:
    printable(config.poolFees?.baseFee ?? null),

  dynamicFee:
    printable(config.poolFees?.dynamicFee ?? null),

  partnerBaseFee:
    feeMetrics.current.partnerBaseFee.toString(),

  partnerQuoteFee:
    feeMetrics.current.partnerQuoteFee.toString(),

  creatorBaseFee:
    feeMetrics.current.creatorBaseFee.toString(),

  creatorQuoteFee:
    feeMetrics.current.creatorQuoteFee.toString(),

  totalTradingBaseFee:
    feeMetrics.total.totalTradingBaseFee.toString(),

  totalTradingQuoteFee:
    feeMetrics.total.totalTradingQuoteFee.toString(),
});
