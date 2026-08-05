# RBVR Documentation Source Audit

Generated: 2026-08-05T19:49:27+00:00

Commit before documentation patch: `868306789cbd8632c3d7cdea12e66038192b94b9`

## Scope

- `ACCOUNT_MODEL.md`
- `ARCHITECTURE.md`
- `AUDITING.md`
- `BEAVERNOMICS.md`
- `CI_CD.md`
- `DEPLOYMENT.md`
- `DEVELOPER_GUIDE.md`
- `DOCUMENTATION_INDEX.md`
- `INSTRUCTION_REFERENCE.md`
- `MAINNET_READINESS.md`
- `OPERATOR_GUIDE.md`
- `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md`
- `RBVR-SECURITY-REVIEW.md`
- `README.md`
- `SECURITY.md`
- `TESTING.md`
- `THREAT_MODEL.md`

## Classification

- **VERIFIED**: direct source, IDL, test, or deployment evidence found.
- **QUALIFY**: partly true but requires narrower wording.
- **INFERRED**: plausible but not directly established by the automated rules.
- **UNSUPPORTED**: the stated implementation claim was not found.

## Summary

- QUALIFY: 5
- UNSUPPORTED: 4
- VERIFIED: 56

## Automatically changed files

- `BEAVERNOMICS.md`
- `README.md`

## Findings

### VERIFIED: `ARCHITECTURE.md:15`

Category: `overflow`

> - overflow to Liquidity Growth;

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `ARCHITECTURE.md:17`

Category: `permissionless`

> - constrained permissionless Reserve deployment.

The Spillway source describes permissionless invocation and does not expose caller-selected policy or destinations.

Evidence:

- `programs/treasury-router/src/errors/mod.rs:124` — `#[msg("The Spillway destination is invalid.")]`
- `programs/treasury-router/src/errors/mod.rs:125` — `InvalidSpillwayDestination,`
- `programs/treasury-router/src/errors/mod.rs:127` — `#[msg("The Spillway would breach the protected Reserve floor.")]`
- `programs/treasury-router/src/events/mod.rs:118` — `/// Emitted after a successful permissionless Reserve Spillway release.`
- `programs/treasury-router/src/events/mod.rs:120` — `pub struct SpillwayReleaseExecuted {`
- `programs/treasury-router/src/instructions/buyback.rs:177` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/company.rs:99` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/founder.rs:99` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/liquidity.rs:177` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/mod.rs:17` — `pub mod spillway_release;`
- `programs/treasury-router/src/instructions/mod.rs:33` — `pub use spillway_release::*;`
- `programs/treasury-router/src/instructions/reserve.rs:184` — `/// Permissionless transaction caller and fee payer.`

### VERIFIED: `ARCHITECTURE.md:29`

Category: `allocation`

> - Reserve: 3000;

The fixed basis-point configuration is present in constants, initialization, or verified state.

Evidence:

- `programs/treasury-router/src/constants.rs:40` — `pub const INITIAL_RESERVE_BPS: u16 = 3_000;`
- `programs/treasury-router/src/constants.rs:41` — `pub const INITIAL_BUYBACK_BURN_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:42` — `pub const INITIAL_LIQUIDITY_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:43` — `pub const INITIAL_COMPANY_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:44` — `pub const INITIAL_FOUNDER_BPS: u16 = 1_000;`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:5` — `BPS_DENOMINATOR, INITIAL_BUYBACK_BURN_BPS, INITIAL_COMPANY_BPS, INITIAL_FOUNDER_BPS,`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:6` — `INITIAL_LIQUIDITY_BPS, INITIAL_RESERVE_BPS, PROTOCOL_CONFIG_SEED, PROTOCOL_CONFIG_VERSION,`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:52` — `let total_bps = INITIAL_RESERVE_BPS`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:53` — `.checked_add(INITIAL_BUYBACK_BURN_BPS)`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:54` — `.and_then(|value| value.checked_add(INITIAL_LIQUIDITY_BPS))`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:55` — `.and_then(|value| value.checked_add(INITIAL_COMPANY_BPS))`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:56` — `.and_then(|value| value.checked_add(INITIAL_FOUNDER_BPS))`

### VERIFIED: `ARCHITECTURE.md:30`

Category: `allocation`

> - Buyback/Burn: 2000;

The fixed basis-point configuration is present in constants, initialization, or verified state.

Evidence:

- `programs/treasury-router/src/constants.rs:40` — `pub const INITIAL_RESERVE_BPS: u16 = 3_000;`
- `programs/treasury-router/src/constants.rs:41` — `pub const INITIAL_BUYBACK_BURN_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:42` — `pub const INITIAL_LIQUIDITY_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:43` — `pub const INITIAL_COMPANY_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:44` — `pub const INITIAL_FOUNDER_BPS: u16 = 1_000;`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:5` — `BPS_DENOMINATOR, INITIAL_BUYBACK_BURN_BPS, INITIAL_COMPANY_BPS, INITIAL_FOUNDER_BPS,`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:6` — `INITIAL_LIQUIDITY_BPS, INITIAL_RESERVE_BPS, PROTOCOL_CONFIG_SEED, PROTOCOL_CONFIG_VERSION,`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:52` — `let total_bps = INITIAL_RESERVE_BPS`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:53` — `.checked_add(INITIAL_BUYBACK_BURN_BPS)`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:54` — `.and_then(|value| value.checked_add(INITIAL_LIQUIDITY_BPS))`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:55` — `.and_then(|value| value.checked_add(INITIAL_COMPANY_BPS))`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:56` — `.and_then(|value| value.checked_add(INITIAL_FOUNDER_BPS))`

### VERIFIED: `ARCHITECTURE.md:31`

Category: `allocation`

> - Liquidity: 2000;

The fixed basis-point configuration is present in constants, initialization, or verified state.

Evidence:

