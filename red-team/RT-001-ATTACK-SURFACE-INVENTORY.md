# RT-001 — RBVR Attack Surface Inventory

Generated: **2026-08-03 20:01:26 UTC**

Program ID: `5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3`

Git commit: `988930c6d6b17ebe791e2bf1ff3c793420dd185d`

## Mission

Map every externally reachable instruction, account, PDA, signer, CPI,
token transfer, arithmetic operation, and state-mutation surface before
attempting active exploitation.

## Executive summary

- Public instruction entrypoints: **13**
- Anchor account fields: **107**
- PDA-constrained account fields: **48**
- Signer fields: **7**
- Writable account fields: **31**
- Unchecked account fields: **0**
- CPI/token-operation references: **12**
- Arithmetic-operation references: **239**
- Require/constraint macro references: **71**
- Rust and integration tests: **138**
- Fuzz targets: **6**
- Attack hypotheses registered: **147**

## Severity count

| Severity | Count |
|---|---:|
| CRITICAL | 12 |
| HIGH | 1 |
| MEDIUM | 102 |
| LOW | 32 |

## Public instruction attack surface

| Entrypoint | lib.rs line | Priority |
|---|---:|---|
| `initialize` | 19 | Critical path |
| `initialize_protocol_config` | 23 | Critical path |
| `initialize_founder` | 27 | Critical path |
| `initialize_company` | 36 | Critical path |
| `initialize_treasury` | 45 | Critical path |
| `initialize_execution_config` | 49 | Critical path |
| `deposit_settlement` | 53 | Critical path |
| `process_fees` | 57 | Critical path |
| `authorize_reserve_execution` | 61 | Critical path |
| `authorize_buyback_execution` | 65 | Critical path |
| `authorize_liquidity_execution` | 69 | Critical path |
| `authorize_founder_execution` | 73 | Critical path |
| `authorize_company_execution` | 77 | Critical path |

## Account and PDA inventory

| File | Line | Accounts struct | Account | Type | Signer | Mut | Init | PDA seeds | Unchecked |
|---|---:|---|---|---|---:|---:|---:|---|---:|
| `programs/treasury-router/src/instructions/buyback.rs` | 24 | `AuthorizeBuybackExecution` | `protocol_state` | `Box<Account<'info` | No | No | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 33 | `AuthorizeBuybackExecution` | `protocol_config` | `Box<Account<'info` | No | No | No | `PROTOCOL_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 46 | `AuthorizeBuybackExecution` | `founder_state` | `Box<Account<'info` | No | No | No | `FOUNDER_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 59 | `AuthorizeBuybackExecution` | `company_state` | `Box<Account<'info` | No | No | No | `COMPANY_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 72 | `AuthorizeBuybackExecution` | `treasury` | `Box<Account<'info` | No | Yes | No | `TREASURY_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 88 | `AuthorizeBuybackExecution` | `execution_config` | `Box<Account<'info` | No | No | No | `EXECUTION_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 101 | `AuthorizeBuybackExecution` | `settlement_mint` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 107 | `AuthorizeBuybackExecution` | `settlement_vault` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 120 | `AuthorizeBuybackExecution` | `buyback_destination` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 133 | `AuthorizeBuybackExecution` | `reserve_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 143 | `AuthorizeBuybackExecution` | `liquidity_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 153 | `AuthorizeBuybackExecution` | `company_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 165 | `AuthorizeBuybackExecution` | `founder_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/buyback.rs` | 175 | `AuthorizeBuybackExecution` | `token_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/company.rs` | 14 | `AuthorizeCompanyExecution` | `protocol_state` | `Box<Account<'info` | No | No | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/company.rs` | 25 | `AuthorizeCompanyExecution` | `treasury` | `Box<Account<'info` | No | Yes | No | `TREASURY_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/company.rs` | 41 | `AuthorizeCompanyExecution` | `company_state` | `Box<Account<'info` | No | No | No | `COMPANY_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/company.rs` | 52 | `AuthorizeCompanyExecution` | `execution_config` | `Box<Account<'info` | No | No | No | `EXECUTION_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/company.rs` | 65 | `AuthorizeCompanyExecution` | `settlement_mint` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/company.rs` | 71 | `AuthorizeCompanyExecution` | `settlement_vault` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/company.rs` | 84 | `AuthorizeCompanyExecution` | `company_destination` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/company.rs` | 97 | `AuthorizeCompanyExecution` | `token_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 11 | `DepositSettlement` | `protocol_state` | `Box<Account<'info` | No | No | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 21 | `DepositSettlement` | `treasury` | `Box<Account<'info` | No | No | No | `TREASURY_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 36 | `DepositSettlement` | `settlement_mint` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 42 | `DepositSettlement` | `source_token_account` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 51 | `DepositSettlement` | `settlement_vault` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 62 | `DepositSettlement` | `authority` | `Signer<'info>` | Yes | No | No | `—` | No |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 64 | `DepositSettlement` | `token_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/founder.rs` | 14 | `AuthorizeFounderExecution` | `protocol_state` | `Box<Account<'info` | No | No | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/founder.rs` | 25 | `AuthorizeFounderExecution` | `treasury` | `Box<Account<'info` | No | Yes | No | `TREASURY_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/founder.rs` | 41 | `AuthorizeFounderExecution` | `founder_state` | `Box<Account<'info` | No | No | No | `FOUNDER_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/founder.rs` | 52 | `AuthorizeFounderExecution` | `execution_config` | `Box<Account<'info` | No | No | No | `EXECUTION_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/founder.rs` | 65 | `AuthorizeFounderExecution` | `settlement_mint` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/founder.rs` | 71 | `AuthorizeFounderExecution` | `settlement_vault` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/founder.rs` | 84 | `AuthorizeFounderExecution` | `founder_destination` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/founder.rs` | 97 | `AuthorizeFounderExecution` | `token_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize.rs` | 9 | `Initialize` | `protocol_state` | `Account<'info` | No | No | Yes | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/initialize.rs` | 17 | `Initialize` | `authority` | `Signer<'info>` | Yes | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize.rs` | 20 | `Initialize` | `system_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 10 | `InitializeCompany` | `protocol_state` | `Account<'info` | No | Yes | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 17 | `InitializeCompany` | `company_state` | `Account<'info` | No | No | Yes | `COMPANY_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 29 | `InitializeCompany` | `authority` | `Signer<'info>` | Yes | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 32 | `InitializeCompany` | `system_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 14 | `InitializeExecutionConfig` | `protocol_state` | `Box<Account<'info` | No | No | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 28 | `InitializeExecutionConfig` | `treasury` | `Box<Account<'info` | No | No | No | `TREASURY_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 43 | `InitializeExecutionConfig` | `founder_state` | `Box<Account<'info` | No | No | No | `FOUNDER_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 54 | `InitializeExecutionConfig` | `company_state` | `Box<Account<'info` | No | No | No | `COMPANY_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 65 | `InitializeExecutionConfig` | `settlement_mint` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 71 | `InitializeExecutionConfig` | `settlement_vault` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 81 | `InitializeExecutionConfig` | `reserve_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 87 | `InitializeExecutionConfig` | `buyback_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 93 | `InitializeExecutionConfig` | `liquidity_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 99 | `InitializeExecutionConfig` | `company_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 107 | `InitializeExecutionConfig` | `founder_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 115 | `InitializeExecutionConfig` | `execution_config` | `Box<Account<'info` | No | No | Yes | `EXECUTION_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 127 | `InitializeExecutionConfig` | `authority` | `Signer<'info>` | Yes | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 130 | `InitializeExecutionConfig` | `system_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 10 | `InitializeFounder` | `protocol_state` | `Account<'info` | No | Yes | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 17 | `InitializeFounder` | `founder_state` | `Account<'info` | No | No | Yes | `FOUNDER_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 29 | `InitializeFounder` | `authority` | `Signer<'info>` | Yes | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 32 | `InitializeFounder` | `system_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 14 | `InitializeProtocolConfig` | `protocol_state` | `Account<'info` | No | Yes | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 21 | `InitializeProtocolConfig` | `protocol_config` | `Account<'info` | No | No | Yes | `PROTOCOL_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 33 | `InitializeProtocolConfig` | `authority` | `Signer<'info>` | Yes | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 36 | `InitializeProtocolConfig` | `system_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 14 | `InitializeTreasury` | `protocol_state` | `Account<'info` | No | Yes | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 21 | `InitializeTreasury` | `treasury_state` | `Account<'info` | No | No | Yes | `TREASURY_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 33 | `InitializeTreasury` | `settlement_mint` | `Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 38 | `InitializeTreasury` | `settlement_vault` | `Account<'info` | No | No | Yes | `TREASURY_VAULT_SEED, treasury_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 50 | `InitializeTreasury` | `authority` | `Signer<'info>` | Yes | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 53 | `InitializeTreasury` | `token_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 55 | `InitializeTreasury` | `system_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 24 | `AuthorizeLiquidityExecution` | `protocol_state` | `Box<Account<'info` | No | No | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 33 | `AuthorizeLiquidityExecution` | `protocol_config` | `Box<Account<'info` | No | No | No | `PROTOCOL_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 46 | `AuthorizeLiquidityExecution` | `founder_state` | `Box<Account<'info` | No | No | No | `FOUNDER_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 59 | `AuthorizeLiquidityExecution` | `company_state` | `Box<Account<'info` | No | No | No | `COMPANY_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 72 | `AuthorizeLiquidityExecution` | `treasury` | `Box<Account<'info` | No | Yes | No | `TREASURY_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 88 | `AuthorizeLiquidityExecution` | `execution_config` | `Box<Account<'info` | No | No | No | `EXECUTION_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 101 | `AuthorizeLiquidityExecution` | `settlement_mint` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 107 | `AuthorizeLiquidityExecution` | `settlement_vault` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 120 | `AuthorizeLiquidityExecution` | `liquidity_destination` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 133 | `AuthorizeLiquidityExecution` | `reserve_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 143 | `AuthorizeLiquidityExecution` | `buyback_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 153 | `AuthorizeLiquidityExecution` | `company_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 165 | `AuthorizeLiquidityExecution` | `founder_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/liquidity.rs` | 175 | `AuthorizeLiquidityExecution` | `token_program` | `Program<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/process_fees.rs` | 17 | `ProcessFees` | `protocol_state` | `Box<Account<'info` | No | Yes | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/process_fees.rs` | 27 | `ProcessFees` | `protocol_config` | `Box<Account<'info` | No | No | No | `PROTOCOL_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/process_fees.rs` | 37 | `ProcessFees` | `treasury` | `Box<Account<'info` | No | Yes | No | `TREASURY_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/process_fees.rs` | 48 | `ProcessFees` | `founder_state` | `Box<Account<'info` | No | Yes | No | `FOUNDER_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/process_fees.rs` | 59 | `ProcessFees` | `company_state` | `Box<Account<'info` | No | Yes | No | `COMPANY_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/process_fees.rs` | 70 | `ProcessFees` | `settlement_vault` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 24 | `AuthorizeReserveExecution` | `protocol_state` | `Box<Account<'info` | No | No | No | `PROTOCOL_SEED` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 33 | `AuthorizeReserveExecution` | `protocol_config` | `Box<Account<'info` | No | No | No | `PROTOCOL_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 46 | `AuthorizeReserveExecution` | `founder_state` | `Box<Account<'info` | No | No | No | `FOUNDER_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 59 | `AuthorizeReserveExecution` | `company_state` | `Box<Account<'info` | No | No | No | `COMPANY_STATE_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 72 | `AuthorizeReserveExecution` | `treasury` | `Box<Account<'info` | No | Yes | No | `TREASURY_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 88 | `AuthorizeReserveExecution` | `execution_config` | `Box<Account<'info` | No | No | No | `EXECUTION_CONFIG_SEED, protocol_state.key().as_ref()` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 101 | `AuthorizeReserveExecution` | `settlement_mint` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 107 | `AuthorizeReserveExecution` | `settlement_vault` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 120 | `AuthorizeReserveExecution` | `reserve_destination` | `Box<Account<'info` | No | Yes | No | `—` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 133 | `AuthorizeReserveExecution` | `buyback_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 143 | `AuthorizeReserveExecution` | `liquidity_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 153 | `AuthorizeReserveExecution` | `company_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 165 | `AuthorizeReserveExecution` | `founder_destination` | `Box<Account<'info` | No | No | No | `—` | No |
| `programs/treasury-router/src/instructions/reserve.rs` | 175 | `AuthorizeReserveExecution` | `token_program` | `Program<'info` | No | No | No | `—` | No |

