# RT-002B — Account Substitution Assault

## Baseline

- Branch: `red-team`
- Baseline commit: `3df61063b89e3973580214d459ccd3299b8da12a`
- Program ID: `5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3`

## Attacks executed

1. Substituted settlement mint during reserve execution.
2. Attacker-controlled settlement vault during reserve execution.
3. Attacker-controlled reserve destination.
4. System Program substituted for the SPL Token Program.

## Required security property

Every attack must fail without:

- changing treasury account data;
- changing settlement-vault account data;
- reducing the legitimate vault balance;
- increasing an attacker-controlled token balance;
- changing the legitimate reserve destination;
- modifying pending or released reserve accounting.

## Result

All account-substitution attacks were rejected. Treasury state, vault state,
legitimate destination balances, and attacker balances remained unchanged.

## Validation

- Rust tests: 122 passing
- Integration tests: 27 expected passing
- Strict Clippy: passing
- Anchor build: passing

## Disposition

**PASS — No exploitable mint, vault, destination, or token-program substitution path identified in reserve execution.**

## Next phase

RT-002C — PDA, protocol-state, configuration, and linked-account substitution.
