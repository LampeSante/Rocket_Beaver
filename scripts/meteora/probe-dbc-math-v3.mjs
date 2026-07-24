import Decimal from "decimal.js";
import BN from "bn.js";

import {
  getPercentageSupplyOnMigration,
  getMigrationQuoteAmount,
  calculateAdjustedPercentageSupplyOnMigration,
} from "@meteora-ag/dynamic-bonding-curve-sdk";

const GENESIS = new BN("1000000000");
const LEFTOVER = new BN("390000000");

const TARGET_CURVE = new BN("400000000");
const TARGET_MIGRATION = new BN("210000000");

const INITIAL_MC = new Decimal("10000");

const noLockedVesting = {
  cliffUnlockAmount: new BN(0),
  amountPerPeriod: new BN(0),
  numberOfPeriod: new BN(0),
  cliffDurationFromMigrationTime: new BN(0),
  frequency: new BN(0),
};

const MIGRATION_MARKET_CAPS = [
  25000,
  30000,
  35000,
  36000,
  36250,
  36286,
  36300,
  36500,
  40000,
  50000,
  75000,
  100000,
];

function fmt(n, digits = 6) {
  return Number(n).toLocaleString("en-US", {
    maximumFractionDigits: digits
  });
}

console.log("============================================================");
console.log("ROCKET BEAVER — METEORA DBC MATHEMATICAL PROBE V3");
console.log("============================================================");
console.log();

console.log(`Canonical supply:       ${GENESIS.toString()} RBVR`);
console.log(`Leftover / outside DBC: ${LEFTOVER.toString()} RBVR`);
console.log(`Launch architecture:    ${GENESIS.sub(LEFTOVER).toString()} RBVR`);
console.log();
console.log(`Target curve side:      ${TARGET_CURVE.toString()} RBVR`);
console.log(`Target migration side:  ${TARGET_MIGRATION.toString()} RBVR`);
console.log();

let closest = null;

for (const mc of MIGRATION_MARKET_CAPS) {
  const migrationMC = new Decimal(mc);

  try {
    const percentage =
      getPercentageSupplyOnMigration(
        INITIAL_MC,
        migrationMC,
        noLockedVesting,
        LEFTOVER,
        GENESIS
      );

    const migrationTokens =
      new Decimal(GENESIS.toString())
        .mul(new Decimal(percentage))
        .div(100);

    const theoreticalCurveTokens =
      new Decimal(GENESIS.sub(LEFTOVER).toString())
        .sub(migrationTokens);

    const migrationQuote =
      getMigrationQuoteAmount(
        migrationMC,
        new Decimal(percentage)
      );

    const adjustedZeroFee =
      calculateAdjustedPercentageSupplyOnMigration(
        INITIAL_MC.toNumber(),
        migrationMC.toNumber(),
        { feePercentage: 0 },
        noLockedVesting,
        LEFTOVER,
        GENESIS
      );

    const migrationDifference =
      migrationTokens.sub(
        new Decimal(TARGET_MIGRATION.toString())
      ).abs();

    if (
      !closest ||
      migrationDifference.lt(closest.difference)
    ) {
      closest = {
        mc,
        percentage,
        migrationTokens,
        theoreticalCurveTokens,
        migrationQuote,
        adjustedZeroFee,
        difference: migrationDifference
      };
    }

    console.log("------------------------------------------------------------");
    console.log(`Migration market cap: $${fmt(mc, 2)}`);
    console.log(`Migration supply %:    ${fmt(percentage, 9)}%`);
    console.log(
      `Migration tokens:      ${fmt(migrationTokens.toNumber(), 3)} RBVR`
    );
    console.log(
      `Theoretical curve:     ${fmt(theoreticalCurveTokens.toNumber(), 3)} RBVR`
    );
    console.log(
      `Migration quote:       ${fmt(migrationQuote.toNumber(), 9)}`
    );
    console.log(
      `Adjusted % @ 0 fee:    ${fmt(adjustedZeroFee, 9)}%`
    );
    console.log(
      `Difference from 210M:  ${fmt(migrationDifference.toNumber(), 3)} RBVR`
    );

  } catch (error) {
    console.log("------------------------------------------------------------");
    console.log(`Migration market cap: $${fmt(mc, 2)}`);
    console.log("ERROR:");
    console.log(error?.stack ?? error?.message ?? String(error));
  }
}

console.log();
console.log("============================================================");
console.log("CLOSEST TESTED POINT");
console.log("============================================================");

if (closest) {
  console.log(
    `Migration market cap:  $${fmt(closest.mc, 2)}`
  );
  console.log(
    `Migration percentage:  ${fmt(closest.percentage, 9)}%`
  );
  console.log(
    `Migration tokens:      ${fmt(closest.migrationTokens.toNumber(), 3)}`
  );
  console.log(
    `Theoretical curve:     ${fmt(closest.theoreticalCurveTokens.toNumber(), 3)}`
  );
  console.log(
    `Difference from 210M:  ${fmt(closest.difference.toNumber(), 3)}`
  );
}

console.log();
console.log("NOTE:");
console.log(
  "Theoretical curve tokens = 610M available launch inventory minus"
);
console.log(
  "the SDK-calculated migration allocation."
);
console.log(
  "This does not yet include the SDK swap buffer or prove the final"
);
console.log(
  "actual DBC curve/migration token accounting."
);
console.log();
console.log("MAINNET: BLOCKED");