## CPI and token movement inventory

| File | Line | Operation | Evidence |
|---|---:|---|---|
| `programs/treasury-router/src/instructions/buyback.rs` | 274 | `CpiContext::new_with_signer` | `let cpi_context = CpiContext::new_with_signer(` |
| `programs/treasury-router/src/instructions/buyback.rs` | 280 | `token::transfer_checked` | `token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;` |
| `programs/treasury-router/src/instructions/company.rs` | 134 | `CpiContext::new_with_signer` | `let cpi_context = CpiContext::new_with_signer(` |
| `programs/treasury-router/src/instructions/company.rs` | 140 | `token::transfer_checked` | `token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;` |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 84 | `CpiContext::new` | `let cpi_context = CpiContext::new(ctx.accounts.token_program.key(), transfer_accounts);` |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 86 | `token::transfer_checked` | `token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;` |
| `programs/treasury-router/src/instructions/founder.rs` | 134 | `CpiContext::new_with_signer` | `let cpi_context = CpiContext::new_with_signer(` |
| `programs/treasury-router/src/instructions/founder.rs` | 140 | `token::transfer_checked` | `token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;` |
| `programs/treasury-router/src/instructions/liquidity.rs` | 271 | `CpiContext::new_with_signer` | `let cpi_context = CpiContext::new_with_signer(` |
| `programs/treasury-router/src/instructions/liquidity.rs` | 277 | `token::transfer_checked` | `token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;` |
| `programs/treasury-router/src/instructions/reserve.rs` | 274 | `CpiContext::new_with_signer` | `let cpi_context = CpiContext::new_with_signer(` |
| `programs/treasury-router/src/instructions/reserve.rs` | 280 | `token::transfer_checked` | `token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;` |

## Protocol engines

| Engine | File | Lines |
|---|---|---:|
| `beaver_score` | `programs/treasury-router/src/engines/beaver_score.rs` | 335 |
| `buyback` | `programs/treasury-router/src/engines/buyback.rs` | 38 |
| `company` | `programs/treasury-router/src/engines/company.rs` | 102 |
| `dam` | `programs/treasury-router/src/engines/dam.rs` | 211 |
| `execution_guard` | `programs/treasury-router/src/engines/execution_guard.rs` | 631 |
| `founder` | `programs/treasury-router/src/engines/founder.rs` | 102 |
| `integrity_firewall` | `programs/treasury-router/src/engines/integrity_firewall.rs` | 787 |
| `liquidity` | `programs/treasury-router/src/engines/liquidity.rs` | 37 |
| `mod` | `programs/treasury-router/src/engines/mod.rs` | 12 |
| `release` | `programs/treasury-router/src/engines/release.rs` | 614 |
| `reserve` | `programs/treasury-router/src/engines/reserve.rs` | 41 |
| `sentinel` | `programs/treasury-router/src/engines/sentinel.rs` | 685 |
| `waterfall` | `programs/treasury-router/src/engines/waterfall.rs` | 299 |

