# RBVR Devnet Readiness Preflight

Generated: **2026-08-03 22:03:41 UTC**

## Source checkpoint

- Branch: `red-team`
- Commit: `a89e8b7f5f4f323e95165f37507526112369fc84`
- Program ID: `5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3`
- Expected checkpoint: `a89e8b7`
- Spillway fuzz status: **RUNNING**
- Spillway fuzz PID: `101170`

## Build artifact

- File: `target/deploy/treasury_router.so`
- Size: **716808 bytes**
- SHA-256: `4ad303700259a14fd208e5d5ed3f996b1231694af699fcf3f99764690208b05c`

## Devnet wallet

- Wallet: `4bGnfSSGKBbPhTeKbwxRDADnrFNhwX6FM8nwng2TBv1C`
- Balance: **4.929858583 SOL**

## Verification summary

| Result | Count |
|---|---:|
| Pass | 15 |
| Warning | 0 |
| Failure | 0 |

## Deployment decision

**TECHNICALLY READY, FUZZ CAMPAIGN STILL RUNNING**

The build and devnet environment passed the current checks. Deployment remains
intentionally blocked until the eight-hour Spillway fuzz campaign completes
without crash, timeout, OOM, or invariant-failure artifacts.