- `programs/treasury-router/src/constants.rs:40` — `pub const INITIAL_RESERVE_BPS: u16 = 3_000;`
- `programs/treasury-router/src/constants.rs:41` — `pub const INITIAL_BUYBACK_BURN_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:42` — `pub const INITIAL_LIQUIDITY_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:43` — `pub const INITIAL_COMPANY_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:44` — `pub const INITIAL_FOUNDER_BPS: u16 = 1_000;`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:5` — `BPS_DENOMINATOR, INITIAL_BUYBACK_BURN_BPS, INITIAL_COMPANY_BPS, INITIAL_FOUNDER_BPS,`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:6` — `INITIAL_LIQUIDITY_BPS, INITIAL_RESERVE_BPS, PROTOCOL_CONFIG_SEED, PROTOCOL_CONFIG_VERSION,`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:52` — `let total_bps = INITIAL_RESERVE_BPS`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:53` — `.checked_add(INITIAL_BUYBACK_BURN_BPS)`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:54` — `.and_then(|value| value.checked_add(INITIAL_LIQUIDITY_BPS))`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:55` — `.and_then(|value| value.checked_add(INITIAL_COMPANY_BPS))`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:56` — `.and_then(|value| value.checked_add(INITIAL_FOUNDER_BPS))`

### VERIFIED: `ARCHITECTURE.md:32`

Category: `allocation`

> - Company: 2000;

The fixed basis-point configuration is present in constants, initialization, or verified state.

Evidence:

- `programs/treasury-router/src/constants.rs:40` — `pub const INITIAL_RESERVE_BPS: u16 = 3_000;`
- `programs/treasury-router/src/constants.rs:41` — `pub const INITIAL_BUYBACK_BURN_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:42` — `pub const INITIAL_LIQUIDITY_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:43` — `pub const INITIAL_COMPANY_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:44` — `pub const INITIAL_FOUNDER_BPS: u16 = 1_000;`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:5` — `BPS_DENOMINATOR, INITIAL_BUYBACK_BURN_BPS, INITIAL_COMPANY_BPS, INITIAL_FOUNDER_BPS,`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:6` — `INITIAL_LIQUIDITY_BPS, INITIAL_RESERVE_BPS, PROTOCOL_CONFIG_SEED, PROTOCOL_CONFIG_VERSION,`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:52` — `let total_bps = INITIAL_RESERVE_BPS`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:53` — `.checked_add(INITIAL_BUYBACK_BURN_BPS)`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:54` — `.and_then(|value| value.checked_add(INITIAL_LIQUIDITY_BPS))`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:55` — `.and_then(|value| value.checked_add(INITIAL_COMPANY_BPS))`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:56` — `.and_then(|value| value.checked_add(INITIAL_FOUNDER_BPS))`

### VERIFIED: `ARCHITECTURE.md:33`

Category: `allocation`

> - Founder: 1000.

The fixed basis-point configuration is present in constants, initialization, or verified state.

Evidence:

- `programs/treasury-router/src/constants.rs:40` — `pub const INITIAL_RESERVE_BPS: u16 = 3_000;`
- `programs/treasury-router/src/constants.rs:41` — `pub const INITIAL_BUYBACK_BURN_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:42` — `pub const INITIAL_LIQUIDITY_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:43` — `pub const INITIAL_COMPANY_BPS: u16 = 2_000;`
- `programs/treasury-router/src/constants.rs:44` — `pub const INITIAL_FOUNDER_BPS: u16 = 1_000;`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:5` — `BPS_DENOMINATOR, INITIAL_BUYBACK_BURN_BPS, INITIAL_COMPANY_BPS, INITIAL_FOUNDER_BPS,`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:6` — `INITIAL_LIQUIDITY_BPS, INITIAL_RESERVE_BPS, PROTOCOL_CONFIG_SEED, PROTOCOL_CONFIG_VERSION,`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:52` — `let total_bps = INITIAL_RESERVE_BPS`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:53` — `.checked_add(INITIAL_BUYBACK_BURN_BPS)`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:54` — `.and_then(|value| value.checked_add(INITIAL_LIQUIDITY_BPS))`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:55` — `.and_then(|value| value.checked_add(INITIAL_COMPANY_BPS))`
- `programs/treasury-router/src/instructions/initialize_protocol_config.rs:56` — `.and_then(|value| value.checked_add(INITIAL_FOUNDER_BPS))`

### VERIFIED: `ARCHITECTURE.md:53`

Category: `reserve`

> Stores the Reserve floor, deployment ratio, cooldown, last deployment time, and lifetime deployed amount.

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `ARCHITECTURE.md:83`

Category: `permissionless`

> The Spillway is permissionless but policy-constrained. A caller may trigger eligible Reserve surplus deployment, but cannot choose the destination, bypass the minimum floor, alter the deployment ratio, or ignore cooldown.

The Spillway source describes permissionless invocation and does not expose caller-selected policy or destinations.

Evidence:

- `programs/treasury-router/src/errors/mod.rs:124` — `#[msg("The Spillway destination is invalid.")]`
- `programs/treasury-router/src/errors/mod.rs:125` — `InvalidSpillwayDestination,`
- `programs/treasury-router/src/errors/mod.rs:127` — `#[msg("The Spillway would breach the protected Reserve floor.")]`
- `programs/treasury-router/src/events/mod.rs:118` — `/// Emitted after a successful permissionless Reserve Spillway release.`
- `programs/treasury-router/src/events/mod.rs:120` — `pub struct SpillwayReleaseExecuted {`
- `programs/treasury-router/src/instructions/buyback.rs:177` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/company.rs:99` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/founder.rs:99` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/liquidity.rs:177` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/mod.rs:17` — `pub mod spillway_release;`
- `programs/treasury-router/src/instructions/mod.rs:33` — `pub use spillway_release::*;`
- `programs/treasury-router/src/instructions/reserve.rs:184` — `/// Permissionless transaction caller and fee payer.`

### VERIFIED: `AUDITING.md:19`

Category: `overflow`

> 9. overflow to Liquidity Growth;

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `AUDITING.md:20`

Category: `atomicity`

> 10. release atomicity;

Atomic transfer/accounting expectations are supported by source comments and mutation tests.

Evidence:

- `programs/treasury-router/src/events/mod.rs:47` — `/// The settlement-token transfer and accounting update occur atomically.`
- `programs/treasury-router/src/events/mod.rs:69` — `/// transfer and treasury execution-accounting update occur atomically.`
- `programs/treasury-router/src/events/mod.rs:95` — `/// transfer and treasury execution-accounting update occur atomically.`
- `programs/treasury-router/src/instructions/company.rs:110` — `/// The settlement-token transfer and treasury accounting update are atomic.`
- `programs/treasury-router/src/instructions/founder.rs:110` — `/// The settlement-token transfer and treasury accounting update are atomic.`
- `tests/rbvr_protocol.ts:462` — `it("RT-002A rejects repeated protocol-config initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:551` — `it("RT-002A rejects repeated treasury initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:653` — `it("RT-002A rejects repeated founder initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:741` — `it("RT-002A rejects repeated company initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:1216` — `it("RT-003 rejects a zero-value deposit without mutation", async () => {`
- `tests/rbvr_protocol.ts:1451` — `it("RT-002A rejects repeated execution-config initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:1783` — `it("RT-002B rejects a fake settlement mint without mutation", async () => {`

### VERIFIED: `AUDITING.md:22`

Category: `reserve`

> 12. Spillway cooldown and floor;

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `AUDITING.md:23`

Category: `replay`

> 13. replay protection;

Replay rejection is covered by program guards and adversarial tests.

Evidence:

- `programs/treasury-router/src/errors/mod.rs:27` — `NoUnprocessedFees,`
- `programs/treasury-router/src/errors/mod.rs:113` — `ReserveDeploymentCooldownActive,`
- `programs/treasury-router/src/instructions/process_fees.rs:132` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/process_fees.rs:401` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/spillway_release.rs:136` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `programs/treasury-router/src/state/reserve_policy.rs:110` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `tests/rbvr_protocol.ts:1559` — `it("RT-004 rejects fee-processing replay without mutating accounting", async () => {`
- `tests/rbvr_protocol.ts:1608` — `/NoUnprocessedFees|no unprocessed fees/i,`
- `tests/rbvr_protocol.ts:3380` — `it("RT-005D rejects an immediate Spillway replay without mutation", async () => {`
- `tests/rbvr_protocol.ts:3437` — `/ReserveDeploymentCooldownActive|cooldown is still active|cooldown/i,`
- `target/idl/treasury_router.json:2407` — `"name": "NoUnprocessedFees",`
- `target/idl/treasury_router.json:2552` — `"name": "ReserveDeploymentCooldownActive",`

### VERIFIED: `AUDITING.md:43`

Category: `overflow`

> Account for overflow redirection.

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `AUDITING.md:65`

Category: `overflow`

> - overflow;

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `AUDITING.md:72`

Category: `reserve`

> - cooldown.

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `BEAVERNOMICS.md:15`

Category: `overflow`

> Company allocation is capped per period, not for the life of the protocol. When the current-period cap is reached, excess Company allocation is redirected to Liquidity Growth.

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### UNSUPPORTED: `BEAVERNOMICS.md:19`

Category: `progressive_founder`

> Founder compensation is capped per period and follows a progressive marginal volume schedule. Later eligible volume may receive a lower marginal rate, while total compensation can continue increasing in absolute terms.

The implementation contains `current_tier` and period-cap state, but this audit did not find a progressive marginal-rate calculation.

Evidence:

- `programs/treasury-router/src/engines/company.rs:61` — `.period_cap`
- `programs/treasury-router/src/engines/company.rs:87` — `company.spent_current_period = company.period_cap;`
- `programs/treasury-router/src/engines/founder.rs:61` — `.period_cap`
- `programs/treasury-router/src/engines/founder.rs:87` — `founder.earned_current_period = founder.period_cap;`
- `programs/treasury-router/src/engines/integrity_firewall.rs:517` — `period_cap: 1_000,`
- `programs/treasury-router/src/engines/integrity_firewall.rs:522` — `current_tier: 0,`
- `programs/treasury-router/src/engines/integrity_firewall.rs:532` — `period_cap: 1_000,`
- `programs/treasury-router/src/events/mod.rs:86` — `pub founder_period_cap: u64,`
- `programs/treasury-router/src/events/mod.rs:112` — `pub company_period_cap: u64,`
- `programs/treasury-router/src/instructions/company.rs:166` — `company_period_cap: ctx.accounts.company_state.period_cap,`
- `programs/treasury-router/src/instructions/company.rs:223` — `ctx.accounts.company_state.period_cap`
- `programs/treasury-router/src/instructions/founder.rs:166` — `founder_period_cap: ctx.accounts.founder_state.period_cap,`

### VERIFIED: `BEAVERNOMICS.md:21`

Category: `overflow`

> When the Founder period cap is reached, excess Founder allocation is redirected to Liquidity Growth.

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `BEAVERNOMICS.md:41`

Category: `reserve`

> Reserve capital is retained to support protocol resilience. Surplus deployment is governed by the Reserve Policy and Spillway rather than caller discretion.

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### QUALIFY: `INSTRUCTION_REFERENCE.md:33`

Category: `immutable`

> Creates the immutable or constrained Reserve surplus policy.

Configuration may be initialized once and locked with `updates_enabled = false`, but the deployed Solana program remains upgradeable until upgrade authority is revoked.

Evidence:

- `programs/treasury-router/src/engines/integrity_firewall.rs:472` — `updates_enabled: false,`
- `programs/treasury-router/src/engines/sentinel.rs:328` — `pub configuration_locked: bool,`
- `programs/treasury-router/src/engines/sentinel.rs:388` — `let configuration_locked = !protocol_config.updates_enabled;`
- `programs/treasury-router/src/engines/sentinel.rs:390` — `if !configuration_locked {`
- `programs/treasury-router/src/engines/sentinel.rs:454` — `configuration_locked,`
- `programs/treasury-router/src/engines/sentinel.rs:487` — `updates_enabled: false,`
- `programs/treasury-router/src/engines/sentinel.rs:544` — `assert!(report.configuration_locked);`
- `programs/treasury-router/src/engines/sentinel.rs:573` — `fn configuration_updates_enabled_fails_closed() {`
- `programs/treasury-router/src/engines/sentinel.rs:578` — `config.updates_enabled = true;`
- `programs/treasury-router/src/engines/sentinel.rs:582` — `assert!(!report.configuration_locked);`
- `programs/treasury-router/src/instructions/initialize.rs:11` — `init,`
- `programs/treasury-router/src/instructions/initialize_company.rs:20` — `init,`

