import Decimal from "decimal.js";
import BN from "bn.js";

import {
  getPercentageSupplyOnMigration,
  getMigrationQuoteAmount,
} from "@meteora-ag/dynamic-bonding-curve-sdk";

Decimal.set({ precision: 50 });

const GENESIS = new BN("1000000000");
const LEFTOVER = new BN("390000000");

const INITIAL_MC = new Decimal("10000");

const TARGET_MIGRATION_PERCENT = new Decimal("21");
const TARGET_MIGRATION_TOKENS = new Decimal("210000000");
const TARGET_CURVE_TOKENS = new Decimal("400000000");

const noLockedVesting = {
  cliffUnlockAmount: new BN(0),
  amountPerPeriod: new BN(0),
  numberOfPeriod: new BN(0),
  cliffDurationFromMigrationTime: new BN(0),
  frequency: new BN(0),
};

function pctAt(mc) {
  return new Decimal(
    getPercentageSupplyOnMigration(
      INITIAL_MC,
      new Decimal(mc),
      noLockedVesting,
      LEFTOVER,
      GENESIS
    )
  );
}

// Binary search for the market cap where SDK migration allocation = 21%.
let low = new Decimal("36000");
let high = new Decimal("36500");

for (let i = 0; i < 150; i++) {
  const mid = low.plus(high).div(2);
  const pct = pctAt(mid);

  // Migration percentage falls as migration MC rises.
  if (pct.gt(TARGET_MIGRATION_PERCENT)) {
    low = mid;
  } else {
    high = mid;
  }
}

const solvedMC = low.plus(high).div(2);
const solvedPct = pctAt(solvedMC);

const migrationTokens =
  new Decimal(GENESIS.toString())
    .mul(solvedPct)
    .div(100);

const launchInventory =
  new Decimal(GENESIS.toString())
    .minus(new Decimal(LEFTOVER.toString()));

const theoreticalCurve =
  launchInventory.minus(migrationTokens);

const migrationQuote =
  getMigrationQuoteAmount(
    solvedMC,
    solvedPct
  );

console.log("============================================================");
console.log("ROCKET BEAVER — EXACT DBC TARGET SOLVER V5");
console.log("============================================================");
console.log();

console.log(`Initial market cap:          $${INITIAL_MC.toFixed(12)}`);
console.log(`Solved migration market cap: $${solvedMC.toFixed(12)}`);
console.log();

console.log(`SDK migration percentage:    ${solvedPct.toFixed(15)}%`);
console.log(`Migration tokens:            ${migrationTokens.toFixed(9)} RBVR`);
console.log(`Theoretical curve tokens:    ${theoreticalCurve.toFixed(9)} RBVR`);
console.log();

console.log(`Target migration:            ${TARGET_MIGRATION_TOKENS.toFixed()} RBVR`);
console.log(`Target curve:                ${TARGET_CURVE_TOKENS.toFixed()} RBVR`);

console.log();
console.log(`Migration difference:        ${migrationTokens.minus(TARGET_MIGRATION_TOKENS).toFixed(9)} RBVR`);
console.log(`Curve difference:            ${theoreticalCurve.minus(TARGET_CURVE_TOKENS).toFixed(9)} RBVR`);

console.log();
console.log(`Migration quote amount:      ${migrationQuote.toFixed(12)}`);

console.log();
console.log("IMPORTANT:");
console.log("This solves the mathematical 21% migration target.");
console.log("It does NOT yet prove exact integer-lamport accounting after");
console.log("curve construction, swap-buffer capping, and SDK rounding.");

console.log();
console.log("MAINNET: BLOCKED");
