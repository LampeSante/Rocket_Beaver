# Instruction Reference

Exact account lists and arguments are defined by the generated IDL. This document describes instruction families and security expectations.

## Initialization

### Protocol initialization

Creates canonical protocol state. Requires the intended authority and rejects repeated initialization.

### Protocol configuration initialization

Stores the 30/20/20/20/10 allocation and links it to the canonical protocol.

### Treasury initialization

Creates treasury accounting and links the settlement mint and vault.

### Founder initialization

Creates period-based Founder compensation controls.

### Company initialization

Creates period-based Company allocation controls.

### Execution Configuration initialization

Locks the settlement mint and five token destinations.

### Reserve Policy initialization

Creates the immutable or constrained Reserve surplus policy.

## Deposit

Moves settlement tokens from an authorized source account into the canonical treasury vault.

Expected validation includes signer ownership, source mint, canonical vault, non-zero amount, available balance, and correct token program.

## Process fees

Allocates unprocessed settlement fees into the five accounting buckets. It must conserve value, use locked configuration, enforce caps, redirect capped overflow to Liquidity Growth, and reject replay.

## Release instructions

Separate release paths move pending balances to Reserve, Buyback/Burn, Liquidity, Company, and Founder destinations. Token transfer and accounting update must be atomic.

## Spillway

Deploys eligible Reserve surplus while preserving the Reserve floor, respecting the deployment ratio and cooldown, and using the locked Liquidity destination.

## Failure expectations

The program should reject fake PDAs, wrong owners, wrong discriminators, substituted mints, fake vaults, duplicate destinations, wrong token programs, repeated initialization, replay, empty processing, and cooldown bypass.