### VERIFIED: `INSTRUCTION_REFERENCE.md:43`

Category: `overflow`

> Allocates unprocessed settlement fees into the five accounting buckets. It must conserve value, use locked configuration, enforce caps, redirect capped overflow to Liquidity Growth, and reject replay.

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `INSTRUCTION_REFERENCE.md:47`

Category: `atomicity`

> Separate release paths move pending balances to Reserve, Buyback/Burn, Liquidity, Company, and Founder destinations. Token transfer and accounting update must be atomic.

Atomic transfer/accounting expectations are supported by source comments and mutation tests.

Evidence:

- `programs/treasury-router/src/events/mod.rs:47` — `/// The settlement-token transfer and accounting update occur atomically.`
- `programs/treasury-router/src/events/mod.rs:69` — `/// transfer and treasury execution-accounting update occur atomically.`
- `programs/treasury-router/src/events/mod.rs:95` — `/// transfer and treasury execution-accounting update occur atomically.`
- `programs/treasury-router/src/instructions/company.rs:110` — `/// The settlement-token transfer and treasury accounting update are atomic.`
- `programs/treasury-router/src/instructions/founder.rs:110` — `/// The settlement-token transfer and treasury accounting update are atomic.`
- `tests/rbvr_protocol.ts:462` — `it("RT-002A rejects repeated protocol-config initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:551` — `it("RT-002A rejects repeated treasury initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:653` — `it("RT-002A rejects repeated founder initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:741` — `it("RT-002A rejects repeated company initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:1216` — `it("RT-003 rejects a zero-value deposit without mutation", async () => {`
- `tests/rbvr_protocol.ts:1451` — `it("RT-002A rejects repeated execution-config initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:1783` — `it("RT-002B rejects a fake settlement mint without mutation", async () => {`

### VERIFIED: `INSTRUCTION_REFERENCE.md:51`

Category: `reserve`

> Deploys eligible Reserve surplus while preserving the Reserve floor, respecting the deployment ratio and cooldown, and using the locked Liquidity destination.

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `INSTRUCTION_REFERENCE.md:55`

Category: `replay`

> The program should reject fake PDAs, wrong owners, wrong discriminators, substituted mints, fake vaults, duplicate destinations, wrong token programs, repeated initialization, replay, empty processing, and cooldown bypass.

Replay rejection is covered by program guards and adversarial tests.

Evidence:

- `programs/treasury-router/src/errors/mod.rs:27` — `NoUnprocessedFees,`
- `programs/treasury-router/src/errors/mod.rs:113` — `ReserveDeploymentCooldownActive,`
- `programs/treasury-router/src/instructions/process_fees.rs:132` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/process_fees.rs:401` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/spillway_release.rs:136` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `programs/treasury-router/src/state/reserve_policy.rs:110` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `tests/rbvr_protocol.ts:1559` — `it("RT-004 rejects fee-processing replay without mutating accounting", async () => {`
- `tests/rbvr_protocol.ts:1608` — `/NoUnprocessedFees|no unprocessed fees/i,`
- `tests/rbvr_protocol.ts:3380` — `it("RT-005D rejects an immediate Spillway replay without mutation", async () => {`
- `tests/rbvr_protocol.ts:3437` — `/ReserveDeploymentCooldownActive|cooldown is still active|cooldown/i,`
- `target/idl/treasury_router.json:2407` — `"name": "NoUnprocessedFees",`
- `target/idl/treasury_router.json:2552` — `"name": "ReserveDeploymentCooldownActive",`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:57`

Category: `reserve`

> - `programs/treasury-router/src/instructions/spillway_release.rs`

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:754`

Category: `reserve`

> - `cooldown_seconds`: `"i64"`

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:816`

Category: `reserve`

> - `spillway_release.rs`

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:822`

Category: `reserve`

> - `Spillway` — source/IDL: found; docs: ARCHITECTURE.md, INSTRUCTION_REFERENCE.md

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### UNSUPPORTED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:824`

Category: `progressive_founder`

> - `progressive marginal` — source/IDL: not found; docs: not used

The implementation contains `current_tier` and period-cap state, but this audit did not find a progressive marginal-rate calculation.

Evidence:

- `programs/treasury-router/src/engines/company.rs:61` — `.period_cap`
- `programs/treasury-router/src/engines/company.rs:87` — `company.spent_current_period = company.period_cap;`
- `programs/treasury-router/src/engines/founder.rs:61` — `.period_cap`
- `programs/treasury-router/src/engines/founder.rs:87` — `founder.earned_current_period = founder.period_cap;`
- `programs/treasury-router/src/engines/integrity_firewall.rs:517` — `period_cap: 1_000,`
- `programs/treasury-router/src/engines/integrity_firewall.rs:522` — `current_tier: 0,`
- `programs/treasury-router/src/engines/integrity_firewall.rs:532` — `period_cap: 1_000,`
- `programs/treasury-router/src/events/mod.rs:86` — `pub founder_period_cap: u64,`
- `programs/treasury-router/src/events/mod.rs:112` — `pub company_period_cap: u64,`
- `programs/treasury-router/src/instructions/company.rs:166` — `company_period_cap: ctx.accounts.company_state.period_cap,`
- `programs/treasury-router/src/instructions/company.rs:223` — `ctx.accounts.company_state.period_cap`
- `programs/treasury-router/src/instructions/founder.rs:166` — `founder_period_cap: ctx.accounts.founder_state.period_cap,`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:825`

Category: `overflow`

> - `overflow` — source/IDL: found; docs: ARCHITECTURE.md, INSTRUCTION_REFERENCE.md

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### QUALIFY: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:843`

Category: `immutable`

> - Distinguish immutable account data from an upgradeable program.

Configuration may be initialized once and locked with `updates_enabled = false`, but the deployed Solana program remains upgradeable until upgrade authority is revoked.

Evidence:

- `programs/treasury-router/src/engines/integrity_firewall.rs:472` — `updates_enabled: false,`
- `programs/treasury-router/src/engines/sentinel.rs:328` — `pub configuration_locked: bool,`
- `programs/treasury-router/src/engines/sentinel.rs:388` — `let configuration_locked = !protocol_config.updates_enabled;`
- `programs/treasury-router/src/engines/sentinel.rs:390` — `if !configuration_locked {`
- `programs/treasury-router/src/engines/sentinel.rs:454` — `configuration_locked,`
- `programs/treasury-router/src/engines/sentinel.rs:487` — `updates_enabled: false,`
- `programs/treasury-router/src/engines/sentinel.rs:544` — `assert!(report.configuration_locked);`
- `programs/treasury-router/src/engines/sentinel.rs:573` — `fn configuration_updates_enabled_fails_closed() {`
- `programs/treasury-router/src/engines/sentinel.rs:578` — `config.updates_enabled = true;`
- `programs/treasury-router/src/engines/sentinel.rs:582` — `assert!(!report.configuration_locked);`
- `programs/treasury-router/src/instructions/initialize.rs:11` — `init,`
- `programs/treasury-router/src/instructions/initialize_company.rs:20` — `init,`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:851`

Category: `permissionless`

> - Confirm any caller may invoke the instruction.

The Spillway source describes permissionless invocation and does not expose caller-selected policy or destinations.

Evidence:

- `programs/treasury-router/src/errors/mod.rs:124` — `#[msg("The Spillway destination is invalid.")]`
- `programs/treasury-router/src/errors/mod.rs:125` — `InvalidSpillwayDestination,`
- `programs/treasury-router/src/errors/mod.rs:127` — `#[msg("The Spillway would breach the protected Reserve floor.")]`
- `programs/treasury-router/src/events/mod.rs:118` — `/// Emitted after a successful permissionless Reserve Spillway release.`
- `programs/treasury-router/src/events/mod.rs:120` — `pub struct SpillwayReleaseExecuted {`
- `programs/treasury-router/src/instructions/buyback.rs:177` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/company.rs:99` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/founder.rs:99` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/liquidity.rs:177` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/mod.rs:17` — `pub mod spillway_release;`
- `programs/treasury-router/src/instructions/mod.rs:33` — `pub use spillway_release::*;`
- `programs/treasury-router/src/instructions/reserve.rs:184` — `/// Permissionless transaction caller and fee payer.`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:869`

Category: `overflow`

> - Confirm both Company and Founder overflow route to Liquidity Growth.

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:870`

Category: `overflow`

> - Confirm accounting conservation after overflow.

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:885`

Category: `overflow`

> 4. Review process-fee arithmetic, cap logic, overflow routing, and rounding.

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:886`

Category: `atomicity`

> 5. Review each release handler for atomic transfer and accounting updates.

Atomic transfer/accounting expectations are supported by source comments and mutation tests.

Evidence:

- `programs/treasury-router/src/events/mod.rs:47` — `/// The settlement-token transfer and accounting update occur atomically.`
- `programs/treasury-router/src/events/mod.rs:69` — `/// transfer and treasury execution-accounting update occur atomically.`
- `programs/treasury-router/src/events/mod.rs:95` — `/// transfer and treasury execution-accounting update occur atomically.`
- `programs/treasury-router/src/instructions/company.rs:110` — `/// The settlement-token transfer and treasury accounting update are atomic.`
- `programs/treasury-router/src/instructions/founder.rs:110` — `/// The settlement-token transfer and treasury accounting update are atomic.`
- `tests/rbvr_protocol.ts:462` — `it("RT-002A rejects repeated protocol-config initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:551` — `it("RT-002A rejects repeated treasury initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:653` — `it("RT-002A rejects repeated founder initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:741` — `it("RT-002A rejects repeated company initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:1216` — `it("RT-003 rejects a zero-value deposit without mutation", async () => {`
- `tests/rbvr_protocol.ts:1451` — `it("RT-002A rejects repeated execution-config initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:1783` — `it("RT-002B rejects a fake settlement mint without mutation", async () => {`

### VERIFIED: `RBVR-DOC-SOURCE-AUDIT-20260805T194523Z.md:888`

Category: `reserve`

> 7. Review Reserve floor, deployment ratio, destination, and cooldown.

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### QUALIFY: `RBVR-SECURITY-REVIEW.md:6`

Category: `immutable`

> existing Sentinel V2 immutable-configuration/linkage checks. A configuration

Configuration may be initialized once and locked with `updates_enabled = false`, but the deployed Solana program remains upgradeable until upgrade authority is revoked.

Evidence:

- `programs/treasury-router/src/engines/integrity_firewall.rs:472` — `updates_enabled: false,`
- `programs/treasury-router/src/engines/sentinel.rs:328` — `pub configuration_locked: bool,`
- `programs/treasury-router/src/engines/sentinel.rs:388` — `let configuration_locked = !protocol_config.updates_enabled;`
- `programs/treasury-router/src/engines/sentinel.rs:390` — `if !configuration_locked {`
- `programs/treasury-router/src/engines/sentinel.rs:454` — `configuration_locked,`
- `programs/treasury-router/src/engines/sentinel.rs:487` — `updates_enabled: false,`
- `programs/treasury-router/src/engines/sentinel.rs:544` — `assert!(report.configuration_locked);`
- `programs/treasury-router/src/engines/sentinel.rs:573` — `fn configuration_updates_enabled_fails_closed() {`
- `programs/treasury-router/src/engines/sentinel.rs:578` — `config.updates_enabled = true;`
- `programs/treasury-router/src/engines/sentinel.rs:582` — `assert!(!report.configuration_locked);`
- `programs/treasury-router/src/instructions/initialize.rs:11` — `init,`
- `programs/treasury-router/src/instructions/initialize_company.rs:20` — `init,`

### VERIFIED: `RBVR-SECURITY-REVIEW.md:24`

Category: `atomicity`

> - Solana transaction atomicity rolls the entire operation back.

Atomic transfer/accounting expectations are supported by source comments and mutation tests.

Evidence:

