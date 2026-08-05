# RBVR Immutable Protocol Audit

Generated: **2026-07-30 20:02:05 UTC**

Git commit: `988930c6d6b17ebe791e2bf1ff3c793420dd185d`

## Executive summary

- Public entrypoints: **13**
- Initialized accounts detected: **7**
- `init_if_needed` accounts detected: **0**
- Mutable accounts detected: **31**
- Sensitive assignments detected: **44**
- Critical findings: **1**
- High-severity findings: **45**
- Missing immutability test categories: **1**

## Preliminary verdict

❌ **FAIL / MANUAL REVIEW REQUIRED:** One or more potentially critical reinitialization or public mutation paths were detected.

This is a static source audit. It identifies candidate mutation and reinitialization paths but does not replace transaction-level tests.

## Public entrypoints

| Entrypoint | Line | Initializer |
|---|---:|---:|
| `initialize` | 19 | Yes |
| `initialize_protocol_config` | 23 | Yes |
| `initialize_founder` | 27 | Yes |
| `initialize_company` | 36 | Yes |
| `initialize_treasury` | 45 | Yes |
| `initialize_execution_config` | 49 | Yes |
| `deposit_settlement` | 53 | No |
| `process_fees` | 57 | No |
| `authorize_reserve_execution` | 61 | No |
| `authorize_buyback_execution` | 65 | No |
| `authorize_liquidity_execution` | 69 | No |
| `authorize_founder_execution` | 73 | No |
| `authorize_company_execution` | 77 | No |

## Initialized account inventory

| File | Line | Accounts struct | Account | Type |
|---|---:|---|---|---|
| `programs/treasury-router/src/instructions/initialize.rs` | 9 | `Initialize` | `protocol_state` | `Account<'info` |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 17 | `InitializeCompany` | `company_state` | `Account<'info` |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 115 | `InitializeExecutionConfig` | `execution_config` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 17 | `InitializeFounder` | `founder_state` | `Account<'info` |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 21 | `InitializeProtocolConfig` | `protocol_config` | `Account<'info` |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 21 | `InitializeTreasury` | `treasury_state` | `Account<'info` |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 38 | `InitializeTreasury` | `settlement_vault` | `Account<'info` |

## `init_if_needed` inventory

| File | Line | Accounts struct | Account | Type |
|---|---:|---|---|---|
| — | — | — | — | None detected |

## PDA uniqueness inventory

| File | Line | Accounts struct | Account | Seeds | Payer |
|---|---:|---|---|---|---|
| `programs/treasury-router/src/instructions/initialize.rs` | 10 | `Initialize` | `protocol_state` | `PROTOCOL_SEED` | `authority` |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 19 | `InitializeCompany` | `company_state` | `COMPANY_STATE_SEED, protocol_state.key().as_ref()` | `authority` |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 117 | `InitializeExecutionConfig` | `execution_config` | `EXECUTION_CONFIG_SEED, protocol_state.key().as_ref()` | `authority` |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 19 | `InitializeFounder` | `founder_state` | `FOUNDER_STATE_SEED, protocol_state.key().as_ref()` | `authority` |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 23 | `InitializeProtocolConfig` | `protocol_config` | `PROTOCOL_CONFIG_SEED, protocol_state.key().as_ref()` | `authority` |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 23 | `InitializeTreasury` | `treasury_state` | `TREASURY_SEED, protocol_state.key().as_ref()` | `authority` |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 39 | `InitializeTreasury` | `settlement_vault` | `TREASURY_VAULT_SEED, treasury_state.key().as_ref()` | `authority` |

## Mutable account inventory