## Fuzz targets

| Target | File | Lines |
|---|---|---:|
| `adaptive_dam` | `fuzz/fuzz_targets/adaptive_dam.rs` | 158 |
| `execution_release_limit` | `fuzz/fuzz_targets/execution_release_limit.rs` | 61 |
| `process_fees` | `fuzz/fuzz_targets/process_fees.rs` | 342 |
| `process_release` | `fuzz/fuzz_targets/process_release.rs` | 389 |
| `treasury_accounting` | `fuzz/fuzz_targets/treasury_accounting.rs` | 211 |
| `waterfall_engine` | `fuzz/fuzz_targets/waterfall_engine.rs` | 212 |

## Master attack register

| ID | Attack class | Severity | Status | Target | Location | Required next attack |
|---|---|---|---|---|---|---|
| RT-135 | Initialization replay | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `initialize` | `programs/treasury-router/src/lib.rs:19` | Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts. |
| RT-136 | Initialization replay | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `initialize_protocol_config` | `programs/treasury-router/src/lib.rs:23` | Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts. |
| RT-137 | Initialization replay | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `initialize_founder` | `programs/treasury-router/src/lib.rs:27` | Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts. |
| RT-138 | Initialization replay | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `initialize_company` | `programs/treasury-router/src/lib.rs:36` | Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts. |
| RT-139 | Initialization replay | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `initialize_treasury` | `programs/treasury-router/src/lib.rs:45` | Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts. |
| RT-140 | Initialization replay | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `initialize_execution_config` | `programs/treasury-router/src/lib.rs:49` | Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts. |
| RT-142 | Accounting corruption | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `process_fees` | `programs/treasury-router/src/lib.rs:57` | Corrupt every bucket independently; overflow totals; repeat processing; manipulate timestamps and caps. |
| RT-143 | Treasury release theft | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `authorize_reserve_execution` | `programs/treasury-router/src/lib.rs:61` | Supply fake destinations, duplicate accounts, wrong mint, wrong vault, corrupt accounting, paused state, excessive release. |
| RT-144 | Treasury release theft | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `authorize_buyback_execution` | `programs/treasury-router/src/lib.rs:65` | Supply fake destinations, duplicate accounts, wrong mint, wrong vault, corrupt accounting, paused state, excessive release. |
| RT-145 | Treasury release theft | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `authorize_liquidity_execution` | `programs/treasury-router/src/lib.rs:69` | Supply fake destinations, duplicate accounts, wrong mint, wrong vault, corrupt accounting, paused state, excessive release. |
| RT-146 | Treasury release theft | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `authorize_founder_execution` | `programs/treasury-router/src/lib.rs:73` | Supply fake destinations, duplicate accounts, wrong mint, wrong vault, corrupt accounting, paused state, excessive release. |
| RT-147 | Treasury release theft | **CRITICAL** | PLANNED — ADVERSARIAL TEST | `authorize_company_execution` | `programs/treasury-router/src/lib.rs:77` | Supply fake destinations, duplicate accounts, wrong mint, wrong vault, corrupt accounting, paused state, excessive release. |
| RT-141 | Deposit substitution | **HIGH** | PLANNED — ADVERSARIAL TEST | `deposit_settlement` | `programs/treasury-router/src/lib.rs:53` | Supply wrong mint, wrong owner, wrong source authority, fake vault, zero amount, maximum amount. |
| RT-001 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/constants.rs:4` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-002 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/constants.rs:5` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-003 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/constants.rs:10` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-004 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/constants.rs:11` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-005 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/constants.rs:14` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-006 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/constants.rs:17` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-007 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/constants.rs:20` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-008 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/constants.rs:22` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-009 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked plus` | `programs/treasury-router/src/engines/beaver_score.rs:47` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-010 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked plus` | `programs/treasury-router/src/engines/beaver_score.rs:165` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-012 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/buyback.rs:7` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-013 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/buyback.rs:10` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-014 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/buyback.rs:13` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-015 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/buyback.rs:17` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-016 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/company.rs:45` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-017 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/dam.rs:6` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-018 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/dam.rs:41` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-019 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/dam.rs:70` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-020 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/dam.rs:81` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-021 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/dam.rs:82` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-022 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/dam.rs:84` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-024 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/execution_guard.rs:23` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-025 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/execution_guard.rs:38` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-026 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/execution_guard.rs:114` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-028 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/founder.rs:45` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-030 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/integrity_firewall.rs:71` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-031 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/integrity_firewall.rs:71` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-032 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/integrity_firewall.rs:103` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-029 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked plus` | `programs/treasury-router/src/engines/integrity_firewall.rs:368` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-036 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/release.rs:18` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-040 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/sentinel.rs:32` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-048 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked multiply` | `programs/treasury-router/src/engines/sentinel.rs:106` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-049 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked multiply` | `programs/treasury-router/src/engines/sentinel.rs:138` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-050 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked multiply` | `programs/treasury-router/src/engines/sentinel.rs:149` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-062 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/engines/waterfall.rs:8` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-064 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked divide` | `programs/treasury-router/src/engines/waterfall.rs:76` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-065 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/errors/mod.rs:20` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-066 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/errors/mod.rs:23` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-067 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/errors/mod.rs:26` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-068 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/errors/mod.rs:41` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-069 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/errors/mod.rs:44` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-070 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/errors/mod.rs:53` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-071 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/errors/mod.rs:56` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-072 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/errors/mod.rs:88` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-073 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/events/mod.rs:47` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-074 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/events/mod.rs:68` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-075 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/events/mod.rs:69` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-076 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/events/mod.rs:94` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-077 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/events/mod.rs:95` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-078 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/buyback.rs:120` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-079 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/buyback.rs:133` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-080 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/buyback.rs:143` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-081 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/buyback.rs:153` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-082 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/buyback.rs:165` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-083 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/buyback.rs:182` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-084 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/buyback.rs:187` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-085 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/buyback.rs:188` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-086 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/buyback.rs:311` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-087 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/company.rs:84` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-088 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/company.rs:106` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-089 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/company.rs:110` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-090 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/company.rs:172` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-091 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/deposit_settlement.rs:88` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-092 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/founder.rs:84` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-093 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/founder.rs:106` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-094 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/founder.rs:110` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-095 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/founder.rs:172` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-096 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/initialize_treasury.rs:38` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-097 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/liquidity.rs:120` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-098 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/liquidity.rs:133` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-099 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/liquidity.rs:143` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-100 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/liquidity.rs:153` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-101 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/liquidity.rs:165` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-102 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/liquidity.rs:304` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-103 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/liquidity.rs:352` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-104 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:91` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-105 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:176` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-106 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:201` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-118 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked divide` | `programs/treasury-router/src/instructions/process_fees.rs:207` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-119 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked divide` | `programs/treasury-router/src/instructions/process_fees.rs:207` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-107 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:238` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-108 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:243` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-109 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:248` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-110 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:306` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-111 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:362` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-112 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:364` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-113 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:387` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-120 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/reserve.rs:120` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-121 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/reserve.rs:120` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-122 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/reserve.rs:133` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-123 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/reserve.rs:143` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-124 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/reserve.rs:153` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-125 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/reserve.rs:165` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-126 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/instructions/reserve.rs:307` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-127 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/state/execution_config.rs:3` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-128 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/state/execution_config.rs:13` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-129 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked divide` | `programs/treasury-router/src/state/execution_config.rs:25` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-131 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/state/treasury.rs:37` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-132 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/state/treasury.rs:44` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-134 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked divide` | `programs/treasury-router/src/state/treasury.rs:69` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-133 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked minus` | `programs/treasury-router/src/state/treasury.rs:75` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-130 | Arithmetic review | **MEDIUM** | OPEN — MANUAL REVIEW | `unchecked plus` | `programs/treasury-router/src/state/treasury.rs:79` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-011 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked plus` | `programs/treasury-router/src/engines/beaver_score.rs:333` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-027 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/execution_guard.rs:534` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-023 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked plus` | `programs/treasury-router/src/engines/execution_guard.rs:597` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-033 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked plus` | `programs/treasury-router/src/engines/release.rs:330` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-037 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/release.rs:330` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-034 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked plus` | `programs/treasury-router/src/engines/release.rs:439` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-035 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked plus` | `programs/treasury-router/src/engines/release.rs:477` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-038 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/release.rs:539` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-039 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/release.rs:544` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-051 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked multiply` | `programs/treasury-router/src/engines/sentinel.rs:281` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-041 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/sentinel.rs:292` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-042 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/sentinel.rs:315` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-043 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/sentinel.rs:317` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-044 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/sentinel.rs:339` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-052 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked multiply` | `programs/treasury-router/src/engines/sentinel.rs:354` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-053 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked multiply` | `programs/treasury-router/src/engines/sentinel.rs:372` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-045 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/sentinel.rs:397` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-046 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/sentinel.rs:397` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-054 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked multiply` | `programs/treasury-router/src/engines/sentinel.rs:397` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-047 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/sentinel.rs:398` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-055 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked multiply` | `programs/treasury-router/src/engines/sentinel.rs:413` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-056 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked multiply` | `programs/treasury-router/src/engines/sentinel.rs:435` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-057 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked multiply` | `programs/treasury-router/src/engines/sentinel.rs:436` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-063 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/engines/waterfall.rs:167` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-058 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked plus` | `programs/treasury-router/src/engines/waterfall.rs:183` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-059 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked plus` | `programs/treasury-router/src/engines/waterfall.rs:184` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-060 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked plus` | `programs/treasury-router/src/engines/waterfall.rs:185` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-061 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked plus` | `programs/treasury-router/src/engines/waterfall.rs:186` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-114 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:683` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-115 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:710` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-116 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:736` | Test maximum values, zero values, rounding, and overflow behavior. |
| RT-117 | Arithmetic review | **LOW** | REVIEW — LIKELY TEST CODE | `unchecked minus` | `programs/treasury-router/src/instructions/process_fees.rs:740` | Test maximum values, zero values, rounding, and overflow behavior. |

