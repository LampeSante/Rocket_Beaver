# Developer Guide

## Setup

```bash
git clone https://github.com/LampeSante/Rocket_Beaver.git
cd Rocket_Beaver
npm install
anchor build
anchor test
```

## Branch model

The documented development branch is `red-team`. Establish a stable default branch before external collaboration.

Recommended branches:

- `main`: reviewed release state;
- `develop`: active integration;
- feature branches;
- security branches for embargoed fixes.

## Code locations

- `programs/treasury-router/src/lib.rs`: program entry;
- `instructions/`: instruction handlers and account contexts;
- `state/`: account layouts;
- `engines/`: deterministic business and integrity logic;
- `events/`: emitted events;
- `errors/`: program errors;
- `tests/`: integration and Devnet verification;
- `scripts/`: reusable operations;
- `tools/`: one-off migration and recovery utilities.

## Development rules

- preserve deterministic integer arithmetic;
- never introduce caller-controlled destinations;
- derive canonical PDAs from constants;
- update account lengths when layouts change;
- preserve reserved bytes where possible;
- add explicit errors;
- test failure without mutation;
- document authority changes;
- distinguish on-chain, off-chain, and documentation work.

## Adding an instruction

1. define the account context;
2. validate ownership, seeds, links, signer, and programs;
3. implement deterministic handler logic;
4. emit events where useful;
5. add errors;
6. expose from module and `lib.rs`;
7. regenerate IDL;
8. add positive and adversarial tests;
9. update documentation.

## Changing account layouts

Account layout changes can break deployed state.

Before changing:

- review serialization compatibility;
- update `LEN`;
- consider versioning;
- plan migration;
- add backward-compatibility tests;
- document Mainnet impact.

## Local paths

Do not hard-code developer-specific paths. Use:

```bash
RBVR_WALLET="${RBVR_WALLET:-$HOME/.config/solana/id.json}"
```

## Logging

Logs should help operators identify:

- instruction;
- invariant failure;
- bucket;
- amount;
- epoch;
- destination;
- policy condition.

Avoid logging secrets.

## Pull requests

A protocol PR should include:

- problem statement;
- security impact;
- state-layout impact;
- authority impact;
- tests;
- migration impact;
- documentation updates.