| File | Line | Accounts struct | Account | Type |
|---|---:|---|---|---|
| `programs/treasury-router/src/instructions/buyback.rs` | 72 | `AuthorizeBuybackExecution` | `treasury` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/buyback.rs` | 107 | `AuthorizeBuybackExecution` | `settlement_vault` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/buyback.rs` | 120 | `AuthorizeBuybackExecution` | `buyback_destination` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/company.rs` | 25 | `AuthorizeCompanyExecution` | `treasury` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/company.rs` | 71 | `AuthorizeCompanyExecution` | `settlement_vault` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/company.rs` | 84 | `AuthorizeCompanyExecution` | `company_destination` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 42 | `DepositSettlement` | `source_token_account` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 51 | `DepositSettlement` | `settlement_vault` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/founder.rs` | 25 | `AuthorizeFounderExecution` | `treasury` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/founder.rs` | 71 | `AuthorizeFounderExecution` | `settlement_vault` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/founder.rs` | 84 | `AuthorizeFounderExecution` | `founder_destination` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/initialize.rs` | 17 | `Initialize` | `authority` | `Signer<'info>` |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 10 | `InitializeCompany` | `protocol_state` | `Account<'info` |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 29 | `InitializeCompany` | `authority` | `Signer<'info>` |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 127 | `InitializeExecutionConfig` | `authority` | `Signer<'info>` |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 10 | `InitializeFounder` | `protocol_state` | `Account<'info` |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 29 | `InitializeFounder` | `authority` | `Signer<'info>` |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 14 | `InitializeProtocolConfig` | `protocol_state` | `Account<'info` |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 33 | `InitializeProtocolConfig` | `authority` | `Signer<'info>` |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 14 | `InitializeTreasury` | `protocol_state` | `Account<'info` |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 50 | `InitializeTreasury` | `authority` | `Signer<'info>` |
| `programs/treasury-router/src/instructions/liquidity.rs` | 72 | `AuthorizeLiquidityExecution` | `treasury` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/liquidity.rs` | 107 | `AuthorizeLiquidityExecution` | `settlement_vault` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/liquidity.rs` | 120 | `AuthorizeLiquidityExecution` | `liquidity_destination` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/process_fees.rs` | 17 | `ProcessFees` | `protocol_state` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/process_fees.rs` | 37 | `ProcessFees` | `treasury` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/process_fees.rs` | 48 | `ProcessFees` | `founder_state` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/process_fees.rs` | 59 | `ProcessFees` | `company_state` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/reserve.rs` | 72 | `AuthorizeReserveExecution` | `treasury` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/reserve.rs` | 107 | `AuthorizeReserveExecution` | `settlement_vault` | `Box<Account<'info` |
| `programs/treasury-router/src/instructions/reserve.rs` | 120 | `AuthorizeReserveExecution` | `reserve_destination` | `Box<Account<'info` |

## Sensitive field assignments

| File | Line | Field | Test-only heuristic |
|---|---:|---|---:|
| `programs/treasury-router/src/engines/execution_guard.rs` | 442 | `buybacks_paused` | No |
| `programs/treasury-router/src/engines/execution_guard.rs` | 455 | `buybacks_paused` | No |
| `programs/treasury-router/src/engines/execution_guard.rs` | 418 | `paused` | No |
| `programs/treasury-router/src/engines/integrity_firewall.rs` | 441 | `buyback_destination` | No |
| `programs/treasury-router/src/engines/integrity_firewall.rs` | 443 | `company_destination` | No |
| `programs/treasury-router/src/engines/integrity_firewall.rs` | 444 | `founder_destination` | No |
| `programs/treasury-router/src/engines/integrity_firewall.rs` | 442 | `liquidity_destination` | No |
| `programs/treasury-router/src/engines/integrity_firewall.rs` | 440 | `reserve_destination` | No |
| `programs/treasury-router/src/engines/integrity_firewall.rs` | 328 | `settlement_mint` | No |
| `programs/treasury-router/src/engines/integrity_firewall.rs` | 436 | `settlement_mint` | No |
| `programs/treasury-router/src/engines/sentinel.rs` | 560 | `founder_bps` | No |
| `programs/treasury-router/src/engines/sentinel.rs` | 559 | `reserve_bps` | No |
| `programs/treasury-router/src/engines/sentinel.rs` | 609 | `settlement_mint` | No |
| `programs/treasury-router/src/engines/sentinel.rs` | 610 | `settlement_vault` | No |
| `programs/treasury-router/src/instructions/initialize.rs` | 30 | `authority` | No |
| `programs/treasury-router/src/instructions/initialize.rs` | 42 | `paused` | No |
| `programs/treasury-router/src/instructions/initialize.rs` | 29 | `version` | No |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 76 | `period_duration` | No |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 69 | `version` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 139 | `buyback_destination` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 229 | `buyback_destination` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 141 | `company_destination` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 231 | `company_destination` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 142 | `founder_destination` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 232 | `founder_destination` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 140 | `liquidity_destination` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 230 | `liquidity_destination` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 138 | `reserve_destination` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 228 | `reserve_destination` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 226 | `settlement_mint` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 136 | `settlement_vault` | No |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 234 | `version` | No |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 76 | `period_duration` | No |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 69 | `version` | No |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 74 | `company_bps` | No |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 75 | `founder_bps` | No |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 73 | `liquidity_bps` | No |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 71 | `reserve_bps` | No |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 68 | `version` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 48 | `authority` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 101 | `buybacks_paused` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 74 | `settlement_mint` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 75 | `settlement_vault` | No |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 72 | `version` | No |

