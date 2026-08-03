# RT-003 — Deposit and Token-Authority Assault

## Objective

Attack the settlement deposit path by supplying hostile token accounts,
incorrect authorities, incorrect mints, substituted vaults, and invalid
transfer amounts.

## Attacks executed

1. Source token account owned by an attacker while another signer is supplied.
2. Source token account associated with a non-settlement mint.
3. Valid SPL token account substituted for the canonical treasury vault.
4. Zero-value deposit.
5. Deposit amount exceeding the source token balance.

## Expected security behavior

Every hostile deposit must:

- be rejected before treasury accounting mutation;
- transfer no settlement assets;
- leave the canonical vault unchanged;
- leave attacker-controlled accounts unchanged;
- preserve global treasury invariants.

## Result

- Hostile deposit attempts: **5**
- Hostile deposits rejected: **5**
- Unauthorized token movement: **none**
- Treasury accounting mutation after rejection: **none**
- Integration tests: **36 passing**
- Rust tests: **122 passing**
- Strict Clippy: **passing**

## Conclusion

The tested deposit path rejected token-owner confusion, mint substitution,
vault substitution, zero-value transfers, and insufficient-balance attempts
without unauthorized asset movement or accounting mutation.

This result applies to the specific RT-003 cases executed. It does not replace
additional adversarial testing, fuzzing, devnet validation, or independent
security review.

## Next phase

**RT-004 — Fee-processing and accounting assault**

Planned cases:

- repeated processing without new settlement assets;
- vault/accounting balance mismatch;
- malformed bucket accounting;
- company and founder cap-boundary manipulation;
- arithmetic maximums;
- processing epoch consistency;
- failed processing must leave all state unchanged.
