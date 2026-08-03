# RT-002C — Linked-Account and PDA Substitution Assault

## Objective

Attempt to bypass canonical PDA, account-type, and protocol-linkage
constraints by supplying valid but incorrect RBVR accounts in place of the
accounts required by critical instructions.

## Attacks executed

1. Supplied `execution_config` as `protocol_config` to `process_fees`.
2. Swapped the canonical Founder and Company state PDAs.
3. Supplied `protocol_config` as `execution_config` to Reserve execution.
4. Supplied `execution_config` as the canonical `protocol_state`.

## Expected security behavior

Every hostile substitution must:

- be rejected before protocol mutation;
- produce no settlement-token movement;
- leave all treasury accounting unchanged;
- leave processing epoch unchanged;
- preserve all global accounting invariants.

## Result

- Malicious substitutions attempted: **4**
- Malicious substitutions rejected: **4**
- Unauthorized token movement: **none**
- Treasury mutation after rejection: **none**
- Global accounting invariant failures: **none**
- Integration tests: **31 passing**
- Rust tests: **122 passing**
- Strict Clippy: **passing**

## Conclusion

The tested instructions rejected linked-account and PDA substitution attacks
without altering treasury state or transferring settlement assets.

This result covers the specific attack cases executed in RT-002C. It does not
replace additional adversarial testing, fuzzing, or independent review.

## Next phase

**RT-003 — Deposit and token-authority assault**

Planned cases:

- source token account owned by the wrong signer;
- source account using the wrong settlement mint;
- canonical vault substitution;
- zero-value deposit;
- balance-underflow attempt;
- maximum-value transfer attempt;
- signer/account-owner confusion;
- repeated deposit accounting consistency.