## Immutability test coverage

| Required test category | Detected |
|---|---:|
| initialize twice | No |
| protocol config reinitialization | Yes |
| treasury reinitialization | Yes |
| execution config reinitialization | Yes |
| founder reinitialization | Yes |
| company reinitialization | Yes |
| destination immutability | Yes |
| allocation immutability | Yes |

## Findings requiring review

### 1. CRITICAL: Public mutation entrypoint

**Location:** `programs/treasury-router/src/lib.rs:53`

Public entrypoint `deposit_settlement` appears capable of changing configuration, authority, or protocol state.

```text
    49:     pub fn initialize_execution_config(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
    50:         instructions::initialize_execution_config::handler(ctx)
    51:     }
    52: 
>   53:     pub fn deposit_settlement(ctx: Context<DepositSettlement>, amount: u64) -> Result<()> {
    54:         instructions::deposit_settlement::handler(ctx, amount)
    55:     }
    56: 
    57:     pub fn process_fees(ctx: Context<ProcessFees>) -> Result<()> {
```

### 2. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/engines/execution_guard.rs:418`

Production code assigns sensitive field `paused`.

```text
   414:     fn rejects_release_when_protocol_is_paused() {
   415:         let mut protocol = valid_protocol();
   416:         let treasury = valid_treasury();
   417: 
>  418:         protocol.paused = true;
   419: 
   420:         assert_anchor_error(
   421:             authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
   422:             "ProtocolPaused",
```

### 3. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/engines/execution_guard.rs:442`

Production code assigns sensitive field `buybacks_paused`.

```text
   438:     fn rejects_buyback_when_buybacks_are_paused() {
   439:         let protocol = valid_protocol();
   440:         let mut treasury = valid_treasury();
   441: 
>  442:         treasury.buybacks_paused = true;
   443: 
   444:         assert_anchor_error(
   445:             authorize_release(&protocol, &treasury, ReleaseBucket::BuybackBurn, 1),
   446:             "BuybacksPaused",
```

### 4. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/engines/execution_guard.rs:455`

Production code assigns sensitive field `buybacks_paused`.

```text
   451:     fn buyback_pause_does_not_block_other_buckets() {
   452:         let protocol = valid_protocol();
   453:         let mut treasury = valid_treasury();
   454: 
>  455:         treasury.buybacks_paused = true;
   456: 
   457:         let authorization =
   458:             authorize_release(&protocol, &treasury, ReleaseBucket::Liquidity, 50).unwrap();
   459: 
```

### 5. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:328`

Production code assigns sensitive field `settlement_mint`.

```text
   324: 
   325:     /*
   326:      * All destination accounts must use the settlement mint.
   327:      */
>  328:     let settlement_mint = treasury.settlement_mint;
   329: 
   330:     if destinations.reserve.mint != settlement_mint
   331:         || destinations.buyback.mint != settlement_mint
   332:         || destinations.liquidity.mint != settlement_mint
```

### 6. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:436`

Production code assigns sensitive field `settlement_mint`.

