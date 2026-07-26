import { PublicKey } from "@solana/web3.js";

import {
  buildCurveWithMarketCap,
  validateConfigParameters,
  TokenType,
  TokenDecimal,
  TokenAuthorityOption,
  MigrationOption,
  MigrationFeeOption,
  ActivationType,
  CollectFeeMode,
  BaseFeeMode,
  MigratedCollectFeeMode,
  DammV2DynamicFeeMode,
} from "@meteora-ag/dynamic-bonding-curve-sdk";

const INITIAL_MARKET_CAP = 10_000;
const MIGRATION_MARKET_CAP = 36_281.179138321986;

const GENESIS = 1_000_000_000;
const OUTSIDE_DBC = 390_000_000;

const EXPECTED_CURVE = 400_000_000;
const EXPECTED_MIGRATION = 210_000_000;

// ------------------------------------------------------------------
// Canonical RBVR V7 candidate
// ------------------------------------------------------------------

const params = {
  token: {
    tokenType: TokenType.SPLToken,
    tokenBaseDecimal: TokenDecimal.NINE,
    tokenQuoteDecimal: TokenDecimal.NINE,

    // Fixed-supply / immutable authority architecture.
    tokenAuthorityOption: TokenAuthorityOption.Immutable,

    totalTokenSupply: GENESIS,
    leftover: OUTSIDE_DBC,
  },

  fee: {
    /*
     * V7 intentionally validates mechanics before implementing the
     * final Beavernomics revenue-distribution model.
     *
     * Minimum DBC base fee supported by this SDK = 25 bps.
     */
    baseFeeParams: {
      baseFeeMode: BaseFeeMode.FeeSchedulerLinear,
      feeSchedulerParam: {
        startingFeeBps: 25,
        endingFeeBps: 25,
        numberOfPeriod: 0,
        totalDuration: 0,
      },
    },

    dynamicFeeEnabled: false,
    collectFeeMode: CollectFeeMode.QuoteToken,

    // No creator trading-fee allocation in this baseline.
    creatorTradingFeePercentage: 0,

    poolCreationFee: 0,
    enableFirstSwapWithMinFee: false,
  },

  migration: {
    migrationOption: MigrationOption.MET_DAMM_V2,

    /*
     * Use a fixed SDK-supported migration fee option for the baseline.
     * This is NOT the future RBVR transaction/revenue tax.
     */
    migrationFeeOption: MigrationFeeOption.FixedBps25,

    migrationFee: {
      feePercentage: 0,
      creatorFeePercentage: 0,
    },

    /*
     * DAMM v2 requires a valid migrated pool fee.
     * SDK minimum observed: 10 bps.
     */
    migratedPoolFee: {
      collectFeeMode: MigratedCollectFeeMode.QuoteToken,
      dynamicFee: DammV2DynamicFeeMode.Disabled,
      poolFeeBps: 10,
    },
  },

  liquidityDistribution: {
    /*
     * 100% of migrated liquidity remains permanently locked
     * between partner + creator categories.
     *
     * For this mechanical baseline, assign all permanent liquidity
     * to the creator side and zero withdrawable liquidity.
     */
    partnerPermanentLockedLiquidityPercentage: 0,
    partnerLiquidityPercentage: 0,

    creatorPermanentLockedLiquidityPercentage: 100,
    creatorLiquidityPercentage: 0,
  },

  lockedVesting: {
    totalLockedVestingAmount: 0,
    numberOfVestingPeriod: 0,
    cliffUnlockAmount: 0,
    totalVestingDuration: 0,
    cliffDurationFromMigrationTime: 0,
  },

  activationType: ActivationType.Timestamp,

  initialMarketCap: INITIAL_MARKET_CAP,
  migrationMarketCap: MIGRATION_MARKET_CAP,
};

console.log("============================================================");
console.log("ROCKET BEAVER — METEORA DBC FULL CONFIG VALIDATION V7");
console.log("============================================================");
console.log();

console.log("Canonical genesis:          ", GENESIS.toLocaleString(), "RBVR");
console.log("Outside DBC:                ", OUTSIDE_DBC.toLocaleString(), "RBVR");
console.log("Launch inventory:           ", (GENESIS - OUTSIDE_DBC).toLocaleString(), "RBVR");
console.log();

console.log("Target curve:               ", EXPECTED_CURVE.toLocaleString(), "RBVR");
console.log("Target migration:           ", EXPECTED_MIGRATION.toLocaleString(), "RBVR");
console.log();

console.log("Initial market cap:          $", INITIAL_MARKET_CAP);
console.log("Migration market cap:        $", MIGRATION_MARKET_CAP);
console.log();

let config;