- `programs/treasury-router/src/events/mod.rs:47` — `/// The settlement-token transfer and accounting update occur atomically.`
- `programs/treasury-router/src/events/mod.rs:69` — `/// transfer and treasury execution-accounting update occur atomically.`
- `programs/treasury-router/src/events/mod.rs:95` — `/// transfer and treasury execution-accounting update occur atomically.`
- `programs/treasury-router/src/instructions/company.rs:110` — `/// The settlement-token transfer and treasury accounting update are atomic.`
- `programs/treasury-router/src/instructions/founder.rs:110` — `/// The settlement-token transfer and treasury accounting update are atomic.`
- `tests/rbvr_protocol.ts:462` — `it("RT-002A rejects repeated protocol-config initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:551` — `it("RT-002A rejects repeated treasury initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:653` — `it("RT-002A rejects repeated founder initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:741` — `it("RT-002A rejects repeated company initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:1216` — `it("RT-003 rejects a zero-value deposit without mutation", async () => {`
- `tests/rbvr_protocol.ts:1451` — `it("RT-002A rejects repeated execution-config initialization without mutation", async () => {`
- `tests/rbvr_protocol.ts:1783` — `it("RT-002B rejects a fake settlement mint without mutation", async () => {`

### VERIFIED: `RBVR-SECURITY-REVIEW.md:26`

Category: `overflow`

> No allocation percentage, period cap, overflow route, Waterfall rule, Dam rule,

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### QUALIFY: `RBVR-SECURITY-REVIEW.md:31`

Category: `immutable`

> The snapshot's buyback instruction transfers settlement tokens to the immutable

Configuration may be initialized once and locked with `updates_enabled = false`, but the deployed Solana program remains upgradeable until upgrade authority is revoked.

Evidence:

- `programs/treasury-router/src/engines/integrity_firewall.rs:472` — `updates_enabled: false,`
- `programs/treasury-router/src/engines/sentinel.rs:328` — `pub configuration_locked: bool,`
- `programs/treasury-router/src/engines/sentinel.rs:388` — `let configuration_locked = !protocol_config.updates_enabled;`
- `programs/treasury-router/src/engines/sentinel.rs:390` — `if !configuration_locked {`
- `programs/treasury-router/src/engines/sentinel.rs:454` — `configuration_locked,`
- `programs/treasury-router/src/engines/sentinel.rs:487` — `updates_enabled: false,`
- `programs/treasury-router/src/engines/sentinel.rs:544` — `assert!(report.configuration_locked);`
- `programs/treasury-router/src/engines/sentinel.rs:573` — `fn configuration_updates_enabled_fails_closed() {`
- `programs/treasury-router/src/engines/sentinel.rs:578` — `config.updates_enabled = true;`
- `programs/treasury-router/src/engines/sentinel.rs:582` — `assert!(!report.configuration_locked);`
- `programs/treasury-router/src/instructions/initialize.rs:11` — `init,`
- `programs/treasury-router/src/instructions/initialize_company.rs:20` — `init,`

### UNSUPPORTED: `README.md:17`

Category: `progressive_founder`

> Company and Founder controls are period-based. When either period cap is reached, overflow is redirected to Liquidity Growth. Founder compensation follows a progressive marginal volume schedule in which the applicable rate declines as eligible volume increases.

The implementation contains `current_tier` and period-cap state, but this audit did not find a progressive marginal-rate calculation.

Evidence:

- `programs/treasury-router/src/engines/company.rs:61` — `.period_cap`
- `programs/treasury-router/src/engines/company.rs:87` — `company.spent_current_period = company.period_cap;`
- `programs/treasury-router/src/engines/founder.rs:61` — `.period_cap`
- `programs/treasury-router/src/engines/founder.rs:87` — `founder.earned_current_period = founder.period_cap;`
- `programs/treasury-router/src/engines/integrity_firewall.rs:517` — `period_cap: 1_000,`
- `programs/treasury-router/src/engines/integrity_firewall.rs:522` — `current_tier: 0,`
- `programs/treasury-router/src/engines/integrity_firewall.rs:532` — `period_cap: 1_000,`
- `programs/treasury-router/src/events/mod.rs:86` — `pub founder_period_cap: u64,`
- `programs/treasury-router/src/events/mod.rs:112` — `pub company_period_cap: u64,`
- `programs/treasury-router/src/instructions/company.rs:166` — `company_period_cap: ctx.accounts.company_state.period_cap,`
- `programs/treasury-router/src/instructions/company.rs:223` — `ctx.accounts.company_state.period_cap`
- `programs/treasury-router/src/instructions/founder.rs:166` — `founder_period_cap: ctx.accounts.founder_state.period_cap,`

### VERIFIED: `README.md:21`

Category: `overflow`

> RBVR is built around deterministic arithmetic, canonical PDAs, locked destinations, one-time initialization, replay resistance, account-link validation, cap enforcement, overflow redirection, constrained Reserve deployment, and independently verifiable on-chain state.

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### QUALIFY: `SECURITY.md:11`

Category: `immutable`

> - immutable or disabled updates where intended;

Configuration may be initialized once and locked with `updates_enabled = false`, but the deployed Solana program remains upgradeable until upgrade authority is revoked.

Evidence:

- `programs/treasury-router/src/engines/integrity_firewall.rs:472` — `updates_enabled: false,`
- `programs/treasury-router/src/engines/sentinel.rs:328` — `pub configuration_locked: bool,`
- `programs/treasury-router/src/engines/sentinel.rs:388` — `let configuration_locked = !protocol_config.updates_enabled;`
- `programs/treasury-router/src/engines/sentinel.rs:390` — `if !configuration_locked {`
- `programs/treasury-router/src/engines/sentinel.rs:454` — `configuration_locked,`
- `programs/treasury-router/src/engines/sentinel.rs:487` — `updates_enabled: false,`
- `programs/treasury-router/src/engines/sentinel.rs:544` — `assert!(report.configuration_locked);`
- `programs/treasury-router/src/engines/sentinel.rs:573` — `fn configuration_updates_enabled_fails_closed() {`
- `programs/treasury-router/src/engines/sentinel.rs:578` — `config.updates_enabled = true;`
- `programs/treasury-router/src/engines/sentinel.rs:582` — `assert!(!report.configuration_locked);`
- `programs/treasury-router/src/instructions/initialize.rs:11` — `init,`
- `programs/treasury-router/src/instructions/initialize_company.rs:20` — `init,`

