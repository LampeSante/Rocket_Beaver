import Decimal from "decimal.js";
import BN from "bn.js";

import {
  getPercentageSupplyOnMigration,
  getMigrationQuoteAmount,
  getMigrationQuoteThresholdFromMigrationQuoteAmount,
  getSqrtPriceFromPrice,
  getMigrationBaseToken,
  getFirstCurve,
  getMigrationThresholdPrice,
  getBaseTokenForSwap,
  getSwapAmountWithBuffer,
  getTotalSupplyFromCurve,
  getLockedVestingParams,
  getTotalVestingAmount,
  convertToLamports,
  fromDecimalToBN,
  MigrationOption,
} from "@meteora-ag/dynamic-bonding-curve-sdk";

Decimal.set({ precision: 60 });

const BASE_DECIMALS = 9;
const QUOTE_DECIMALS = 9;

const GENESIS = new Decimal("1000000000");
const LEFTOVER = new Decimal("390000000");

const TARGET_CURVE = new Decimal("400000000");
const TARGET_MIGRATION = new Decimal("210000000");

const INITIAL_MC = new Decimal("10000");

const SOLVED_MIGRATION_MC =
  new Decimal("36281.179138321986");

const MIGRATION_FEE_PERCENT = 0;

const migrationOption =
  MigrationOption.MET_DAMM_V2;

const lockedVesting =
  getLockedVestingParams(
    0,
    0,
    0,
    0,
    0,
    BASE_DECIMALS
  );

const totalSupplyLamports =
  convertToLamports(
    GENESIS.toString(),
    BASE_DECIMALS
  );

const leftoverLamports =
  convertToLamports(
    LEFTOVER.toString(),
    BASE_DECIMALS
  );

function toTokens(bn) {
  return new Decimal(bn.toString())
    .div(
      new Decimal(10)
        .pow(BASE_DECIMALS)
    );
}

function fmt(value, dp = 9) {
  return new Decimal(value).toFixed(dp);
}

console.log(
  "============================================================"
);
console.log(
  "ROCKET BEAVER — EXACT METEORA DBC ACCOUNTING V6"
);
console.log(
  "============================================================"
);
console.log();

const percentageSupplyOnMigration =
  getPercentageSupplyOnMigration(
    INITIAL_MC,
    SOLVED_MIGRATION_MC,
    lockedVesting,
    leftoverLamports,
    totalSupplyLamports
  );

const theoreticalMigrationSupply =
  GENESIS
    .mul(percentageSupplyOnMigration)
    .div(100);

const migrationQuoteAmount =
  getMigrationQuoteAmount(
    SOLVED_MIGRATION_MC,
    new Decimal(
      percentageSupplyOnMigration
    )
  );

const migrationQuoteThreshold =
  getMigrationQuoteThresholdFromMigrationQuoteAmount(
    migrationQuoteAmount,
    new Decimal(
      MIGRATION_FEE_PERCENT
    )
  );

const migrationPrice =
  migrationQuoteAmount.div(
    theoreticalMigrationSupply
  );

const migrateSqrtPrice =
  getSqrtPriceFromPrice(
    migrationPrice.toString(),
    BASE_DECIMALS,
    QUOTE_DECIMALS
  );

const migrationQuoteThresholdLamports =
  convertToLamports(
    migrationQuoteThreshold.toString(),
    QUOTE_DECIMALS
  );

const migrationQuoteAmountLamports =
  fromDecimalToBN(
    migrationQuoteAmount.mul(
      new Decimal(10)
        .pow(QUOTE_DECIMALS)
    )
  );

const migrationBaseAmount =
  getMigrationBaseToken(
    migrationQuoteAmountLamports,
    migrateSqrtPrice,
    migrationOption
  );

const vestingAmount =
  getTotalVestingAmount(
    lockedVesting
  );

const initialSwapAmount =
  totalSupplyLamports
    .sub(migrationBaseAmount)
    .sub(vestingAmount)
    .sub(leftoverLamports);

