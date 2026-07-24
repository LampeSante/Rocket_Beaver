import BN from "bn.js";
import Decimal from "decimal.js";

import {
  getPercentageSupplyOnMigration,
  getMigrationQuoteAmount,
  getMigrationQuoteThresholdFromMigrationQuoteAmount,
  getSqrtPriceFromPrice,
  getFirstCurve,
  getMigrationThresholdPrice,
  getBaseTokenForSwap,
  getSwapAmountWithBuffer,
  getMigrationBaseToken,
  getTotalSupplyFromCurve,
  getTotalVestingAmount,
  getLockedVestingParams,
  convertToLamports,
  fromDecimalToBN,
  MigrationOption,
} from "@meteora-ag/dynamic-bonding-curve-sdk";

const DECIMALS = 9;

const GENESIS = new Decimal("1000000000");
const LEFTOVER = new Decimal("390000000");

const TARGET_CURVE = new Decimal("400000000");
const TARGET_MIGRATION = new Decimal("210000000");

const INITIAL_MC = new Decimal("10000");

// V3 showed the mathematical target is very close to this region.
const MIGRATION_MCS = [
  36250,
  36280,
  36285,
  36286,
  36287,
  36290,
  36300,
];

const migrationOption = MigrationOption.MET_DAMM_V2;

const lockedVesting = getLockedVestingParams(
  0, // totalLockedVestingAmount
  0, // numberOfVestingPeriod
  0, // cliffUnlockAmount
  0, // totalVestingDuration
  0, // cliffDurationFromMigrationTime
  DECIMALS
);

const totalSupplyLamports =
  convertToLamports(GENESIS.toString(), DECIMALS);

const leftoverLamports =
  convertToLamports(LEFTOVER.toString(), DECIMALS);

function tokens(bn) {
  return new Decimal(bn.toString())
    .div(new Decimal(10).pow(DECIMALS));
}

function fmt(d) {
  return new Decimal(d).toFixed(6);
}

console.log("============================================================");
console.log("ROCKET BEAVER — METEORA DBC ACCOUNTING PROBE V4");
console.log("============================================================");
console.log();
console.log(`Genesis:              ${GENESIS.toFixed()} RBVR`);
console.log(`Outside DBC leftover: ${LEFTOVER.toFixed()} RBVR`);
console.log(`Available launch:     ${GENESIS.minus(LEFTOVER).toFixed()} RBVR`);
console.log(`Target curve:         ${TARGET_CURVE.toFixed()} RBVR`);
console.log(`Target migration:     ${TARGET_MIGRATION.toFixed()} RBVR`);
console.log();

for (const migrationMcRaw of MIGRATION_MCS) {

  console.log("------------------------------------------------------------");
  console.log(`MIGRATION MARKET CAP: $${migrationMcRaw.toLocaleString()}`);

  try {

    const migrationMC = new Decimal(migrationMcRaw);

    const pct = getPercentageSupplyOnMigration(
      INITIAL_MC,
      migrationMC,
      lockedVesting,
      leftoverLamports,
      totalSupplyLamports
    );

    const migrationBaseSupply =
      GENESIS.mul(pct).div(100);

    const migrationQuoteAmount =
      getMigrationQuoteAmount(
        migrationMC,
        new Decimal(pct)
      );

    // Zero migration fee for this accounting baseline.
    const migrationQuoteThreshold =
      getMigrationQuoteThresholdFromMigrationQuoteAmount(
        migrationQuoteAmount,
        new Decimal(0)
      );

    const migrationPrice =
      migrationQuoteAmount.div(migrationBaseSupply);

    const migrationSqrtPrice =
      getSqrtPriceFromPrice(
        migrationPrice.toString(),
        DECIMALS,
        9
      );

    const migrationQuoteThresholdLamports =
      convertToLamports(
        migrationQuoteThreshold.toString(),
        9
      );

    const migrationQuoteAmountLamports =
      fromDecimalToBN(
        migrationQuoteAmount.mul(
          new Decimal(10).pow(9)
        )
      );

    const migrationBaseAmount =
      getMigrationBaseToken(
        migrationQuoteAmountLamports,
        migrationSqrtPrice,
        migrationOption
      );

    const vestingAmount =
      getTotalVestingAmount(lockedVesting);

    const initialSwapAmount =
      totalSupplyLamports
        .sub(migrationBaseAmount)
        .sub(vestingAmount)
        .sub(leftoverLamports);

    const {
      sqrtStartPrice,
      curve
    } = getFirstCurve(
      migrationSqrtPrice,
      migrationBaseAmount,
      initialSwapAmount,
      migrationQuoteThresholdLamports,
      0
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

    const sdkTotalDynamic =
      getTotalSupplyFromCurve(
        migrationQuoteThresholdLamports,
        sqrtStartPrice,
        curve,
        lockedVesting,
        migrationOption,
        leftoverLamports,
        0
      );

    const actualSwapTokens = tokens(actualSwapBase);
    const bufferedSwapTokens = tokens(bufferedSwapBase);
    const migrationTokens = tokens(migrationBaseAmount);
    const sdkTotalTokens = tokens(sdkTotalDynamic);

    const bufferTokens =
      bufferedSwapTokens.minus(actualSwapTokens);

    const launchRequired =
      bufferedSwapTokens.plus(migrationTokens);

    const remaining =
      GENESIS.minus(sdkTotalTokens);

    console.log(`Migration %:          ${pct.toFixed(12)}%`);
    console.log(`Migration base:       ${fmt(migrationTokens)} RBVR`);
    console.log(`Actual curve swap:    ${fmt(actualSwapTokens)} RBVR`);
    console.log(`Swap buffer:          ${fmt(bufferTokens)} RBVR`);
    console.log(`Buffered curve need:  ${fmt(bufferedSwapTokens)} RBVR`);
    console.log();
    console.log(`Curve + migration:    ${fmt(launchRequired)} RBVR`);
    console.log(`Leftover:             ${fmt(LEFTOVER)} RBVR`);
    console.log(`SDK total required:   ${fmt(sdkTotalTokens)} RBVR`);
    console.log(`Remaining to 1B:      ${fmt(remaining)} RBVR`);
    console.log();

    const curveDiff =
      bufferedSwapTokens.minus(TARGET_CURVE);

    const migrationDiff =
      migrationTokens.minus(TARGET_MIGRATION);

    console.log(`vs 400M curve target: ${fmt(curveDiff)} RBVR`);
    console.log(`vs 210M migr target:  ${fmt(migrationDiff)} RBVR`);

    if (sdkTotalTokens.lte(GENESIS)) {
      console.log("SUPPLY CHECK:          PASS");
    } else {
      console.log("SUPPLY CHECK:          FAIL — exceeds 1B");
    }

  } catch (e) {
    console.log("ERROR:");
    console.log(e?.stack ?? e);
  }
}

console.log();
console.log("============================================================");
console.log("MAINNET: BLOCKED");
console.log("============================================================");