## Detailed attack hypotheses

### RT-135 — Initialization replay

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `initialize`
- **Location:** `programs/treasury-router/src/lib.rs:19`
- **Description:** Public instruction `initialize` is an externally reachable attack surface.
- **Evidence:** `pub fn initialize(...)`
- **Required attack:** Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts.

### RT-136 — Initialization replay

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `initialize_protocol_config`
- **Location:** `programs/treasury-router/src/lib.rs:23`
- **Description:** Public instruction `initialize_protocol_config` is an externally reachable attack surface.
- **Evidence:** `pub fn initialize_protocol_config(...)`
- **Required attack:** Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts.

### RT-137 — Initialization replay

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `initialize_founder`
- **Location:** `programs/treasury-router/src/lib.rs:27`
- **Description:** Public instruction `initialize_founder` is an externally reachable attack surface.
- **Evidence:** `pub fn initialize_founder(...)`
- **Required attack:** Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts.

### RT-138 — Initialization replay

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `initialize_company`
- **Location:** `programs/treasury-router/src/lib.rs:36`
- **Description:** Public instruction `initialize_company` is an externally reachable attack surface.
- **Evidence:** `pub fn initialize_company(...)`
- **Required attack:** Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts.

### RT-139 — Initialization replay

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `initialize_treasury`
- **Location:** `programs/treasury-router/src/lib.rs:45`
- **Description:** Public instruction `initialize_treasury` is an externally reachable attack surface.
- **Evidence:** `pub fn initialize_treasury(...)`
- **Required attack:** Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts.

### RT-140 — Initialization replay

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `initialize_execution_config`
- **Location:** `programs/treasury-router/src/lib.rs:49`
- **Description:** Public instruction `initialize_execution_config` is an externally reachable attack surface.
- **Evidence:** `pub fn initialize_execution_config(...)`
- **Required attack:** Call twice; call out of order; race two transactions; supply alternate PDA, wrong authority, wrong linked accounts.

### RT-142 — Accounting corruption

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `process_fees`
- **Location:** `programs/treasury-router/src/lib.rs:57`
- **Description:** Public instruction `process_fees` is an externally reachable attack surface.
- **Evidence:** `pub fn process_fees(...)`
- **Required attack:** Corrupt every bucket independently; overflow totals; repeat processing; manipulate timestamps and caps.

### RT-143 — Treasury release theft

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `authorize_reserve_execution`
- **Location:** `programs/treasury-router/src/lib.rs:61`
- **Description:** Public instruction `authorize_reserve_execution` is an externally reachable attack surface.
- **Evidence:** `pub fn authorize_reserve_execution(...)`
- **Required attack:** Supply fake destinations, duplicate accounts, wrong mint, wrong vault, corrupt accounting, paused state, excessive release.

### RT-144 — Treasury release theft

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `authorize_buyback_execution`
- **Location:** `programs/treasury-router/src/lib.rs:65`
- **Description:** Public instruction `authorize_buyback_execution` is an externally reachable attack surface.
- **Evidence:** `pub fn authorize_buyback_execution(...)`
- **Required attack:** Supply fake destinations, duplicate accounts, wrong mint, wrong vault, corrupt accounting, paused state, excessive release.

### RT-145 — Treasury release theft

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `authorize_liquidity_execution`
- **Location:** `programs/treasury-router/src/lib.rs:69`
- **Description:** Public instruction `authorize_liquidity_execution` is an externally reachable attack surface.
- **Evidence:** `pub fn authorize_liquidity_execution(...)`
- **Required attack:** Supply fake destinations, duplicate accounts, wrong mint, wrong vault, corrupt accounting, paused state, excessive release.

### RT-146 — Treasury release theft

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `authorize_founder_execution`
- **Location:** `programs/treasury-router/src/lib.rs:73`
- **Description:** Public instruction `authorize_founder_execution` is an externally reachable attack surface.
- **Evidence:** `pub fn authorize_founder_execution(...)`
- **Required attack:** Supply fake destinations, duplicate accounts, wrong mint, wrong vault, corrupt accounting, paused state, excessive release.

### RT-147 — Treasury release theft

- **Severity:** CRITICAL
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `authorize_company_execution`
- **Location:** `programs/treasury-router/src/lib.rs:77`
- **Description:** Public instruction `authorize_company_execution` is an externally reachable attack surface.
- **Evidence:** `pub fn authorize_company_execution(...)`
- **Required attack:** Supply fake destinations, duplicate accounts, wrong mint, wrong vault, corrupt accounting, paused state, excessive release.

### RT-141 — Deposit substitution

- **Severity:** HIGH
- **Status:** PLANNED — ADVERSARIAL TEST
- **Target:** `deposit_settlement`
- **Location:** `programs/treasury-router/src/lib.rs:53`
- **Description:** Public instruction `deposit_settlement` is an externally reachable attack surface.
- **Evidence:** `pub fn deposit_settlement(...)`
- **Required attack:** Supply wrong mint, wrong owner, wrong source authority, fake vault, zero amount, maximum amount.

