import fs from "node:fs";

const model = JSON.parse(
  fs.readFileSync("config/meteora/dbc-model-v1.json", "utf8")
);

const supply = model.token.fixed_initial_supply;
const targets = model.launch_targets;

const allocations = {
  community_curve: targets.working_curve_tokens,
  initial_amm: targets.working_initial_amm_tokens,
  future_liquidity: targets.locked_future_liquidity_tokens,
  community_initiatives: targets.community_initiatives_tokens
};

const total = Object.values(allocations)
  .reduce((sum, value) => sum + value, 0);

const openingFdv =
  supply * targets.target_opening_price_usd;

const errors = [];

if (total !== supply) {
  errors.push(
    `Allocation total ${total} does not equal fixed supply ${supply}`
  );
}

if (model.constraints.creator_token_allocation !== 0) {
  errors.push("Creator token allocation must remain zero.");
}

if (model.constraints.team_token_allocation !== 0) {
  errors.push("Team token allocation must remain zero.");
}

console.log("====================================================");
console.log("ROCKET BEAVER — METEORA DBC MODEL V1");
console.log("====================================================");

console.log();
console.log("Fixed initial supply:");
console.log(`${supply.toLocaleString()} RBVR`);

console.log();
console.log("Working allocation:");

for (const [name, amount] of Object.entries(allocations)) {
  const pct = (amount / supply) * 100;

  console.log(
    `${name.padEnd(24)} ${amount.toLocaleString().padStart(15)}  ${pct.toFixed(2)}%`
  );
}

console.log();
console.log(`Total allocated: ${total.toLocaleString()} RBVR`);

console.log();
console.log(
  `Target opening price: $${targets.target_opening_price_usd}`
);

console.log(
  `Implied opening FDV:  $${openingFdv.toLocaleString()}`
);

console.log(
  `Working quote threshold: $${model.migration.working_quote_threshold_usd.toLocaleString()}`
);

console.log(
  `Migration fee assumption: ${model.migration.protocol_liquidity_migration_fee_percent}%`
);

console.log();

if (errors.length) {
  console.log("RESULT: FAIL");

  for (const error of errors) {
    console.log(`- ${error}`);
  }

  process.exit(1);
}

console.log("RESULT: PASS");
console.log("Economic targets are internally consistent.");
console.log("Meteora curve mechanics still require SDK validation.");
console.log();
console.log("MAINNET: BLOCKED");