try {
  config = buildCurveWithMarketCap(params);

  console.log("BUILD RESULT:                PASS");
} catch (err) {
  console.error("BUILD RESULT:                FAIL");
  console.error();
  console.error(err);
  console.error();
  console.error("MAINNET: BLOCKED");
  process.exit(1);
}

console.log();

try {
  /*
   * buildCurveWithMarketCap() returns ConfigParameters only.
   * validateConfigParameters() also expects leftoverReceiver.
   *
   * TEST-ONLY deterministic address for offline validation.
   * NEVER use this as the production RBVR treasury receiver.
   */
  const TEST_ONLY_LEFTOVER_RECEIVER =
    new PublicKey("DChqykVeh3FHLQcRo4XU9Q9oKZANAFDuXfaEPDfGMh7J");

  const validationConfig = {
    ...config,
    leftoverReceiver: TEST_ONLY_LEFTOVER_RECEIVER,
  };

  const result = validateConfigParameters(validationConfig);

  console.log("VALIDATION RETURN:", result);
  console.log("SDK VALIDATION:              PASS");
} catch (err) {
  console.error("SDK VALIDATION:              FAIL");
  console.error();
  console.error(err);
  console.error();
  console.error("MAINNET: BLOCKED");
  process.exit(1);
}

console.log();
console.log("------------------------------------------------------------");
console.log("GENERATED CONFIG SUMMARY");
console.log("------------------------------------------------------------");

console.log("Token supply (lamports):     ", config.tokenSupply?.preMigrationTokenSupply?.toString());
console.log("Post-migration supply:       ", config.tokenSupply?.postMigrationTokenSupply?.toString());

console.log("Migration option:            ", config.migrationOption);
console.log("Migration fee option:        ", config.migrationFeeOption);
console.log("Migration quote threshold:   ", config.migrationQuoteThreshold?.toString());

console.log("Curve points:                ", config.curve?.length ?? 0);

console.log("Partner locked liquidity %:  ", config.partnerPermanentLockedLiquidityPercentage);
console.log("Partner liquidity %:         ", config.partnerLiquidityPercentage);
console.log("Creator locked liquidity %:  ", config.creatorPermanentLockedLiquidityPercentage);
console.log("Creator liquidity %:         ", config.creatorLiquidityPercentage);

console.log("Creator trading fee %:       ", config.creatorTradingFeePercentage);

console.log();
console.log("------------------------------------------------------------");
console.log("RBVR INVARIANTS");
console.log("------------------------------------------------------------");

const invariantChecks = [
  ["Genesis = 1B", params.token.totalTokenSupply === 1_000_000_000],
  ["Outside DBC = 390M", params.token.leftover === 390_000_000],
  [
    "Launch inventory = 610M",
    params.token.totalTokenSupply - params.token.leftover === 610_000_000,
  ],
  ["Token = SPL", params.token.tokenType === TokenType.SPLToken],
  ["Decimals = 9", params.token.tokenBaseDecimal === TokenDecimal.NINE],
  [
    "Authority = immutable",
    params.token.tokenAuthorityOption === TokenAuthorityOption.Immutable,
  ],
  [
    "Migration = DAMM v2",
    params.migration.migrationOption === MigrationOption.MET_DAMM_V2,
  ],
  ["Token vesting = zero", params.lockedVesting.totalLockedVestingAmount === 0],
  [
    "Withdrawable migrated liquidity = zero",
    params.liquidityDistribution.partnerLiquidityPercentage === 0 &&
      params.liquidityDistribution.creatorLiquidityPercentage === 0,
  ],
  [
    "Permanent migrated liquidity = 100%",
    params.liquidityDistribution.partnerPermanentLockedLiquidityPercentage +
      params.liquidityDistribution.creatorPermanentLockedLiquidityPercentage ===
      100,
  ],
];

let invariantFailure = false;

for (const [name, pass] of invariantChecks) {
  console.log(`${name.padEnd(42)} ${pass ? "PASS" : "FAIL"}`);

  if (!pass) invariantFailure = true;
}

if (invariantFailure) {
  console.error();
  console.error("RBVR INVARIANT CHECK: FAIL");
  console.error("MAINNET: BLOCKED");
  process.exit(1);
}

console.log();
console.log("RBVR INVARIANT CHECK:        PASS");

console.log();
console.log("============================================================");
console.log("RESULT: PASS");
console.log("============================================================");
console.log();
console.log("The canonical RBVR architecture can be constructed by");
console.log("the installed Meteora DBC SDK and passes SDK validation.");
console.log();
console.log("This does NOT yet validate:");
console.log("- exact Devnet transaction execution");
console.log("- pool/config account creation");
console.log("- migration execution");
console.log("- final Beavernomics fee distribution");
console.log("- treasury authority enforcement");
console.log();
console.log("MAINNET: BLOCKED");