### RT-001 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/constants.rs:4`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// On-chain Beavernomics configuration PDA.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-002 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/constants.rs:5`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `pub const PROTOCOL_CONFIG_SEED: &[u8] = b"protocol-config";`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-003 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/constants.rs:10`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Treasury SPL settlement-token vault PDA.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-004 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/constants.rs:11`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `pub const TREASURY_VAULT_SEED: &[u8] = b"treasury-vault";`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-005 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/constants.rs:14`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `pub const FOUNDER_STATE_SEED: &[u8] = b"founder-state";`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-006 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/constants.rs:17`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `pub const COMPANY_STATE_SEED: &[u8] = b"company-state";`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-007 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/constants.rs:20`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `pub const EXECUTION_CONFIG_SEED: &[u8] = b"execution-config";`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-008 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/constants.rs:22`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Current on-chain state versions.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-009 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/beaver_score.rs:47`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `WATERFALL_MAX_POINTS + RESERVE_MAX_POINTS + ACCOUNTING_MAX_POINTS;`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-010 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/beaver_score.rs:165`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `integrity_points + activity_points`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-012 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/buyback.rs:7`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Amount credited to the pending buyback-and-burn bucket.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-013 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/buyback.rs:10`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Updated pending buyback-and-burn balance.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-014 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/buyback.rs:13`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Updated lifetime buyback-and-burn allocation.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-015 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/buyback.rs:17`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Credits a buyback-and-burn allocation to TreasuryState.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-016 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/company.rs:45`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `// Automatic accounting-period reset`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-017 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/dam.rs:6`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// of the on-chain interface. Do not reorder them without a state migration.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-018 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/dam.rs:41`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Maximum capital-release rate associated with this Dam level.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-019 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/dam.rs:70`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Dam v1 uses a deterministic one-to-one mapping:`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-020 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/dam.rs:81`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Evaluates the Dam using the Waterfall stage and the non-circular`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-021 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/dam.rs:82`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Pre-Dam Health Score.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-022 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/dam.rs:84`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// The Waterfall establishes the least-permissive baseline required by`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-024 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/execution_guard.rs:23`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Result returned by the central execution-security gate.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-025 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/execution_guard.rs:38`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// protocol-to-treasury linkage must be enforced by the Anchor instruction`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-026 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/execution_guard.rs:114`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// on-chain security system.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-028 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/founder.rs:45`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `// Automatic accounting-period reset`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-030 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:71`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Read-only SPL token-account facts supplied to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-031 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:71`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Read-only SPL token-account facts supplied to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-032 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:103`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Read-only Integrity Firewall report.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-029 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:368`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `for right in (left + 1)..destination_keys.len() {`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-036 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/release.rs:18`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Applies the accounting transition for an already-authorized treasury release.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-040 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:32`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Read-only protocol integrity report.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-048 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked multiply`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:106`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* This explicit match is intentionally exhaustive. Adding a new stage`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-049 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked multiply`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:138`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* A Dam may never permit more than 100% release, and the reported rate`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-050 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked multiply`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:149`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* The Dam evaluation must retain the exact Waterfall stage supplied to it.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-062 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/waterfall.rs:8`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// The stored u8 values are part of the on-chain protocol interface,`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-064 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked divide`
- **Location:** `programs/treasury-router/src/engines/waterfall.rs:76`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// pending_reserve / total_pending`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-065 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/errors/mod.rs:20`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `#[msg("The supplied settlement-token vault is invalid.")]`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-066 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/errors/mod.rs:23`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `#[msg("The settlement-token deposit amount must be greater than zero.")]`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-067 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/errors/mod.rs:26`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `#[msg("There are no unprocessed settlement-token fees in the treasury vault.")]`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-068 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/errors/mod.rs:41`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `#[msg("The founder accounting-period cap must be greater than zero.")]`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-069 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/errors/mod.rs:44`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `#[msg("The founder accounting-period duration must be greater than zero.")]`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-070 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/errors/mod.rs:53`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `#[msg("The company accounting-period cap must be greater than zero.")]`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-071 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/errors/mod.rs:56`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `#[msg("The company accounting-period duration must be greater than zero.")]`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-072 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/errors/mod.rs:88`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `#[msg("Invalid settlement-token execution destination")]`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-073 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/events/mod.rs:47`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// The settlement-token transfer and accounting update occur atomically.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-074 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/events/mod.rs:68`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Founder caps are enforced during fee processing. The settlement-token`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-075 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/events/mod.rs:69`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// transfer and treasury execution-accounting update occur atomically.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-076 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/events/mod.rs:94`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Company caps are enforced during fee processing. The settlement-token`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-077 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/events/mod.rs:95`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// transfer and treasury execution-accounting update occur atomically.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-078 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/buyback.rs:120`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Settlement-token account used for the future buyback execution pipeline.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-079 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/buyback.rs:133`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent reserve destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-080 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/buyback.rs:143`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent liquidity destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-081 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/buyback.rs:153`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent company destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-082 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/buyback.rs:165`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent founder destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-083 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/buyback.rs:182`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Releases previously allocated buyback-and-burn funds.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-084 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/buyback.rs:187`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// - decreases pending buyback-and-burn accounting;`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-085 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/buyback.rs:188`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// - increases released buyback-and-burn accounting.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-086 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/buyback.rs:311`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `msg!("Buyback settlement-token execution completed");`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-087 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/company.rs:84`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Settlement-token account controlled by the configured company recipient.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-088 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/company.rs:106`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Company period-cap enforcement occurs during fee processing when funds are`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-089 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/company.rs:110`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// The settlement-token transfer and treasury accounting update are atomic.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-090 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/company.rs:172`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `msg!("Company settlement-token execution completed");`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-091 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/deposit_settlement.rs:88`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `msg!("Settlement-token deposit completed");`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-092 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/founder.rs:84`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Settlement-token account controlled by the configured founder recipient.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-093 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/founder.rs:106`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Founder period-cap enforcement occurs while fees are processed and credited`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-094 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/founder.rs:110`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// The settlement-token transfer and treasury accounting update are atomic.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-095 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/founder.rs:172`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `msg!("Founder settlement-token execution completed");`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-096 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/initialize_treasury.rs:38`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Treasury-owned SPL token vault for the settlement mint.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-097 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/liquidity.rs:120`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Settlement-token account used for liquidity deployment.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-098 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/liquidity.rs:133`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent reserve destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-099 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/liquidity.rs:143`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent buyback destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-100 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/liquidity.rs:153`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent company destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-101 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/liquidity.rs:165`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent founder destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-102 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/liquidity.rs:304`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `msg!("Liquidity settlement-token execution completed");`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-103 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/liquidity.rs:352`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `msg!("No external swap or liquidity-pool CPI was performed");`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-104 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:91`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `// canonical protocol linkage, and lifetime-allocation conservation.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-105 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:176`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `// Re-run both Sentinel layers after every state mutation. A failed`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-106 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:201`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `"Pre-allocation Waterfall stage: {} ({})",`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-118 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked divide`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:207`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `"Adaptive allocation BPS R/B/L/C/F: {}/{}/{}/{}/{}",`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-119 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked divide`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:207`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `"Adaptive allocation BPS R/B/L/C/F: {}/{}/{}/{}/{}",`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-107 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:238`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `"Buyback-and-burn allocation deposited: {}",`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-108 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:243`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `"Pending buyback-and-burn balance: {}",`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-109 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:248`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `"Lifetime buyback-and-burn allocation: {}",`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-110 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:306`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `"Pre-Dam Health Score: {} / {}",`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-111 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:362`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Deterministic result of one successful Beavernomics fee-processing cycle.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-112 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:364`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Account validation, vault-balance discovery, protocol pause enforcement,`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-113 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:387`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Applies one complete production fee-allocation state transition.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-120 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/reserve.rs:120`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Reserve-controlled settlement-token account receiving the release.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-121 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/reserve.rs:120`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Reserve-controlled settlement-token account receiving the release.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-122 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/reserve.rs:133`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent buyback destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-123 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/reserve.rs:143`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent liquidity destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-124 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/reserve.rs:153`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent company destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-125 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/reserve.rs:165`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent founder destination supplied read-only to the firewall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-126 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/reserve.rs:307`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `msg!("Reserve settlement-token execution completed");`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-127 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/state/execution_config.rs:3`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Immutable routing configuration for all protocol-controlled executions.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-128 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/state/execution_config.rs:13`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Settlement-token mint accepted by every configured destination.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-129 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked divide`
- **Location:** `programs/treasury-router/src/state/execution_config.rs:25`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Permanent destination for Company/Operations funding.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-131 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/state/treasury.rs:37`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Lifetime settlement-token amounts actually released from the treasury.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-132 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/state/treasury.rs:44`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Last successful fee-processing timestamp.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-134 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked divide`
- **Location:** `programs/treasury-router/src/state/treasury.rs:69`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `(19 * 8) + // u64/i64 accounting fields`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-133 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/state/treasury.rs:75`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Validates one accounting bucket using fail-closed checked arithmetic.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-130 — Arithmetic review

