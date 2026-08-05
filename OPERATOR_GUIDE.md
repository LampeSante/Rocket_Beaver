# Operator Guide

## Daily responsibilities

- monitor program and treasury accounts;
- monitor failed transactions;
- monitor Reserve and pending buckets;
- maintain RPC access;
- protect operational wallets;
- archive significant transaction signatures;
- verify public communications.

## Safe command pattern

Always confirm environment first:

```bash
solana config get
solana address
solana balance
```

Then explicitly pass cluster where practical.

## Persistent verification

Run:

```bash
export ANCHOR_PROVIDER_URL="https://api.devnet.solana.com"
export ANCHOR_WALLET="$HOME/path/to/wallet.json"
npm run test:devnet:verify
```

## Never do casually

- run fresh-state integration tests against persistent deployment;
- expose keypair JSON;
- revoke upgrade authority;
- change destination accounts;
- force-push shared history;
- deploy an uncommitted build;
- assume Devnet values are suitable for Mainnet.

## Evidence collection

For every deployment or upgrade record:

- operator;
- timestamp;
- commit;
- binary hash;
- program ID;
- transaction signature;
- authority;
- before/after state;
- verifier result.

## Incident triage

Classify incidents as:

- chain/RPC;
- test infrastructure;
- deployment;
- state/configuration;
- token movement;
- key compromise;
- public communication.

Do not call an infrastructure error a protocol failure without evidence.
