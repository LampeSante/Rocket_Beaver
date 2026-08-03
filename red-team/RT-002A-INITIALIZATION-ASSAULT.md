# RT-002A — Initialization Assault

## Baseline

- Branch: `red-team`
- Baseline commit: `3df61063b89e3973580214d459ccd3299b8da12a`
- Program ID: `5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3`

## Attacks executed

1. Unauthorized signer attempted to initialize canonical protocol configuration.
2. Repeated initialization of canonical protocol configuration.
3. Repeated initialization of canonical treasury and settlement vault.
4. Repeated initialization of canonical founder state.
5. Repeated initialization of canonical company state.
6. Repeated initialization of canonical execution configuration.

## Required security property

Every malicious initialization transaction must fail without:

- creating an unauthorized account;
- modifying protocol state;
- modifying configuration;
- modifying treasury state;
- modifying the settlement vault;
- modifying founder or company state;
- changing execution destinations.

## Result

All RT-002A attacks were rejected and all compared account data remained unchanged.

## Test baseline

- Rust tests: 122 passing
- Integration tests: 23 expected passing
- Strict Clippy: passing
- Anchor build: passing

## Disposition

**PASS — No exploitable canonical reinitialization or unauthorized protocol-config initialization path identified.**

## Next phase

RT-002B — PDA, mint, vault, owner, and account-substitution assault.