- **Severity:** MEDIUM
- **Status:** OPEN — MANUAL REVIEW
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/state/treasury.rs:79`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// lifetime == pending + released`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-011 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/beaver_score.rs:333`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `assert_eq!(score, WATERFALL_MAX_POINTS + RESERVE_MAX_POINTS + 50);`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-027 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/execution_guard.rs:534`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `// The Adaptive Dam tightens this Caution-stage fixture to`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-023 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/execution_guard.rs:597`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `// Lifetime must equal pending + released.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-033 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/release.rs:330`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `// The pre-existing state is itself invalid because pending + released`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-037 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/release.rs:330`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `// The pre-existing state is itself invalid because pending + released`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-034 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/release.rs:439`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `"every bucket must satisfy lifetime = pending + released",`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-035 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/release.rs:477`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `collect_release_permutations(buckets, index + 1, permutations);`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-038 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/release.rs:539`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `"release must not alter received-fee accounting",`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-039 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/release.rs:544`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `"release must not alter allocated-fee accounting",`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-051 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked multiply`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:281`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* It receives plain references and verifies that their values remain`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-041 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:292`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Stable Sentinel v2 failure-mask identifiers.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-042 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:315`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Read-only Sentinel v2 report.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-043 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:317`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// A zero `failure_mask` means every linkage and immutable-configuration`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-044 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:339`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Evaluates immutable configuration and protocol-account linkage.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-052 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked multiply`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:354`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* The configuration and treasury must both point to the same canonical`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-053 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked multiply`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:372`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* Merely totalling 10,000 basis points is insufficient: a corrupted`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-045 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:397`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* Ownership and SPL-token mint checks require actual token-account data`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-046 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:397`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* Ownership and SPL-token mint checks require actual token-account data`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-054 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked multiply`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:397`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* Ownership and SPL-token mint checks require actual token-account data`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-047 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:398`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* and will be added later through a dedicated account-view verifier.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-055 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked multiply`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:413`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* Every lifetime bucket must add up exactly to the protocol's recorded`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-056 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked multiply`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:435`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* In the current accounting design these values should ordinarily be`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-057 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked multiply`
- **Location:** `programs/treasury-router/src/engines/sentinel.rs:436`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `* equal after successful processing, but <= is the fundamental safety`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-063 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/engines/waterfall.rs:167`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `/// Deterministic fee-allocation profile selected by the Survival Waterfall.`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-058 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/waterfall.rs:183`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `self.reserve_bps as u32`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-059 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/waterfall.rs:184`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `+ self.buyback_burn_bps as u32`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-060 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/waterfall.rs:185`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `+ self.liquidity_bps as u32`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-061 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked plus`
- **Location:** `programs/treasury-router/src/engines/waterfall.rs:186`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `+ self.company_bps as u32`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-114 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:683`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `let company_overflow = requested_company - actual_company;`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-115 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:710`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `let founder_overflow = requested_founder - actual_founder;`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-116 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:736`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `let company_overflow = requested_company - actual_company;`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

### RT-117 — Arithmetic review

- **Severity:** LOW
- **Status:** REVIEW — LIKELY TEST CODE
- **Target:** `unchecked minus`
- **Location:** `programs/treasury-router/src/instructions/process_fees.rs:740`
- **Description:** Potential unchecked arithmetic expression detected.
- **Evidence:** `let founder_overflow = requested_founder - actual_founder;`
- **Required attack:** Test maximum values, zero values, rounding, and overflow behavior.

## Current test baseline

```text
warning: unexpected `cfg` condition value: `custom-heap`
  --> programs/treasury-router/src/lib.rs:15:1
   |
15 | #[program]
   | ^^^^^^^^^^
   |
   = note: expected values for `feature` are: `anchor-debug`, `cpi`, `default`, `idl-build`, `no-entrypoint`, `no-idl`, and `no-log-ix-name`
   = note: using a cfg inside a macro will use the cfgs from the destination crate and not the ones from the defining crate
   = help: try referring to `$crate::custom_heap_default` crate for guidance on how handle this unexpected cfg
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
   = note: `#[warn(unexpected_cfgs)]` on by default
   = note: this warning originates in the macro `$crate::custom_heap_default` which comes from the expansion of the attribute macro `program` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: unexpected `cfg` condition value: `solana`
  --> programs/treasury-router/src/lib.rs:15:1
   |
15 | #[program]
   | ^^^^^^^^^^
   |
   = note: expected values for `target_os` are: `aix`, `amdhsa`, `android`, `cuda`, `cygwin`, `dragonfly`, `emscripten`, `espidf`, `freebsd`, `fuchsia`, `haiku`, `helenos`, `hermit`, `horizon`, `hurd`, `illumos`, `ios`, `l4re`, `linux`, `lynxos178`, `macos`, `managarm`, `motor`, `netbsd`, `none`, `nto`, `nuttx`, `openbsd`, `psp`, `psx`, `qurt`, `redox`, `rtems`, `solaris`, and `solid_asp3` and 14 more
   = note: using a cfg inside a macro will use the cfgs from the destination crate and not the ones from the defining crate
   = help: try referring to `$crate::custom_heap_default` crate for guidance on how handle this unexpected cfg
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
   = note: this warning originates in the macro `$crate::custom_heap_default` which comes from the expansion of the attribute macro `program` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: unexpected `cfg` condition value: `custom-panic`
  --> programs/treasury-router/src/lib.rs:15:1
   |
15 | #[program]
   | ^^^^^^^^^^
   |
   = note: expected values for `feature` are: `anchor-debug`, `cpi`, `default`, `idl-build`, `no-entrypoint`, `no-idl`, and `no-log-ix-name`
   = note: using a cfg inside a macro will use the cfgs from the destination crate and not the ones from the defining crate
   = help: try referring to `$crate::custom_panic_default` crate for guidance on how handle this unexpected cfg
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
   = note: this warning originates in the macro `$crate::custom_panic_default` which comes from the expansion of the attribute macro `program` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: unexpected `cfg` condition value: `solana`
  --> programs/treasury-router/src/lib.rs:15:1
   |
