# Deployment Guide

## Environments

RBVR development may use:

- local validator;
- Solana Devnet;
- Solana Mainnet Beta.

Never assume configuration carries safely between environments.

## Required tooling

- Rust toolchain;
- Solana CLI;
- Anchor CLI;
- Node.js and npm;
- repository dependencies installed with `npm install`.

## Environment variables

Prefer environment variables over hard-coded local paths:

```bash
export ANCHOR_PROVIDER_URL="https://api.devnet.solana.com"
export ANCHOR_WALLET="$HOME/path/to/deployment-authority.json"
```

Do not commit the wallet file.

## Local build

```bash
anchor build
```

Confirm the generated program ID matches:

- `declare_id!`;
- `Anchor.toml`;
- generated IDL and types;
- intended deployment keypair.

## Local testing

```bash
anchor test
```

The main integration suite expects fresh state.

## Devnet preflight

Before deployment:

1. confirm RPC;
2. confirm wallet path;
3. confirm wallet address;
4. confirm SOL balance;
5. confirm program ID;
6. confirm Git branch and commit;
7. confirm working tree;
8. run build and local tests;
9. archive evidence.

## Devnet deployment

Use the reviewed deployment script under `scripts/deploy/`.

After deployment:

```bash
solana program show \
  5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3 \
  --url devnet
```

Record:

- program ID;
- ProgramData address;
- upgrade authority;
- deployment slot;
- binary size;
- transaction signature.

## Persistent Devnet verification

```bash
export ANCHOR_PROVIDER_URL="https://api.devnet.solana.com"
export ANCHOR_WALLET="$HOME/path/to/deployment-authority.json"
npm run test:devnet:verify
```

This suite is read-only and verifies existing state.

## Important warning

Do not run plain:

```bash
anchor test --provider.cluster devnet
```

against the persistent deployment unless you intentionally want Anchor to deploy/upgrade and execute fresh-state tests. The integration suite may attempt one-time initialization and fail against existing PDAs.

## Deployment evidence

Archive:

- `solana config get`;
- deployer address and balance;
- `solana program show`;
- Git commit and status;
- IDL;
- binary hash;
- transaction signatures;
- decoded protocol state;
- verifier results;
- authority map.

## Mainnet deployment

Mainnet deployment requires a separate checklist. At minimum:

- independent security review completed;
- all destination accounts created and verified;
- settlement mint confirmed;
- caps and periods approved;
- Reserve policy approved;
- upgrade authority custody approved;
- release scripts reviewed;
- RPC redundancy established;
- monitoring enabled;
- incident-response contacts assigned;
- exact binary reproduced and hashed.

## Rollback

A Solana upgrade cannot simply erase already-created state. Rollback planning must consider:

- program upgrade to a previously reviewed binary;
- state compatibility;
- temporary off-chain suspension;
- token-account controls;
- authority availability;
- communications.

## Immutability

Do not revoke upgrade authority as a ceremonial milestone. Revoke only when the deployed implementation and state are final, reviewed, reproducible, and operationally supported.
