# CI/CD Guide

## Goals

Continuous integration should validate every pull request without requiring deployment keys.

## Recommended checks

- Rust formatting;
- Rust tests;
- Anchor build;
- TypeScript formatting;
- TypeScript integration tests on local validator;
- architecture validation;
- secret scanning;
- generated-file policy.

## Devnet jobs

Persistent Devnet verification should be:

- manual or scheduled;
- read-only;
- configured with GitHub environment protections;
- run without exposing private keys in logs.

Avoid storing a high-value deployment authority in ordinary repository secrets. Prefer a dedicated low-value verifier wallet where signing is needed.

## Release workflow

A release workflow may:

1. validate tag;
2. build;
3. record toolchain;
4. hash binary and IDL;
5. create release notes;
6. attach non-sensitive evidence.

Deployment should remain a protected manual action.
