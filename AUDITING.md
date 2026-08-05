# Auditor Guide

## Scope

The primary scope is the Anchor program under `programs/treasury-router/`.

Operational scripts and tests are supporting evidence but should be reviewed separately.

## High-value review targets

1. initialization;
2. PDA derivation;
3. account links;
4. fee conservation;
5. settlement vault authority;
6. destination locking;
7. fixed Founder rate and annual-cap enforcement;
8. Company and Founder caps;
9. overflow to Liquidity Growth;
10. release atomicity;
11. Reserve policy;
12. Spillway cooldown and floor;
13. replay protection;
14. upgrade authority.

## Suggested audit sequence

### Phase 1: model reconstruction

Reconstruct accounts, seeds, instructions, authorities, state transitions, and token flows independently.

### Phase 2: invariant review

Validate:

```text
received = unprocessed + allocated
allocated = sum(lifetime bucket allocations)
pending + released = lifetime allocation per bucket
sum(bucket allocations) = total allocated
```

Account for overflow redirection.

### Phase 3: adversarial accounts

Attempt:

- fake PDAs;
- correct discriminator under wrong owner;
- wrong mint;
- wrong token program;
- swapped accounts;
- duplicate destinations;
- default public keys;
- stale bumps;
- mismatched protocol links.

### Phase 4: arithmetic

Review:

- basis points;
- multiplication before division;
- overflow;
- underflow;
- rounding residue;
- cap remaining;
- rollover;
- marginal tiers;
- timestamps;
- cooldown.

### Phase 5: authority

Map every signer and authority. Confirm that no release path depends on hidden discretionary authority.

### Phase 6: deployment

Compare:

- source commit;
- built binary;
- deployed program;
- IDL;
- on-chain state;
- configuration values.

## Verified Devnet reference

Program ID:

```text
5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3
```

Use Devnet only as supporting evidence. A future Mainnet deployment will require separate verification.

## Known documentation caveat

Some ProtocolState fields decoded to the default public key on Devnet. Auditors should determine from source whether they are unused placeholders, reserved compatibility fields, or missing initialization.

## Deliverables requested from auditors

- findings by severity;
- exploit preconditions;
- affected instructions;
- proof of concept;
- remediation recommendation;
- regression test;
- residual risk;
- deployment-specific observations.
