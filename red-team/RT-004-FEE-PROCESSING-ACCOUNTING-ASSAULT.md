# RT-004 — Fee Processing and Accounting Assault

Generated: 2026-08-03 20:34:42 UTC

Git commit before RT-004: `3df61063b89e3973580214d459ccd3299b8da12a`

## Defensive objective

Verify that the fee-processing path cannot:

- process the same vault balance twice;
- inflate lifetime or pending allocations;
- advance the processing epoch after a rejected replay;
- mutate Founder or Company cap accounting after rejection;
- change the settlement-vault balance during accounting processing;
- bypass arithmetic overflow protections;
- lose rounding remainders;
- lose Company or Founder cap overflow;
- violate global allocation conservation.

## Active integration attack

A second `processFees()` call was submitted immediately after the complete
vault balance had already been processed.

Expected result:

- rejection with `NoUnprocessedFees`;
- no TreasuryState mutation;
- no FounderState mutation;
- no CompanyState mutation;
- no settlement-vault token movement;
- all global invariants remain valid.

## Supporting deterministic tests

The existing Rust suite verifies:

- u64 multiplication overflow resistance;
- allocation-total overflow rejection;
- rounding remainder assignment to Reserve;
- Company overflow redirection to Liquidity;
- Founder overflow redirection to Liquidity;
- simultaneous Company and Founder overflow conservation;
- locked allocation conservation.

## Results

- Integration tests: **37 passing**
- Rust tests: **122 passing**
- Strict Clippy: **passing**
- Unauthorized accounting mutation: **none**
- Unauthorized token movement: **none**
- Fee replay inflation: **rejected**

## Verdict

**PASS — fee processing rejects replay and preserves all observed accounting
and token balances after rejection.**
