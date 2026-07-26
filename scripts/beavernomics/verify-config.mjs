import fs from "node:fs";

const config = JSON.parse(
  fs.readFileSync(
    new URL("../../config/beavernomics.json", import.meta.url),
    "utf8"
  )
);

const allocations = config.allocationsBps;
const allocationTotal = Object.values(allocations).reduce(
  (total, value) => total + value,
  0
);

if (config.totalFeeBps !== 100) {
  throw new Error(
    `RBVR total fee must equal 100 bps (1%). Found ${config.totalFeeBps} bps.`
  );
}

if (allocationTotal !== config.totalFeeBps) {
  throw new Error(
    `Allocations total ${allocationTotal} bps, but total fee is ` +
    `${config.totalFeeBps} bps.`
  );
}

if (!config.buyback.automatic) {
  throw new Error("Automatic buyback must be enabled.");
}

if (!config.buyback.burnImmediately) {
  throw new Error("Purchased RBVR must be burned immediately.");
}

console.log("========================================");
console.log("RBVR BEAVERNOMICS CONFIGURATION VALID");
console.log("========================================");
console.log(`Total fee:              ${config.totalFeeBps / 100}%`);
console.log(`Protocol reserve:       ${allocations.protocolReserve / 100}%`);
console.log(`Automatic buyback/burn: ${allocations.automaticBuybackBurn / 100}%`);
console.log(`Liquidity growth:       ${allocations.liquidityGrowth / 100}%`);
console.log(`Company/operations:     ${allocations.companyOperations / 100}%`);
console.log(`Founder initial:        ${allocations.founderInitial / 100}%`);
console.log();
console.log(
  `Buyback threshold:     ${
    config.buyback.minimumExecutionLamports / 1_000_000_000
  } SOL`
);
console.log(
  `Maximum slippage:      ${
    config.buyback.maximumSlippageBps / 100
  }%`
);
