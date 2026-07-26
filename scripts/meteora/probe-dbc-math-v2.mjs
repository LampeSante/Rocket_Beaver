import Decimal from "decimal.js";
import BN from "bn.js";

import {
  getPercentageSupplyOnMigration,
  getMigrationQuoteAmount,
  calculateAdjustedPercentageSupplyOnMigration,
} from "@meteora-ag/dynamic-bonding-curve-sdk";

const GENESIS = new BN("1000000000");
const FUTURE_LIQUIDITY = new BN("290000000");
const COMMUNITY_INITIATIVES = new BN("100000000");

const OUTSIDE_LAUNCH =
  FUTURE_LIQUIDITY.add(COMMUNITY_INITIATIVES);

const LAUNCH_INVENTORY =
  GENESIS.sub(OUTSIDE_LAUNCH);

const INITIAL_MC = new Decimal("10000");

const MIGRATION_MARKET_CAPS = [
  25000,
  40000,
  50000,
  75000,
  100000,
  150000,
  250000,
  500000,
  1000000
];

const noLockedVesting = {
  totalLockedVestingAmount: new BN(0),
  numberOfVestingPeriod: new BN(0),
  cliffUnlockAmount: new BN(0),
  totalVestingDuration: new BN(0),
  cliffDurationFromMigrationTime: new BN(0),
};

function fmtNumber(value, digits = 6) {
  return Number(value).toLocaleString(
    "en-US",
    { maximumFractionDigits: digits }
  );
}

console.log("============================================================");
console.log("ROCKET BEAVER — METEORA DBC MATHEMATICAL PROBE V2");
console.log("============================================================");

console.log();
console.log(
  "Canonical genesis supply:      ",
  GENESIS.toString()
);

console.log(
  "Outside launch architecture:   ",
  OUTSIDE_LAUNCH.toString()
);

console.log(
  "Launch-related inventory:      ",
  LAUNCH_INVENTORY.toString()
);

console.log();
console.log(
  "Desired curve distribution:    400000000 RBVR"
);

console.log(
  "Desired migration allocation:  210000000 RBVR"
);

console.log();

for (const mc of MIGRATION_MARKET_CAPS) {

  const migrationMC = new Decimal(mc);

  console.log("------------------------------------------------------------");
  console.log(
    `Migration market cap: $${mc.toLocaleString("en-US")}`
  );

  try {

    const percentage =
      getPercentageSupplyOnMigration(
        INITIAL_MC,
        migrationMC,
        noLockedVesting,
        OUTSIDE_LAUNCH,
        GENESIS
      );

    console.log(
      "SDK percentageSupplyOnMigration:",
      percentage
    );

    const percentageDecimal =
      new Decimal(percentage);

    const migrationQuote =
      getMigrationQuoteAmount(
        migrationMC,
        percentageDecimal
      );

    console.log(
      "SDK migrationQuoteAmount:",
      migrationQuote.toString()
    );

    /*
      Also calculate the migration percentage with an explicit
      migration-fee assumption of zero.

      This allows us to compare the legacy helper with the
      fee-aware helper without making any assumptions yet about
      the final Meteora migration fee.
    */

    const adjustedZeroFee =
      calculateAdjustedPercentageSupplyOnMigration(
        INITIAL_MC.toNumber(),
        migrationMC.toNumber(),
        { feePercentage: 0 },
        noLockedVesting,
        OUTSIDE_LAUNCH,
        GENESIS
      );

    console.log(
      "Adjusted migration % (0 fee):",
      adjustedZeroFee
    );

    /*
      Interpret the percentage only as a diagnostic estimate.

      We are NOT yet assuming whether the SDK percentage is:
        - percentage of total genesis supply, or
        - percentage of launch inventory.

      We print both possibilities so the next phase can compare
      them against the SDK-generated curve itself.
    */

    const pctFraction =
      new Decimal(percentage).div(100);

    const ifGenesis =
      new Decimal(GENESIS.toString())
        .mul(pctFraction);

    const ifLaunchInventory =
      new Decimal(LAUNCH_INVENTORY.toString())
        .mul(pctFraction);

    console.log(
      "If % applies to 1B supply:",
      fmtNumber(ifGenesis.toNumber()),
      "RBVR"
    );

    console.log(
      "If % applies to 610M launch inventory:",
      fmtNumber(ifLaunchInventory.toNumber()),
      "RBVR"
    );

  } catch (error) {

    console.log("ERROR:");
    console.log(
      error?.stack ?? error?.message ?? String(error)
    );

  }

  console.log();
}

console.log("============================================================");
console.log("IMPORTANT");
console.log("============================================================");
console.log();
console.log(
  "This probe evaluates Meteora SDK migration-percentage math."
);
console.log(
  "It does NOT yet prove exact curve-sold or DAMM migration amounts."
);
console.log(
  "Those will be derived from an actual SDK-built curve next."
);
console.log();
console.log("MAINNET: BLOCKED");
