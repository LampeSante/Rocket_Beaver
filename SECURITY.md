# Security Policy

## Security objective

RBVR aims to minimize discretionary trust and make treasury behaviour predictable, constrained, and independently verifiable.

## Implemented security themes

- canonical PDAs;
- locked destinations;
- immutable or disabled updates where intended;
- one-time initialization;
- account ownership validation;
- account discriminator validation;
- settlement mint validation;
- treasury vault validation;
- replay protection;
- no-unprocessed-fee protection;
- period caps;
- overflow redirection;
- Reserve floor and cooldown enforcement;
- deterministic accounting;
- persistent Devnet verification.

## Testing evidence

The project history includes:

- Rust unit tests;
- Anchor integration tests;
- red-team cases in the RT-001 through RT-005 families;
- release and authority tests;
- Reserve and Spillway tests;
- invariant mapping;
- fuzz targets;
- persistent Devnet account verification.

Passing tests reduce risk but do not prove the absence of vulnerabilities.

## Responsible disclosure

Do not publish suspected vulnerabilities before maintainers have had a reasonable opportunity to investigate.

A production repository should provide a dedicated private reporting channel. Until one is configured, avoid including sensitive exploit details in public GitHub issues.

## Supported versions

Only the most recent explicitly supported release should be assumed to receive fixes. Tags represent milestones, not automatic production support guarantees.

## Upgrade authority

The verified Devnet program remained upgradeable. Upgrade authority removal is irreversible and should occur only after:

- final binary verification;
- independent security review;
- complete deployment evidence;
- Mainnet configuration review;
- destination ownership review;
- operational readiness;
- emergency-response planning;
- explicit governance decision.

## Key management

Never commit:

- deployment-authority keypairs;
- payer keypairs;
- mint keypairs;
- local-validator keypairs;
- seed phrases;
- API tokens;
- private RPC credentials.

The repository ignores local key material, but operators remain responsible for secure storage, backup, rotation, and access control.

## Threat assumptions

RBVR assumes:

- Solana consensus and runtime behave correctly;
- the SPL Token program behaves according to specification;
- configured token accounts represent the intended destinations;
- deployment and upgrade keys are controlled securely;
- off-chain operators do not misrepresent protocol behaviour;
- users verify the program ID and deployment environment.

## Security boundaries

On-chain guarantees do not automatically secure:

- frontend code;
- exchange integrations;
- liquidity venues;
- off-chain buyback executors;
- RPC providers;
- operator workstations;
- social-media accounts;
- private keys;
- governance processes.

## Review priorities

Security reviewers should prioritize:

1. seed derivation and PDA substitution;
2. initialization and reinitialization;
3. authority checks;
4. mint and vault substitution;
5. destination uniqueness and locking;
6. fee conservation;
7. period rollover;
8. fixed Founder rate and annual-cap enforcement;
9. Company and Founder overflow;
10. Reserve floor arithmetic;
11. cooldown and replay protection;
12. upgrade-authority handling.

## Incident response

If a vulnerability is suspected:

1. preserve logs and transaction signatures;
2. determine whether the issue affects Devnet, Mainnet, or only tooling;
3. avoid destructive changes before evidence is captured;
4. review upgrade-authority options;
5. disable affected off-chain automation;
6. communicate facts without speculation;
7. produce a post-incident report after remediation.
