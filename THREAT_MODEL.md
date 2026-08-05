# Threat Model

## Assets

The principal assets are:

- settlement tokens held by the treasury vault;
- pending allocation balances;
- destination token accounts;
- Reserve surplus;
- protocol configuration;
- upgrade authority;
- deployment keys;
- integrity of cumulative accounting.

## Adversaries

### Unauthorized initializer

Attempts to initialize canonical accounts before the legitimate authority or to reinitialize existing accounts.

### Account-substitution attacker

Supplies a fake protocol state, configuration, treasury, mint, vault, policy, or destination.

### Replay attacker

Repeats a fee-processing, release, or Spillway transaction to obtain duplicate effects.

### Malicious caller

Invokes permissionless instructions with adversarial account ordering or values.

### Compromised operator

Controls an off-chain wallet, deployment machine, RPC credential, or release script.

### Insider

Attempts to redirect Company or Founder allocations, change caps, or bypass locked destinations.

### Economic attacker

Attempts to exploit rounding, cap boundaries, period rollover, Reserve floors, or release timing.

## Attack surfaces

- account validation;
- PDA derivation;
- initialization;
- fee deposit;
- fee processing;
- release instructions;
- Reserve Spillway;
- period rollover;
- arithmetic boundaries;
- upgrade authority;
- off-chain deployment scripts;
- Devnet/Mainnet configuration differences.

## Security properties

### Conservation

Processed settlement-token units must be conserved across accounting buckets.

### Canonical topology

Security-sensitive accounts must match canonical PDAs and expected links.

### Destination immutability

Token movement must use destinations locked in the Execution Configuration.

### Cap enforcement

Company and Founder current-period allocations must not exceed their configured period caps.

### Overflow safety

Company and Founder excess must redirect to Liquidity Growth.

### Replay resistance

A completed accounting or release transition must not be repeatable for duplicate benefit.

### Reserve safety

Spillway deployment must preserve the minimum Reserve floor and enforce cooldown.

### Atomicity

Token transfer and accounting update must succeed or fail together.

## Abuse cases

| Abuse case | Expected defence |
|---|---|
| Fake settlement mint | Mint constraint rejection |
| Fake treasury vault | PDA/link/constraint rejection |
| Fake Reserve Policy | PDA/link rejection |
| Fake Reserve vault | seed or vault validation rejection |
| Fake destination | Execution Configuration mismatch |
| Duplicate destination | Integrity Firewall rejection |
| Repeated processing | no-unprocessed-fees or replay rejection |
| Immediate Spillway replay | cooldown rejection |
| Unauthorized initialization | signer/authority rejection |
| Double initialization | account-already-in-use or explicit rejection |
| Wrong token program | program constraint rejection |

## Residual risks

- undiscovered arithmetic or serialization defects;
- dependency vulnerabilities;
- Solana or Anchor runtime changes;
- compromised upgrade authority;
- misconfigured Mainnet accounts;
- insecure off-chain execution;
- misleading public communications;
- insufficient monitoring;
- absence of external audit.

## Recommended validation

Before Mainnet:

- independent code review;
- reproducible build;
- binary hash verification;
- state-layout review;
- destination ownership confirmation;
- cap-boundary testing;
- period-rollover testing;
- Mainnet dry run using a deployment rehearsal;
- monitoring and alerting;
- authority custody procedure.