```text
   432:             &[EXECUTION_CONFIG_SEED, protocol_key.as_ref()],
   433:             &program_id,
   434:         );
   435: 
>  436:         let settlement_mint = Pubkey::new_unique();
   437:         let founder_recipient = Pubkey::new_unique();
   438:         let company_recipient = Pubkey::new_unique();
   439: 
   440:         let reserve_destination = Pubkey::new_unique();
```

### 7. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:440`

Production code assigns sensitive field `reserve_destination`.

```text
   436:         let settlement_mint = Pubkey::new_unique();
   437:         let founder_recipient = Pubkey::new_unique();
   438:         let company_recipient = Pubkey::new_unique();
   439: 
>  440:         let reserve_destination = Pubkey::new_unique();
   441:         let buyback_destination = Pubkey::new_unique();
   442:         let liquidity_destination = Pubkey::new_unique();
   443:         let company_destination = Pubkey::new_unique();
   444:         let founder_destination = Pubkey::new_unique();
```

### 8. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:441`

Production code assigns sensitive field `buyback_destination`.

```text
   437:         let founder_recipient = Pubkey::new_unique();
   438:         let company_recipient = Pubkey::new_unique();
   439: 
   440:         let reserve_destination = Pubkey::new_unique();
>  441:         let buyback_destination = Pubkey::new_unique();
   442:         let liquidity_destination = Pubkey::new_unique();
   443:         let company_destination = Pubkey::new_unique();
   444:         let founder_destination = Pubkey::new_unique();
   445: 
```

### 9. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:442`

Production code assigns sensitive field `liquidity_destination`.

```text
   438:         let company_recipient = Pubkey::new_unique();
   439: 
   440:         let reserve_destination = Pubkey::new_unique();
   441:         let buyback_destination = Pubkey::new_unique();
>  442:         let liquidity_destination = Pubkey::new_unique();
   443:         let company_destination = Pubkey::new_unique();
   444:         let founder_destination = Pubkey::new_unique();
   445: 
   446:         let protocol = ProtocolState {
```

### 10. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:443`

Production code assigns sensitive field `company_destination`.

```text
   439: 
   440:         let reserve_destination = Pubkey::new_unique();
   441:         let buyback_destination = Pubkey::new_unique();
   442:         let liquidity_destination = Pubkey::new_unique();
>  443:         let company_destination = Pubkey::new_unique();
   444:         let founder_destination = Pubkey::new_unique();
   445: 
   446:         let protocol = ProtocolState {
   447:             version: PROTOCOL_VERSION,
```

### 11. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/engines/integrity_firewall.rs:444`

Production code assigns sensitive field `founder_destination`.

```text
   440:         let reserve_destination = Pubkey::new_unique();
   441:         let buyback_destination = Pubkey::new_unique();
   442:         let liquidity_destination = Pubkey::new_unique();
   443:         let company_destination = Pubkey::new_unique();
>  444:         let founder_destination = Pubkey::new_unique();
   445: 
   446:         let protocol = ProtocolState {
   447:             version: PROTOCOL_VERSION,
   448:             authority: Pubkey::new_unique(),
```

### 12. HIGH: Economic mutation

**Location:** `programs/treasury-router/src/engines/sentinel.rs:559`

Production code assigns sensitive field `reserve_bps`.

```text
   555:         let mut config = valid_protocol_config(protocol);
   556:         let treasury = valid_treasury(protocol);
   557: 
   558:         // Still totals 10,000 bps, but silently changes Beavernomics.
>  559:         config.reserve_bps = 2_900;
   560:         config.founder_bps = 1_100;
   561: 
   562:         let report = evaluate_linkage(protocol, &config, &treasury);
   563: 
```

### 13. HIGH: Economic mutation

**Location:** `programs/treasury-router/src/engines/sentinel.rs:560`

Production code assigns sensitive field `founder_bps`.

```text
   556:         let treasury = valid_treasury(protocol);
   557: 
   558:         // Still totals 10,000 bps, but silently changes Beavernomics.
   559:         config.reserve_bps = 2_900;
>  560:         config.founder_bps = 1_100;
   561: 
   562:         let report = evaluate_linkage(protocol, &config, &treasury);
   563: 
   564:         assert!(!report.healthy);
```

### 14. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/engines/sentinel.rs:609`

Production code assigns sensitive field `settlement_mint`.

