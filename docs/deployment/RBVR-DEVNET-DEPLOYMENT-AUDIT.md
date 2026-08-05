# RBVR Devnet Deployment Audit

Generated: **2026-07-30 21:01:03 UTC**

- Program ID: `5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3`
- Cluster: **Solana devnet**
- RPC: `https://api.devnet.solana.com`

## Local environment

```text
solana-cli 3.1.10 (src:7bc9c805; feat:1620780344, client:Agave)

Config File: /home/alec_elliott/.config/solana/cli/config.yml
RPC URL: http://127.0.0.1:8899 
WebSocket URL: ws://127.0.0.1:8900/ (computed)
Keypair Path: /home/alec_elliott/rocket-beaver-dev/keys/local-deployment.json 
Commitment: confirmed 

Active wallet: 4bGnfSSGKBbPhTeKbwxRDADnrFNhwX6FM8nwng2TBv1C
Devnet balance: 4.929858583 SOL
Devnet genesis hash: EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG
```

## Program information

```text
Error: Unable to find the account 5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3
```

## Deployment conclusion

The configured RBVR program ID was not found on devnet.

No upgrade-authority, ProgramData, or on-chain state verification
can be completed until a devnet deployment exists.

## Concurrent fuzzing

```text
  76408       10:42  0.0  0.0 /home/alec_elliott/.cargo/bin/cargo-fuzz fuzz run process_release fuzz/corpus/process_release -- -max_total_time=28800 -timeout=10 -rss_limit_mb=4096 -max_len=512 -print_final_stats=1
```

## Audit summary

| Result | Count |
|---|---:|
| Pass | 2 |
| Warning | 1 |
| Failure | 0 |

- Deployment status: **NOT DEPLOYED**
- Upgrade-authority status: **NOT_DEPLOYED**

## Safety note

This audit performed read-only RPC and local filesystem operations.
It did not deploy, upgrade, initialize, transfer, or revoke authority.