const {
  sqrtStartPrice,
  curve
} = getFirstCurve(
  migrateSqrtPrice,
  migrationBaseAmount,
  initialSwapAmount,
  migrationQuoteThresholdLamports,
  MIGRATION_FEE_PERCENT
);

const actualMigrationSqrtPrice =
  getMigrationThresholdPrice(
    migrationQuoteThresholdLamports,
    sqrtStartPrice,
    curve
  );

const actualSwapBase =
  getBaseTokenForSwap(
    sqrtStartPrice,
    actualMigrationSqrtPrice,
    curve
  );

const bufferedSwapBase =
  getSwapAmountWithBuffer(
    actualSwapBase,
    sqrtStartPrice,
    curve
  );

const sdkRequiredSupply =
  getTotalSupplyFromCurve(
    migrationQuoteThresholdLamports,
    sqrtStartPrice,
    curve,
    lockedVesting,
    migrationOption,
    leftoverLamports,
    MIGRATION_FEE_PERCENT
  );

const migrationTokens =
  toTokens(migrationBaseAmount);

const curveTokens =
  toTokens(actualSwapBase);

const bufferedCurveTokens =
  toTokens(bufferedSwapBase);

const requiredSupplyTokens =
  toTokens(sdkRequiredSupply);

const residual =
  GENESIS.minus(
    requiredSupplyTokens
  );

const curveDifference =
  curveTokens.minus(
    TARGET_CURVE
  );

const migrationDifference =
  migrationTokens.minus(
    TARGET_MIGRATION
  );

const exactAccounted =
  curveTokens
    .plus(migrationTokens)
    .plus(LEFTOVER);

console.log(
  `Initial market cap:             $${INITIAL_MC.toFixed(12)}`
);

console.log(
  `Migration market cap:           $${SOLVED_MIGRATION_MC.toFixed(12)}`
);

console.log();

console.log(
  `SDK migration percentage:       ${new Decimal(
    percentageSupplyOnMigration
  ).toFixed(15)}%`
);

console.log(
  `Migration quote amount:         ${migrationQuoteAmount.toFixed(12)}`
);

console.log();

console.log(
  `Actual curve tokens:            ${fmt(curveTokens)} RBVR`
);

console.log(
  `Actual migration tokens:        ${fmt(migrationTokens)} RBVR`
);

console.log(
  `Outside-launch leftover:        ${fmt(LEFTOVER)} RBVR`
);

console.log();

console.log(
  `Curve + migration + leftover:   ${fmt(exactAccounted)} RBVR`
);

console.log(
  `SDK required total supply:      ${fmt(requiredSupplyTokens)} RBVR`
);

console.log(
  `Canonical genesis supply:       ${fmt(GENESIS)} RBVR`
);

console.log(
  `Residual / rounding amount:     ${fmt(residual)} RBVR`
);

console.log();

console.log(
  `Curve difference from 400M:     ${fmt(curveDifference)} RBVR`
);

console.log(
  `Migration difference from 210M: ${fmt(migrationDifference)} RBVR`
);

console.log();

const supplyPass =
  sdkRequiredSupply.lte(
    totalSupplyLamports
  );

const noOverrun =
  residual.gte(0);

console.log(
  `FIXED-SUPPLY CHECK:              ${
    supplyPass ? "PASS" : "FAIL"
  }`
);

console.log(
  `NO SUPPLY OVERRUN:               ${
    noOverrun ? "PASS" : "FAIL"
  }`
);

console.log();

if (!supplyPass || !noOverrun) {
  console.log("RESULT: FAIL");
  process.exitCode = 1;
} else {
  console.log("RESULT: PASS");
}

console.log();
console.log(
  "This is an offline SDK accounting validation."
);
console.log(
  "It does not authorize a Mainnet deployment."
);
console.log();
console.log("MAINNET: BLOCKED");