```text
   605:         let protocol = Pubkey::new_unique();
   606:         let config = valid_protocol_config(protocol);
   607:         let mut treasury = valid_treasury(protocol);
   608: 
>  609:         treasury.settlement_mint = Pubkey::default();
   610:         treasury.settlement_vault = Pubkey::default();
   611: 
   612:         let report = evaluate_linkage(protocol, &config, &treasury);
   613: 
```

### 15. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/engines/sentinel.rs:610`

Production code assigns sensitive field `settlement_vault`.

```text
   606:         let config = valid_protocol_config(protocol);
   607:         let mut treasury = valid_treasury(protocol);
   608: 
   609:         treasury.settlement_mint = Pubkey::default();
>  610:         treasury.settlement_vault = Pubkey::default();
   611: 
   612:         let report = evaluate_linkage(protocol, &config, &treasury);
   613: 
   614:         assert!(!report.settlement_mint_present);
```

### 16. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/instructions/initialize.rs:29`

Production code assigns sensitive field `version`.

```text
    25: pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    26:     let protocol_state = &mut ctx.accounts.protocol_state;
    27:     let clock = Clock::get()?;
    28: 
>   29:     protocol_state.version = PROTOCOL_VERSION;
    30:     protocol_state.authority = ctx.accounts.authority.key();
    31: 
    32:     protocol_state.protocol_config = Pubkey::default();
    33:     protocol_state.treasury_state = Pubkey::default();
```

### 17. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/instructions/initialize.rs:30`

Production code assigns sensitive field `authority`.

```text
    26:     let protocol_state = &mut ctx.accounts.protocol_state;
    27:     let clock = Clock::get()?;
    28: 
    29:     protocol_state.version = PROTOCOL_VERSION;
>   30:     protocol_state.authority = ctx.accounts.authority.key();
    31: 
    32:     protocol_state.protocol_config = Pubkey::default();
    33:     protocol_state.treasury_state = Pubkey::default();
    34:     protocol_state.reserve_state = Pubkey::default();
```

### 18. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/instructions/initialize.rs:42`

Production code assigns sensitive field `paused`.

```text
    38:     protocol_state.buyback_state = Pubkey::default();
    39: 
    40:     protocol_state.beaver_score = 0;
    41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
>   42:     protocol_state.paused = false;
    43:     protocol_state.bump = ctx.bumps.protocol_state;
    44:     protocol_state.initialized_at = clock.unix_timestamp;
    45:     protocol_state.reserved = [0; 64];
    46: 
```

### 19. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/instructions/initialize_company.rs:69`

Production code assigns sensitive field `version`.

```text
    65:     let clock = Clock::get()?;
    66:     let protocol_key = ctx.accounts.protocol_state.key();
    67:     let company_state = &mut ctx.accounts.company_state;
    68: 
>   69:     company_state.version = COMPANY_STATE_VERSION;
    70:     company_state.protocol = protocol_key;
    71:     company_state.recipient = recipient;
    72:     company_state.period_cap = period_cap;
    73:     company_state.spent_current_period = 0;
```

### 20. HIGH: Economic mutation

**Location:** `programs/treasury-router/src/instructions/initialize_company.rs:76`

Production code assigns sensitive field `period_duration`.

```text
    72:     company_state.period_cap = period_cap;
    73:     company_state.spent_current_period = 0;
    74:     company_state.lifetime_spent = 0;
    75:     company_state.period_started_at = clock.unix_timestamp;
>   76:     company_state.period_duration = period_duration;
    77:     company_state.enabled = true;
    78:     company_state.bump = ctx.bumps.company_state;
    79:     company_state.reserved = [0; 64];
    80: 
```

### 21. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:136`

Production code assigns sensitive field `settlement_vault`.

```text
   132:     pub system_program: Program<'info, System>,
   133: }
   134: 
   135: pub fn handler(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
>  136:     let settlement_vault = ctx.accounts.settlement_vault.key();
   137: 
   138:     let reserve_destination = ctx.accounts.reserve_destination.key();
   139:     let buyback_destination = ctx.accounts.buyback_destination.key();
   140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
```

### 22. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:138`