15 | #[program]
   | ^^^^^^^^^^
   |
   = note: expected values for `target_os` are: `aix`, `amdhsa`, `android`, `cuda`, `cygwin`, `dragonfly`, `emscripten`, `espidf`, `freebsd`, `fuchsia`, `haiku`, `helenos`, `hermit`, `horizon`, `hurd`, `illumos`, `ios`, `l4re`, `linux`, `lynxos178`, `macos`, `managarm`, `motor`, `netbsd`, `none`, `nto`, `nuttx`, `openbsd`, `psp`, `psx`, `qurt`, `redox`, `rtems`, `solaris`, and `solid_asp3` and 14 more
   = note: using a cfg inside a macro will use the cfgs from the destination crate and not the ones from the defining crate
   = help: try referring to `$crate::custom_panic_default` crate for guidance on how handle this unexpected cfg
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
   = note: this warning originates in the macro `$crate::custom_panic_default` which comes from the expansion of the attribute macro `program` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: `treasury-router` (lib) generated 4 warnings
warning: `treasury-router` (lib test) generated 4 warnings (4 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.72s
     Running unittests src/lib.rs (target/debug/deps/treasury_router-338f8d4c1464469d)

running 122 tests
test engines::beaver_score::tests::current_initial_allocation_scores_correctly ... ok
test engines::beaver_score::pre_dam_health_tests::corrupted_accounting_removes_integrity_points ... ok
test engines::beaver_score::pre_dam_health_tests::emergency_pre_dam_health_retains_only_accounting_points ... ok
test engines::beaver_score::tests::emergency_protocol_scores_only_accounting_points ... ok
test engines::beaver_score::pre_dam_health_tests::ideal_pre_dam_health_scores_seven_hundred_fifty ... ok
test engines::beaver_score::tests::ideal_protocol_scores_one_thousand ... ok
test engines::beaver_score::tests::inconsistent_accounting_loses_integrity_points ... ok
test engines::beaver_score::tests::reserve_score_is_capped ... ok
test engines::dam::adaptive_dam_tests::critically_low_health_closes_the_dam ... ok
test engines::dam::adaptive_dam_tests::emergency_waterfall_always_closes_the_dam ... ok
test engines::dam::adaptive_dam_tests::health_score_can_never_weaken_waterfall_protection ... ok
test engines::dam::adaptive_dam_tests::healthy_normal_protocol_allows_full_release ... ok
test engines::dam::tests::caution_waterfall_uses_normal_dam_level ... ok
test engines::dam::adaptive_dam_tests::weak_health_tightens_a_normal_waterfall ... ok
test engines::dam::tests::emergency_waterfall_closes_dam ... ok
test engines::dam::tests::defensive_waterfall_uses_controlled_dam_level ... ok
test engines::dam::tests::survival_waterfall_restricts_dam ... ok
test engines::execution_guard::tests::authorizes_release_at_exact_dam_limit ... ok
test engines::execution_guard::tests::authorizes_valid_buyback_release ... ok
test engines::dam::tests::normal_waterfall_opens_dam_fully ... ok
test engines::execution_guard::tests::buyback_pause_does_not_block_other_buckets ... ok
test engines::execution_guard::tests::authorizes_valid_reserve_release ... ok
test engines::execution_guard::tests::full_dam_allows_full_pending_balance ... ok
test engines::execution_guard::tests::controlled_dam_allows_fifty_percent ... ok
test engines::execution_guard::tests::filling_dam_allows_nothing ... ok
test engines::execution_guard::tests::normal_dam_allows_seventy_five_percent ... ok
test engines::execution_guard::tests::pending_balance_returns_correct_bucket_amounts ... ok
test engines::execution_guard::tests::rejects_buyback_when_buybacks_are_paused ... ok
test engines::execution_guard::tests::rejects_invalid_dam_release_rate ... ok
test engines::execution_guard::tests::rejects_invalid_non_requested_bucket_accounting ... ok
test engines::execution_guard::tests::rejects_invalid_reserve_bucket_accounting ... ok
test engines::execution_guard::tests::rejects_release_exceeding_current_dam_limit ... ok
test engines::execution_guard::tests::rejects_release_when_dam_is_closed ... ok
test engines::execution_guard::tests::rejects_release_when_protocol_is_paused ... ok
test engines::execution_guard::tests::rejects_when_allocated_fees_exceed_received_fees ... ok
test engines::execution_guard::tests::rejects_when_total_accounting_is_not_settled ... ok
test engines::execution_guard::tests::rejects_zero_release_amount ... ok
test engines::execution_guard::tests::release_calculation_rounds_down ... ok
test engines::execution_guard::tests::restricted_dam_allows_twenty_five_percent ... ok
test engines::execution_guard::tests::rejects_release_exceeding_pending_balance ... ok
test engines::integrity_firewall::tests::invariant_masks_are_unique_and_single_bit ... ok
test engines::integrity_firewall::tests::corrupted_treasury_bump_is_detected ... ok
test engines::integrity_firewall::tests::destination_pointing_to_treasury_vault_is_detected ... ok
test engines::integrity_firewall::tests::duplicate_destinations_are_detected ... ok
test engines::integrity_firewall::tests::wrong_founder_destination_owner_is_detected ... ok
test engines::release::tests::full_pending_balance_can_be_released ... ok
test engines::integrity_firewall::tests::wrong_treasury_vault_owner_is_detected ... ok
test engines::integrity_firewall::tests::valid_architecture_passes_every_integrity_check ... ok
test engines::release::tests::processes_founder_release ... ok
test engines::integrity_firewall::tests::wrong_program_id_fails_closed ... ok
test engines::integrity_firewall::tests::wrong_destination_mint_is_detected ... ok
test engines::release::tests::all_release_orders_reach_the_same_final_treasury_state ... ok
test engines::release::tests::processes_buyback_release ... ok
test engines::release::tests::processes_liquidity_release ... ok
test engines::release::tests::rejects_corrupted_unrelated_bucket_before_mutation ... ok
test engines::release::tests::rejects_corrupted_selected_bucket_before_mutation ... ok
test engines::release::tests::rejects_released_balance_overflow_without_mutation ... ok
test engines::release::tests::processes_company_release ... ok
test engines::release::tests::rejects_zero_release_without_mutating_treasury ... ok
test engines::sentinel::sentinel_v2_tests::allocated_fees_cannot_exceed_received_fees ... ok
test engines::release::tests::processes_reserve_release ... ok
test engines::sentinel::sentinel_v2_tests::corrupted_lifetime_total_is_detected ... ok
test engines::sentinel::sentinel_v2_tests::allocation_total_of_one_hundred_percent_is_not_enough ... ok
test engines::release::tests::rejects_release_above_pending_without_mutating_treasury ... ok
test engines::sentinel::sentinel_v2_tests::configuration_updates_enabled_fails_closed ... ok
test engines::sentinel::sentinel_v2_tests::lifetime_sum_overflow_fails_closed ... ok
test engines::sentinel::sentinel_v2_tests::linkage_masks_are_unique ... ok
test engines::sentinel::sentinel_v2_tests::missing_settlement_references_fail ... ok
test engines::sentinel::sentinel_v2_tests::wrong_protocol_links_are_detected_independently ... ok
test engines::sentinel::tests::canonical_dam_rates_are_never_above_one_hundred_percent ... ok
test engines::sentinel::sentinel_v2_tests::valid_locked_protocol_passes_every_linkage_check ... ok
test engines::sentinel::tests::invariant_failure_detection_is_exact ... ok
test engines::sentinel::tests::invariant_masks_are_unique_and_single_bit ... ok
test engines::sentinel::tests::zero_failure_mask_contains_no_failures ... ok
test engines::sentinel::tests::every_waterfall_stage_produces_a_consistent_dam_evaluation ... ok
test engines::waterfall::adaptive_allocation_tests::defensive_profiles_prioritize_reserve ... ok
test engines::waterfall::adaptive_allocation_tests::every_profile_allocates_exactly_one_hundred_percent ... ok
test engines::waterfall::adaptive_allocation_tests::normal_profile_preserves_locked_beavernomics ... ok
test engines::waterfall::tests::classifies_caution_stage ... ok
test engines::waterfall::adaptive_allocation_tests::survival_and_emergency_stop_optional_allocations ... ok
test engines::waterfall::tests::classifies_defensive_stage ... ok
test engines::waterfall::tests::classifies_emergency_stage ... ok
test engines::waterfall::tests::classifies_normal_stage ... ok
test engines::waterfall::tests::classifies_survival_stage ... ok
test instructions::process_fees::tests::allocation_total_fails_on_overflow ... ok
test instructions::process_fees::tests::calculate_share_handles_u64_max_without_multiplication_overflow ... ok
test instructions::process_fees::tests::calculate_share_returns_exact_whole_number_share ... ok
test instructions::process_fees::tests::calculate_share_returns_full_amount_for_full_basis_points ... ok
test instructions::process_fees::tests::calculate_share_returns_zero_for_zero_amount ... ok
test instructions::process_fees::tests::calculate_share_returns_zero_for_zero_basis_points ... ok
test instructions::process_fees::tests::calculate_share_rounds_down ... ok
test instructions::process_fees::tests::company_cap_overflow_redirected_to_liquidity_is_conserved ... ok
test instructions::process_fees::tests::founder_cap_overflow_redirected_to_liquidity_is_conserved ... ok
test instructions::process_fees::tests::fully_capped_company_and_founder_allocations_go_to_liquidity ... ok
test instructions::process_fees::tests::locked_normal_allocation_is_conserved ... ok
test instructions::process_fees::tests::rounding_remainder_is_assigned_to_reserve ... ok
test instructions::process_fees::tests::simultaneous_company_and_founder_overflow_is_conserved ... ok
test state::treasury::tests::buyback_accounting_fails_closed_on_overflow ... ok
test state::treasury::tests::buyback_accounting_is_invalid_when_values_do_not_balance ... ok
test state::treasury::tests::buyback_accounting_is_valid_when_lifetime_equals_pending_plus_released ... ok
test state::treasury::tests::company_accounting_fails_closed_on_overflow ... ok
test state::treasury::tests::company_accounting_is_invalid_when_values_do_not_balance ... ok
test state::treasury::tests::company_accounting_is_valid_when_lifetime_equals_pending_plus_released ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_buyback_is_corrupted ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_company_is_corrupted ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_founder_is_corrupted ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_liquidity_is_corrupted ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_multiple_buckets_are_corrupted ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_reserve_is_corrupted ... ok
test state::treasury::tests::execution_accounting_is_valid_when_every_bucket_balances ... ok
test state::treasury::tests::founder_accounting_fails_closed_on_overflow ... ok
test state::treasury::tests::founder_accounting_is_invalid_when_values_do_not_balance ... ok
test state::treasury::tests::founder_accounting_is_valid_when_lifetime_equals_pending_plus_released ... ok
test state::treasury::tests::liquidity_accounting_fails_closed_on_overflow ... ok
test state::treasury::tests::liquidity_accounting_is_invalid_when_values_do_not_balance ... ok
test state::treasury::tests::liquidity_accounting_is_valid_when_lifetime_equals_pending_plus_released ... ok
test state::treasury::tests::overflow_with_max_lifetime_fails_closed_for_every_bucket ... ok
test state::treasury::tests::reserve_accounting_fails_closed_on_overflow ... ok
test state::treasury::tests::reserve_accounting_is_invalid_when_values_do_not_balance ... ok
test state::treasury::tests::reserve_accounting_is_valid_when_lifetime_equals_pending_plus_released ... ok
test state::treasury::tests::zero_balances_are_valid_accounting ... ok
test test_id ... ok

test result: ok. 122 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests treasury_router

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Recent Git history

```text
988930c (HEAD -> phase2-autonomy) security: fuzz complete process fees transition
93073cf security: add core fuzz suite and fix accounting overflow
6f5f96c Add initial libFuzzer security framework and release-limit fuzz target
aec6082 (tag: v1.0.0-security-baseline) RBVR local security gate fully passing
32e4606 Remove outdated generated security baseline snapshot
```

## Working tree

```text
M fuzz/Cargo.toml
 M programs/treasury-router/src/engines/execution_guard.rs
 M programs/treasury-router/src/engines/mod.rs
 M programs/treasury-router/src/events/mod.rs
 M programs/treasury-router/src/instructions/buyback.rs
 M programs/treasury-router/src/instructions/company.rs
 M programs/treasury-router/src/instructions/founder.rs
 M programs/treasury-router/src/instructions/liquidity.rs
 M programs/treasury-router/src/instructions/process_fees.rs
 M programs/treasury-router/src/instructions/reserve.rs
 M programs/treasury-router/src/lib.rs
 M tests/rbvr_protocol.ts
?? .rbvr-devnet-audit/
?? .rbvr-fuzz-runs/
?? .rbvr-security-audit/
?? RBVR-AUTHORITY-LOCKDOWN-AUDIT.md
?? RBVR-AUTHORITY-MAP.md
?? RBVR-DEVNET-DEPLOYMENT-AUDIT.md
?? RBVR-GLOBAL-INVARIANT-MAP.md
?? RBVR-IMMUTABILITY-AUDIT.md
?? RBVR-SECURITY-AUDIT-LATEST.md
?? fuzz/fuzz_targets/process_release.rs
?? programs/treasury-router/src/engines/release.rs
?? rbvr-add-double-initialize-test-v2.py
?? rbvr-add-double-initialize-test.py
?? rbvr-add-release-permutation-test.py
?? rbvr-authority-lockdown-audit.py
?? rbvr-authority-pass.sh
?? rbvr-autonomous-release-pass-v2.sh
?? rbvr-autonomous-release-pass-v3.sh
?? rbvr-autonomous-release-pass.sh
?? rbvr-deep-security-audit.sh
?? rbvr-deploy-devnet.sh
?? rbvr-devnet-deployment-audit.sh
?? rbvr-double-initialize-pass-v2.sh
?? rbvr-double-initialize-pass.sh
?? rbvr-global-invariant-map.py
?? rbvr-global-invariant-pass-v2.py
?? rbvr-global-invariant-pass-v2.sh
?? rbvr-global-invariant-pass.sh
?? rbvr-immutability-audit.py
?? rbvr-patch-autonomous-tests.py
?? rbvr-permissionless-release-pass-v2.sh
?? rbvr-permissionless-release-pass-v3.sh
?? rbvr-permissionless-release-pass.sh
?? rbvr-red-team-rt001.py
?? rbvr-red-team-rt001.sh
?? rbvr-release-authority-pass.sh
?? rbvr-release-fuzz-pass.sh
?? rbvr-release-fuzz-source.txt
?? rbvr-remove-release-authority.py
?? rbvr-remove-release-event-authority-v2.py
?? rbvr-remove-release-event-authority.py
?? rbvr-remove-release-test-authority.py
?? rbvr-start-background-fuzz.sh
```

## RT-001 exit assessment

RT-001 is an inventory phase, not a security pass.

The inventory is complete when every public instruction and high-value
account path has an assigned adversarial test. Findings marked OPEN or
PLANNED are attack hypotheses, not confirmed vulnerabilities.

## Recommended RT-002 target

**Initialization and account-substitution assault.**

Attempt repeated initialization, alternate PDAs, wrong bumps, wrong
owners, wrong mints, wrong vaults, duplicate accounts, and out-of-order
initialization across all canonical protocol accounts.