### VERIFIED: `SECURITY.md:17`

Category: `replay`

> - replay protection;

Replay rejection is covered by program guards and adversarial tests.

Evidence:

- `programs/treasury-router/src/errors/mod.rs:27` — `NoUnprocessedFees,`
- `programs/treasury-router/src/errors/mod.rs:113` — `ReserveDeploymentCooldownActive,`
- `programs/treasury-router/src/instructions/process_fees.rs:132` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/process_fees.rs:401` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/spillway_release.rs:136` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `programs/treasury-router/src/state/reserve_policy.rs:110` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `tests/rbvr_protocol.ts:1559` — `it("RT-004 rejects fee-processing replay without mutating accounting", async () => {`
- `tests/rbvr_protocol.ts:1608` — `/NoUnprocessedFees|no unprocessed fees/i,`
- `tests/rbvr_protocol.ts:3380` — `it("RT-005D rejects an immediate Spillway replay without mutation", async () => {`
- `tests/rbvr_protocol.ts:3437` — `/ReserveDeploymentCooldownActive|cooldown is still active|cooldown/i,`
- `target/idl/treasury_router.json:2407` — `"name": "NoUnprocessedFees",`
- `target/idl/treasury_router.json:2552` — `"name": "ReserveDeploymentCooldownActive",`

### VERIFIED: `SECURITY.md:20`

Category: `overflow`

> - overflow redirection;

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `SECURITY.md:21`

Category: `reserve`

> - Reserve floor and cooldown enforcement;

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `SECURITY.md:33`

Category: `reserve`

> - Reserve and Spillway tests;

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### UNSUPPORTED: `SECURITY.md:113`

Category: `progressive_founder`

> 8. Founder progressive marginal schedule;

The implementation contains `current_tier` and period-cap state, but this audit did not find a progressive marginal-rate calculation.

Evidence:

- `programs/treasury-router/src/engines/company.rs:61` — `.period_cap`
- `programs/treasury-router/src/engines/company.rs:87` — `company.spent_current_period = company.period_cap;`
- `programs/treasury-router/src/engines/founder.rs:61` — `.period_cap`
- `programs/treasury-router/src/engines/founder.rs:87` — `founder.earned_current_period = founder.period_cap;`
- `programs/treasury-router/src/engines/integrity_firewall.rs:517` — `period_cap: 1_000,`
- `programs/treasury-router/src/engines/integrity_firewall.rs:522` — `current_tier: 0,`
- `programs/treasury-router/src/engines/integrity_firewall.rs:532` — `period_cap: 1_000,`
- `programs/treasury-router/src/events/mod.rs:86` — `pub founder_period_cap: u64,`
- `programs/treasury-router/src/events/mod.rs:112` — `pub company_period_cap: u64,`
- `programs/treasury-router/src/instructions/company.rs:166` — `company_period_cap: ctx.accounts.company_state.period_cap,`
- `programs/treasury-router/src/instructions/company.rs:223` — `ctx.accounts.company_state.period_cap`
- `programs/treasury-router/src/instructions/founder.rs:166` — `founder_period_cap: ctx.accounts.founder_state.period_cap,`

### VERIFIED: `SECURITY.md:114`

Category: `overflow`

> 9. Company and Founder overflow;

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `SECURITY.md:115`

Category: `reserve`

> 10. Reserve floor arithmetic;

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `SECURITY.md:116`

Category: `replay`

> 11. cooldown and replay protection;

Replay rejection is covered by program guards and adversarial tests.

Evidence:

- `programs/treasury-router/src/errors/mod.rs:27` — `NoUnprocessedFees,`
- `programs/treasury-router/src/errors/mod.rs:113` — `ReserveDeploymentCooldownActive,`
- `programs/treasury-router/src/instructions/process_fees.rs:132` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/process_fees.rs:401` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/spillway_release.rs:136` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `programs/treasury-router/src/state/reserve_policy.rs:110` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `tests/rbvr_protocol.ts:1559` — `it("RT-004 rejects fee-processing replay without mutating accounting", async () => {`
- `tests/rbvr_protocol.ts:1608` — `/NoUnprocessedFees|no unprocessed fees/i,`
- `tests/rbvr_protocol.ts:3380` — `it("RT-005D rejects an immediate Spillway replay without mutation", async () => {`
- `tests/rbvr_protocol.ts:3437` — `/ReserveDeploymentCooldownActive|cooldown is still active|cooldown/i,`
- `target/idl/treasury_router.json:2407` — `"name": "NoUnprocessedFees",`
- `target/idl/treasury_router.json:2552` — `"name": "ReserveDeploymentCooldownActive",`

### VERIFIED: `TESTING.md:30`

Category: `replay`

> - replay;

Replay rejection is covered by program guards and adversarial tests.

Evidence:

- `programs/treasury-router/src/errors/mod.rs:27` — `NoUnprocessedFees,`
- `programs/treasury-router/src/errors/mod.rs:113` — `ReserveDeploymentCooldownActive,`
- `programs/treasury-router/src/instructions/process_fees.rs:132` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/process_fees.rs:401` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/spillway_release.rs:136` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `programs/treasury-router/src/state/reserve_policy.rs:110` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `tests/rbvr_protocol.ts:1559` — `it("RT-004 rejects fee-processing replay without mutating accounting", async () => {`
- `tests/rbvr_protocol.ts:1608` — `/NoUnprocessedFees|no unprocessed fees/i,`
- `tests/rbvr_protocol.ts:3380` — `it("RT-005D rejects an immediate Spillway replay without mutation", async () => {`
- `tests/rbvr_protocol.ts:3437` — `/ReserveDeploymentCooldownActive|cooldown is still active|cooldown/i,`
- `target/idl/treasury_router.json:2407` — `"name": "NoUnprocessedFees",`
- `target/idl/treasury_router.json:2552` — `"name": "ReserveDeploymentCooldownActive",`