Production code assigns sensitive field `reserve_destination`.

```text
   134: 
   135: pub fn handler(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
   136:     let settlement_vault = ctx.accounts.settlement_vault.key();
   137: 
>  138:     let reserve_destination = ctx.accounts.reserve_destination.key();
   139:     let buyback_destination = ctx.accounts.buyback_destination.key();
   140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
   141:     let company_destination = ctx.accounts.company_destination.key();
   142:     let founder_destination = ctx.accounts.founder_destination.key();
```

### 23. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:139`

Production code assigns sensitive field `buyback_destination`.

```text
   135: pub fn handler(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
   136:     let settlement_vault = ctx.accounts.settlement_vault.key();
   137: 
   138:     let reserve_destination = ctx.accounts.reserve_destination.key();
>  139:     let buyback_destination = ctx.accounts.buyback_destination.key();
   140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
   141:     let company_destination = ctx.accounts.company_destination.key();
   142:     let founder_destination = ctx.accounts.founder_destination.key();
   143: 
```

### 24. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:140`

Production code assigns sensitive field `liquidity_destination`.

```text
   136:     let settlement_vault = ctx.accounts.settlement_vault.key();
   137: 
   138:     let reserve_destination = ctx.accounts.reserve_destination.key();
   139:     let buyback_destination = ctx.accounts.buyback_destination.key();
>  140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
   141:     let company_destination = ctx.accounts.company_destination.key();
   142:     let founder_destination = ctx.accounts.founder_destination.key();
   143: 
   144:     // No allocation destination may point back to the treasury vault.
```

### 25. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:141`

Production code assigns sensitive field `company_destination`.

```text
   137: 
   138:     let reserve_destination = ctx.accounts.reserve_destination.key();
   139:     let buyback_destination = ctx.accounts.buyback_destination.key();
   140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
>  141:     let company_destination = ctx.accounts.company_destination.key();
   142:     let founder_destination = ctx.accounts.founder_destination.key();
   143: 
   144:     // No allocation destination may point back to the treasury vault.
   145:     require_keys_neq!(
```

### 26. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:142`

Production code assigns sensitive field `founder_destination`.

```text
   138:     let reserve_destination = ctx.accounts.reserve_destination.key();
   139:     let buyback_destination = ctx.accounts.buyback_destination.key();
   140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
   141:     let company_destination = ctx.accounts.company_destination.key();
>  142:     let founder_destination = ctx.accounts.founder_destination.key();
   143: 
   144:     // No allocation destination may point back to the treasury vault.
   145:     require_keys_neq!(
   146:         reserve_destination,
```

### 27. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:226`

Production code assigns sensitive field `settlement_mint`.

```text
   222: 
   223:     let execution_config = &mut ctx.accounts.execution_config;
   224: 
   225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
>  226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
```

### 28. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:228`

Production code assigns sensitive field `reserve_destination`.

```text
   224: 
   225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
>  228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
```

### 29. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:229`

Production code assigns sensitive field `buyback_destination`.

```text
   225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
>  229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
```

### 30. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:230`

Production code assigns sensitive field `liquidity_destination`.

```text
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
>  230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
```

### 31. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:231`

Production code assigns sensitive field `company_destination`.

```text
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
>  231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
   235:     execution_config.bump = ctx.bumps.execution_config;
```

### 32. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:232`

Production code assigns sensitive field `founder_destination`.

```text
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
>  232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
   235:     execution_config.bump = ctx.bumps.execution_config;
   236: 
```

### 33. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/instructions/initialize_execution_config.rs:234`

Production code assigns sensitive field `version`.

```text
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
>  234:     execution_config.version = EXECUTION_CONFIG_VERSION;
   235:     execution_config.bump = ctx.bumps.execution_config;
   236: 
   237:     msg!(
   238:         "Immutable ExecutionConfig initialized: {}",
```

### 34. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/instructions/initialize_founder.rs:69`

Production code assigns sensitive field `version`.

