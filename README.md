# Rocket Beaver (RBVR)

Rocket Beaver is a security-first treasury routing protocol for Solana, implemented with Rust, Anchor, and SPL Token primitives.

Current milestone: **`v1.1.0-devnet-verified`**

## Treasury allocation

| Destination | Allocation |
|---|---:|
| Reserve | 30% |
| Buyback/Burn | 20% |
| Liquidity Growth | 20% |
| Company/Operations | 20% |
| Founder Compensation | 10% |

Company and Founder controls are period-based. When either period cap is reached, overflow is redirected to Liquidity Growth. Founder compensation is period-capped and includes a `current_tier` state field. The current source audit did not verify an implemented progressive marginal-rate calculation, so no declining-rate schedule is claimed here.

## Security model

RBVR is built around deterministic arithmetic, canonical PDAs, locked destinations, one-time initialization, replay resistance, account-link validation, cap enforcement, overflow redirection, constrained Reserve deployment, and independently verifiable on-chain state.

## Verified Devnet program

```text
5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3
```

The persistent Devnet verification suite confirmed that the program is executable; canonical state accounts exist and are program-owned; account discriminators match the IDL; the allocation configuration decodes to 30/20/20/20/10; updates are disabled in the verified configuration; and the Execution Configuration locks the settlement mint and five destinations.

## Build and test

```bash
npm install
anchor build
anchor test
```

Persistent Devnet verification:

```bash
export ANCHOR_PROVIDER_URL="https://api.devnet.solana.com"
export ANCHOR_WALLET="$HOME/path/to/verifier-wallet.json"
npm run test:devnet:verify
```

Do not run the fresh-state integration suite against an already initialized persistent deployment.

## Documentation

See [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) for the complete documentation map.

## Important limitation

Devnet verification, internal red-team testing, and fuzzing are not substitutes for an independent external security review. Mainnet deployment requires separate configuration validation, operational controls, and release approval.

## License

No license is selected by this documentation pack. Add one deliberately before inviting external reuse.
