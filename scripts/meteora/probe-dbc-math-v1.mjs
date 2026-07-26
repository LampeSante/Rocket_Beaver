import {
  getPercentageSupplyOnMigration,
  getMigrationQuoteAmount,
  getMigrationThresholdPrice,
  getMigrationBaseToken,
  MigrationOption,
  getProtocolMigrationFee,
} from "@meteora-ag/dynamic-bonding-curve-sdk";

const GENESIS = 1_000_000_000;
const FUTURE_LIQUIDITY = 290_000_000;
const COMMUNITY_INITIATIVES = 100_000_000;
const OUTSIDE_LAUNCH = FUTURE_LIQUIDITY + COMMUNITY_INITIATIVES;

const INITIAL_MC = 10_000;

// Start with a range rather than assuming $40K is correct.
const MIGRATION_MARKET_CAPS = [
  25_000,
  40_000,
  50_000,
  75_000,
  100_000,
  150_000,
  250_000,
  500_000,
];

const noLockedVesting = {
  totalLockedVestingAmount: 0,
  numberOfVestingPeriod: 0,
  cliffUnlockAmount: 0,
  totalVestingDuration: 0,
  cliffDurationFromMigrationTime: 0,
};

function fmt(n) {
  if (typeof n === "bigint") return n.toLocaleString("en-US");
  if (typeof n === "number")
    return Number.isFinite(n)
      ? n.toLocaleString("en-US", { maximumFractionDigits: 6 })
      : String(n);
  return String(n);
}

console.log("============================================================");
console.log("ROCKET BEAVER — METEORA DBC MATHEMATICAL PROBE V1");
console.log("============================================================");
console.log();
console.log(`Canonical genesis supply:      ${fmt(GENESIS)} RBVR`);
console.log(`Future liquidity reserve:      ${fmt(FUTURE_LIQUIDITY)} RBVR`);
console.log(`Community initiatives reserve: ${fmt(COMMUNITY_INITIATIVES)} RBVR`);
console.log(`Outside launch architecture:   ${fmt(OUTSIDE_LAUNCH)} RBVR`);
console.log(`Launch-related inventory:      ${fmt(GENESIS - OUTSIDE_LAUNCH)} RBVR`);
console.log();
console.log(`Initial market cap:            $${fmt(INITIAL_MC)}`);
console.log();

for (const migrationMarketCap of MIGRATION_MARKET_CAPS) {
  console.log("------------------------------------------------------------");
  console.log(`TEST MIGRATION MARKET CAP: $${fmt(migrationMarketCap)}`);

  try {
    const pct = getPercentageSupplyOnMigration(
      INITIAL_MC,
      migrationMarketCap,
      noLockedVesting,
      OUTSIDE_LAUNCH,
      GENESIS
    );

    console.log("percentageSupplyOnMigration:");
    console.dir(pct, { depth: null });

    try {
      const quote = getMigrationQuoteAmount(
        migrationMarketCap,
        pct
      );

      console.log("migrationQuoteAmount:");
      console.dir(quote, { depth: null });

      try {
        const thresholdPrice = getMigrationThresholdPrice(
          quote,
          pct
        );

        console.log("migrationThresholdPrice:");
        console.dir(thresholdPrice, { depth: null });

        try {
          const migrationBase = getMigrationBaseToken(
            quote,
            thresholdPrice,
            MigrationOption.MET_DAMM_V2
          );

          console.log("migrationBaseToken:");
          console.dir(migrationBase, { depth: null });
        } catch (e) {
          console.log("getMigrationBaseToken ERROR:");
          console.log(e?.message ?? e);
        }
      } catch (e) {
        console.log("getMigrationThresholdPrice ERROR:");
        console.log(e?.message ?? e);
      }
    } catch (e) {
      console.log("getMigrationQuoteAmount ERROR:");
      console.log(e?.message ?? e);
    }
  } catch (e) {
    console.log("getPercentageSupplyOnMigration ERROR:");
    console.log(e?.message ?? e);
  }

  console.log();
}

console.log("------------------------------------------------------------");
console.log("PROTOCOL MIGRATION FEE PROBE");

for (const option of [MigrationOption.MET_DAMM, MigrationOption.MET_DAMM_V2]) {
  try {
    const result = getProtocolMigrationFee(option);
    console.log(`Migration option ${option}:`);
    console.dir(result, { depth: null });
  } catch (e) {
    console.log(`Migration option ${option} ERROR:`);
    console.log(e?.message ?? e);
  }
}

console.log();
console.log("MAINNET: BLOCKED");