```text
    65:     let clock = Clock::get()?;
    66:     let protocol_key = ctx.accounts.protocol_state.key();
    67:     let founder_state = &mut ctx.accounts.founder_state;
    68: 
>   69:     founder_state.version = FOUNDER_STATE_VERSION;
    70:     founder_state.protocol = protocol_key;
    71:     founder_state.recipient = recipient;
    72:     founder_state.period_cap = period_cap;
    73:     founder_state.earned_current_period = 0;
```

### 35. HIGH: Economic mutation

**Location:** `programs/treasury-router/src/instructions/initialize_founder.rs:76`

Production code assigns sensitive field `period_duration`.

```text
    72:     founder_state.period_cap = period_cap;
    73:     founder_state.earned_current_period = 0;
    74:     founder_state.lifetime_earned = 0;
    75:     founder_state.period_started_at = clock.unix_timestamp;
>   76:     founder_state.period_duration = period_duration;
    77:     founder_state.current_tier = 0;
    78:     founder_state.enabled = true;
    79:     founder_state.bump = ctx.bumps.founder_state;
    80:     founder_state.reserved = [0; 64];
```

### 36. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/instructions/initialize_protocol_config.rs:68`

Production code assigns sensitive field `version`.

```text
    64:     let clock = Clock::get()?;
    65:     let protocol_key = ctx.accounts.protocol_state.key();
    66:     let protocol_config = &mut ctx.accounts.protocol_config;
    67: 
>   68:     protocol_config.version = PROTOCOL_CONFIG_VERSION;
    69:     protocol_config.protocol = protocol_key;
    70: 
    71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
```

### 37. HIGH: Economic mutation

**Location:** `programs/treasury-router/src/instructions/initialize_protocol_config.rs:71`

Production code assigns sensitive field `reserve_bps`.

```text
    67: 
    68:     protocol_config.version = PROTOCOL_CONFIG_VERSION;
    69:     protocol_config.protocol = protocol_key;
    70: 
>   71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
```

### 38. HIGH: Economic mutation

**Location:** `programs/treasury-router/src/instructions/initialize_protocol_config.rs:73`

Production code assigns sensitive field `liquidity_bps`.

```text
    69:     protocol_config.protocol = protocol_key;
    70: 
    71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
>   73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
    77:     protocol_config.updates_enabled = false;
```

### 39. HIGH: Economic mutation

**Location:** `programs/treasury-router/src/instructions/initialize_protocol_config.rs:74`

Production code assigns sensitive field `company_bps`.

```text
    70: 
    71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
>   74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
    77:     protocol_config.updates_enabled = false;
    78:     protocol_config.bump = ctx.bumps.protocol_config;
```

### 40. HIGH: Economic mutation

**Location:** `programs/treasury-router/src/instructions/initialize_protocol_config.rs:75`

Production code assigns sensitive field `founder_bps`.

```text
    71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
>   75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
    77:     protocol_config.updates_enabled = false;
    78:     protocol_config.bump = ctx.bumps.protocol_config;
    79:     protocol_config.updated_at = clock.unix_timestamp;
```

### 41. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/instructions/initialize_treasury.rs:48`

Production code assigns sensitive field `authority`.

```text
    44:             treasury_state.key().as_ref()
    45:         ],
    46:         bump,
    47:         token::mint = settlement_mint,
>   48:         token::authority = treasury_state
    49:     )]
    50:     pub settlement_vault: Account<'info, TokenAccount>,
    51: 
    52:     #[account(mut)]
```

### 42. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/instructions/initialize_treasury.rs:72`

Production code assigns sensitive field `version`.

```text
    68:     );
    69: 
    70:     let treasury_state = &mut ctx.accounts.treasury_state;
    71: 
>   72:     treasury_state.version = TREASURY_VERSION;
    73:     treasury_state.protocol = ctx.accounts.protocol_state.key();
    74:     treasury_state.settlement_mint = ctx.accounts.settlement_mint.key();
    75:     treasury_state.settlement_vault = ctx.accounts.settlement_vault.key();
    76: 
```

### 43. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_treasury.rs:74`

Production code assigns sensitive field `settlement_mint`.

