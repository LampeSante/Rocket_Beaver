# RT-005 — Hostile Spillway Assault

Generated: 2026-08-03 21:47:18 UTC

Baseline commit: `9f76f504a8019780ccc585f66ef621816bdefab5`

## Defensive objective

Prove that the permissionless Spillway rejects hostile account substitution
and immediate replay without transferring tokens or mutating Reserve Policy
state.

## Live attacks

1. Substituted Reserve Policy PDA.
2. Substituted Reserve Vault.
3. Substituted Liquidity Growth destination.
4. Immediate replay during the immutable deployment cooldown.

## Required rejection invariants

Every rejected transaction preserved:

- canonical Reserve Vault token balance;
- immutable Liquidity Growth destination token balance;
- Reserve Policy `last_deployed_at`;
- Reserve Policy `lifetime_deployed`;
- substituted destination balances where applicable.

## Supporting deterministic coverage

Existing Reserve Engine and Reserve Policy tests cover:

- reserve below floor;
- reserve exactly at floor;
- zero deployable surplus;
- maximum-value arithmetic;
- basis-point validation;
- floor preservation across the rate matrix;
- cooldown arithmetic;
- lifetime deployment overflow;
- failed state transition immutability.

## Results

- Rust tests: **145 passing**
- Integration tests: **42 passing**
- Live hostile Spillway attacks: **4 passing**
- Unauthorized Reserve transfer: **none**
- Unauthorized destination transfer: **none**
- Rejected-call policy mutation: **none**

## Verdict

**PASS — Spillway rejects hostile account substitution and cooldown replay
without token movement or policy mutation.**
