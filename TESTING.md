# Testing and Verification

## Test layers

### Rust unit tests

Validate pure logic and internal engines.

```bash
cargo test
```

### Anchor integration tests

Validate instructions, accounts, token movement, and error behaviour on fresh state.

```bash
anchor test
```

### Red-team tests

Exercise adversarial cases including:

- unauthorized initialization;
- fake mints;
- fake vaults;
- substituted destinations;
- swapped PDAs;
- replay;
- Reserve policy substitution;
- Spillway destination substitution;
- cooldown bypass.

### Fuzz testing

Fuzz targets should focus on invariant-preserving state transitions:

- allocation conservation;
- cap boundaries;
- progressive Founder tiers;
- overflow redirection;
- Reserve floor;
- cooldown;
- arithmetic limits;
- replay.

### Persistent Devnet verification

Reads deployed state without initialization or mutation:

```bash
npm run test:devnet:verify
```

## Fresh-state versus persistent-state tests

The main integration suite is not idempotent on a persistent cluster because initialization is intentionally one-time.

Use:

- local validator for complete fresh-state tests;
- dedicated Devnet verification suite for persistent deployment checks.

## Interpreting failures

A test failure may be:

1. protocol logic failure;
2. test assumption failure;
3. RPC rate limit;
4. faucet failure;
5. persistent-state collision;
6. dependency or toolchain issue.

For example, a Devnet faucet `429` during attacker-wallet funding is an infrastructure failure unless the protocol transaction itself was submitted and rejected incorrectly.

## Required release gates

A release candidate should pass:

- formatting;
- linting;
- Rust tests;
- integration tests;
- targeted red-team tests;
- fuzz regression;
- architecture validation;
- secret scan;
- Devnet verification;
- clean Git status.

## Test evidence

Save:

- command;
- environment;
- commit;
- timestamps;
- pass/fail counts;
- transaction signatures;
- relevant logs;
- fuzz corpus and crash artifacts;
- toolchain versions.

## Non-goals

Tests do not establish:

- economic profitability;
- legal compliance;
- token price performance;
- absence of all vulnerabilities;
- security of external integrations.