```text
    70:     let treasury_state = &mut ctx.accounts.treasury_state;
    71: 
    72:     treasury_state.version = TREASURY_VERSION;
    73:     treasury_state.protocol = ctx.accounts.protocol_state.key();
>   74:     treasury_state.settlement_mint = ctx.accounts.settlement_mint.key();
    75:     treasury_state.settlement_vault = ctx.accounts.settlement_vault.key();
    76: 
    77:     treasury_state.total_fees_received = 0;
    78:     treasury_state.total_fees_allocated = 0;
```

### 44. HIGH: Destination mutation

**Location:** `programs/treasury-router/src/instructions/initialize_treasury.rs:75`

Production code assigns sensitive field `settlement_vault`.

```text
    71: 
    72:     treasury_state.version = TREASURY_VERSION;
    73:     treasury_state.protocol = ctx.accounts.protocol_state.key();
    74:     treasury_state.settlement_mint = ctx.accounts.settlement_mint.key();
>   75:     treasury_state.settlement_vault = ctx.accounts.settlement_vault.key();
    76: 
    77:     treasury_state.total_fees_received = 0;
    78:     treasury_state.total_fees_allocated = 0;
    79: 
```

### 45. HIGH: Security-state mutation

**Location:** `programs/treasury-router/src/instructions/initialize_treasury.rs:101`

Production code assigns sensitive field `buybacks_paused`.

```text
    97: 
    98:     treasury_state.last_processed_at = 0;
    99:     treasury_state.processing_epoch = 0;
   100:     treasury_state.waterfall_stage = INITIAL_WATERFALL_STAGE;
>  101:     treasury_state.buybacks_paused = false;
   102:     treasury_state.bump = ctx.bumps.treasury_state;
   103:     treasury_state.reserved = [0; 24];
   104: 
   105:     require!(
```

### 46. HIGH: Mutation function

**Location:** `programs/treasury-router/src/lib.rs:53`

Function `deposit_settlement` may expose a mutation path.

```text
    49:     pub fn initialize_execution_config(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
    50:         instructions::initialize_execution_config::handler(ctx)
    51:     }
    52: 
>   53:     pub fn deposit_settlement(ctx: Context<DepositSettlement>, amount: u64) -> Result<()> {
    54:         instructions::deposit_settlement::handler(ctx, amount)
    55:     }
    56: 
    57:     pub fn process_fees(ctx: Context<ProcessFees>) -> Result<()> {
```

## Build verification

```text
Checking treasury-router v0.1.0 (/home/alec_elliott/rocket-beaver-contract/programs/treasury-router)
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
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.69s
```

## Current working tree

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
?? .rbvr-security-audit/
?? RBVR-AUTHORITY-LOCKDOWN-AUDIT.md
?? RBVR-AUTHORITY-MAP.md
?? RBVR-SECURITY-AUDIT-LATEST.md
?? fuzz/fuzz_targets/process_release.rs
?? programs/treasury-router/src/engines/release.rs
?? rbvr-authority-lockdown-audit.py
?? rbvr-authority-pass.sh
?? rbvr-autonomous-release-pass-v2.sh
?? rbvr-autonomous-release-pass-v3.sh
?? rbvr-autonomous-release-pass.sh
?? rbvr-deep-security-audit.sh
?? rbvr-immutability-audit.py
?? rbvr-patch-autonomous-tests.py
?? rbvr-permissionless-release-pass-v2.sh
?? rbvr-permissionless-release-pass-v3.sh
?? rbvr-permissionless-release-pass.sh
?? rbvr-release-authority-pass.sh
?? rbvr-release-fuzz-source.txt
?? rbvr-remove-release-authority.py
?? rbvr-remove-release-event-authority-v2.py
?? rbvr-remove-release-event-authority.py
?? rbvr-remove-release-test-authority.py
```

## Required next tests

- Add explicit test: **initialize twice**

## Manual review questions

1. Does every protocol account use one canonical PDA?
2. Can any parallel protocol/config/treasury account be created?
3. Can initialization instructions succeed after their canonical PDA exists?
4. Are destination, cap, period, mint, vault, and allocation fields assigned only once?
5. Is there any public update, pause, authority-rotation, or destination-change instruction?
6. Does every mutable initialization account change only a one-time completion flag?
7. Can any close-account path reopen an initialization surface?