### VERIFIED: `TESTING.md:32`

Category: `reserve`

> - Spillway destination substitution;

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `TESTING.md:33`

Category: `reserve`

> - cooldown bypass.

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `TESTING.md:42`

Category: `overflow`

> - overflow redirection;

Company and Founder capped excess are implemented as Liquidity overflow.

Evidence:

- `programs/treasury-router/src/engines/company.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/company.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/company.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/company.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/engines/founder.rs:12` — `pub liquidity_overflow_amount: u64,`
- `programs/treasury-router/src/engines/founder.rs:40` — `liquidity_overflow_amount: requested,`
- `programs/treasury-router/src/engines/founder.rs:80` — `liquidity_overflow_amount: 0,`
- `programs/treasury-router/src/engines/founder.rs:100` — `liquidity_overflow_amount: overflow,`
- `programs/treasury-router/src/instructions/process_fees.rs:446` — `// Locked Bevernomics rule: all Company and Founder cap overflow is`
- `programs/treasury-router/src/instructions/process_fees.rs:449` — `.checked_add(founder_allocation.liquidity_overflow_amount)`
- `programs/treasury-router/src/instructions/process_fees.rs:450` — `.and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))`
- `programs/treasury-router/src/instructions/process_fees.rs:675` — `fn company_cap_overflow_redirected_to_liquidity_is_conserved() {`

### VERIFIED: `TESTING.md:43`

Category: `reserve`

> - Reserve floor;

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `TESTING.md:44`

Category: `reserve`

> - cooldown;

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `TESTING.md:46`

Category: `replay`

> - replay.

Replay rejection is covered by program guards and adversarial tests.

Evidence:

- `programs/treasury-router/src/errors/mod.rs:27` — `NoUnprocessedFees,`
- `programs/treasury-router/src/errors/mod.rs:113` — `ReserveDeploymentCooldownActive,`
- `programs/treasury-router/src/instructions/process_fees.rs:132` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/process_fees.rs:401` — `require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);`
- `programs/treasury-router/src/instructions/spillway_release.rs:136` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `programs/treasury-router/src/state/reserve_policy.rs:110` — `TreasuryRouterError::ReserveDeploymentCooldownActive`
- `tests/rbvr_protocol.ts:1559` — `it("RT-004 rejects fee-processing replay without mutating accounting", async () => {`
- `tests/rbvr_protocol.ts:1608` — `/NoUnprocessedFees|no unprocessed fees/i,`
- `tests/rbvr_protocol.ts:3380` — `it("RT-005D rejects an immediate Spillway replay without mutation", async () => {`
- `tests/rbvr_protocol.ts:3437` — `/ReserveDeploymentCooldownActive|cooldown is still active|cooldown/i,`
- `target/idl/treasury_router.json:2407` — `"name": "NoUnprocessedFees",`
- `target/idl/treasury_router.json:2552` — `"name": "ReserveDeploymentCooldownActive",`

### VERIFIED: `THREAT_MODEL.md:28`

Category: `reserve`

> Repeats a fee-processing, release, or Spillway transaction to obtain duplicate effects.

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `THREAT_MODEL.md:32`

Category: `permissionless`

> Invokes permissionless instructions with adversarial account ordering or values.

The Spillway source describes permissionless invocation and does not expose caller-selected policy or destinations.

Evidence:

- `programs/treasury-router/src/errors/mod.rs:124` — `#[msg("The Spillway destination is invalid.")]`
- `programs/treasury-router/src/errors/mod.rs:125` — `InvalidSpillwayDestination,`
- `programs/treasury-router/src/errors/mod.rs:127` — `#[msg("The Spillway would breach the protected Reserve floor.")]`
- `programs/treasury-router/src/events/mod.rs:118` — `/// Emitted after a successful permissionless Reserve Spillway release.`
- `programs/treasury-router/src/events/mod.rs:120` — `pub struct SpillwayReleaseExecuted {`
- `programs/treasury-router/src/instructions/buyback.rs:177` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/company.rs:99` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/founder.rs:99` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/liquidity.rs:177` — `/// Permissionless transaction caller and fee payer.`
- `programs/treasury-router/src/instructions/mod.rs:17` — `pub mod spillway_release;`
- `programs/treasury-router/src/instructions/mod.rs:33` — `pub use spillway_release::*;`
- `programs/treasury-router/src/instructions/reserve.rs:184` — `/// Permissionless transaction caller and fee payer.`

### VERIFIED: `THREAT_MODEL.md:44`

Category: `reserve`

> Attempts to exploit rounding, cap boundaries, period rollover, Reserve floors, or release timing.

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `THREAT_MODEL.md:54`

Category: `reserve`

> - Reserve Spillway;

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

### VERIFIED: `THREAT_MODEL.md:89`

Category: `reserve`

> Spillway deployment must preserve the minimum Reserve floor and enforce cooldown.

Reserve floor, deployment ratio, and cooldown are present in source.

Evidence:

- `programs/treasury-router/src/engines/reserve_deployment.rs:36` — `///     minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:41` — `/// 'surplus_deployment_bps' controls how much of the current surplus may be`
- `programs/treasury-router/src/engines/reserve_deployment.rs:45` — `pub minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:47` — `pub surplus_deployment_bps: u16,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:95` — `/// below 'minimum_reserve_floor'.`
- `programs/treasury-router/src/engines/reserve_deployment.rs:97` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:103` — `Ok(minimum_reserve_floor.max(liquidity_floor))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:118` — `if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {`
- `programs/treasury-router/src/engines/reserve_deployment.rs:123` — `policy.minimum_reserve_floor,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:140` — `.checked_mul(u128::from(policy.surplus_deployment_bps))`
- `programs/treasury-router/src/engines/reserve_deployment.rs:175` — `minimum_reserve_floor: u64,`
- `programs/treasury-router/src/engines/reserve_deployment.rs:177` — `surplus_deployment_bps: u16,`

## Required manual review

Automated text matching cannot prove every semantic or security claim. Review every `QUALIFY`, `INFERRED`, and `UNSUPPORTED` finding against the complete handler and account constraints.
