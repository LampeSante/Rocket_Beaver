# RBVR Authority Lockdown Audit

Generated: **2026-07-30 20:52:26 UTC**

## Executive summary

- Public Rust entrypoints found: **13**
- Signer accounts found: **7**
- Mutable program accounts found: **4**
- Signers found in release handlers: **0**
- Unclassified signers requiring review: **0**

This report distinguishes initialization authority, depositor authority, release authority, mutable program accounts, and program upgrade authority.

## Release permissionlessness verification

| Handler | Signer present | Authority access | Autonomous guard | Result |
|---|---:|---:|---:|---|
| `buyback.rs` | No | No | Yes | PASS |
| `company.rs` | No | No | Yes | PASS |
| `founder.rs` | No | No | Yes | PASS |
| `liquidity.rs` | No | No | Yes | PASS |
| `reserve.rs` | No | No | Yes | PASS |

## Remaining signer inventory

| File | Line | Accounts struct | Signer | Classification | Reason |
|---|---:|---|---|---|---|
| `programs/treasury-router/src/instructions/deposit_settlement.rs` | 64 | `DepositSettlement` | `authority` | **Operational depositor** | May be required to authorize movement from the depositor's token account. |
| `programs/treasury-router/src/instructions/initialize.rs` | 20 | `Initialize` | `authority` | **Initialization-only** | Expected only before the protocol is permanently locked. |
| `programs/treasury-router/src/instructions/initialize_company.rs` | 32 | `InitializeCompany` | `authority` | **Initialization-only** | Expected only before the protocol is permanently locked. |
| `programs/treasury-router/src/instructions/initialize_execution_config.rs` | 130 | `InitializeExecutionConfig` | `authority` | **Initialization-only** | Expected only before the protocol is permanently locked. |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 32 | `InitializeFounder` | `authority` | **Initialization-only** | Expected only before the protocol is permanently locked. |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 36 | `InitializeProtocolConfig` | `authority` | **Initialization-only** | Expected only before the protocol is permanently locked. |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 53 | `InitializeTreasury` | `authority` | **Initialization-only** | Expected only before the protocol is permanently locked. |

## Mutable account inventory

| File | Line | Accounts struct | Account | Type |
|---|---:|---|---|---|
| `programs/treasury-router/src/instructions/initialize_company.rs` | 17 | `InitializeCompany` | `protocol_state` | `ProtocolState` |
| `programs/treasury-router/src/instructions/initialize_founder.rs` | 17 | `InitializeFounder` | `protocol_state` | `ProtocolState` |
| `programs/treasury-router/src/instructions/initialize_protocol_config.rs` | 21 | `InitializeProtocolConfig` | `protocol_state` | `ProtocolState` |
| `programs/treasury-router/src/instructions/initialize_treasury.rs` | 21 | `InitializeTreasury` | `protocol_state` | `ProtocolState` |

## Public instruction entrypoints

- `initialize`
- `initialize_protocol_config`
- `initialize_founder`
- `initialize_company`
- `initialize_treasury`
- `initialize_execution_config`
- `deposit_settlement`
- `process_fees`
- `authorize_reserve_execution`
- `authorize_buyback_execution`
- `authorize_liquidity_execution`
- `authorize_founder_execution`
- `authorize_company_execution`

## Upgrade authority status

### Solana configuration

```text
Config File: /home/alec_elliott/.config/solana/cli/config.yml
RPC URL: http://127.0.0.1:8899 
WebSocket URL: ws://127.0.0.1:8900/ (computed)
Keypair Path: /home/alec_elliott/rocket-beaver-dev/keys/local-deployment.json 
Commitment: confirmed
```

### Program information

```text
Error: AccountNotFound: pubkey=5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3: error sending request for url (http://127.0.0.1:8899/)
```

## Authority-related source findings

### 1. Signer constraint

File: `programs/treasury-router/src/engines/execution_guard.rs:37`

```text
    32:     pub reserve_ratio_bps: u16,
    33: }
    34: 
    35: /// Authorizes a treasury release through the mandatory RBVR security layer.
    36: ///
>   37: /// Account ownership, PDA seeds, signer authority, mint validation, and
    38: /// protocol-to-treasury linkage must be enforced by the Anchor instruction
    39: /// account constraints that call this function.
    40: pub fn authorize_release(
    41:     protocol: &ProtocolState,
    42:     treasury: &TreasuryState,
```

### 2. Pause mutation

File: `programs/treasury-router/src/engines/execution_guard.rs:418`

```text
   413:     #[test]
   414:     fn rejects_release_when_protocol_is_paused() {
   415:         let mut protocol = valid_protocol();
   416:         let treasury = valid_treasury();
   417: 
>  418:         protocol.paused = true;
   419: 
   420:         assert_anchor_error(
   421:             authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
   422:             "ProtocolPaused",
   423:         );
```

### 3. Pause mutation

File: `programs/treasury-router/src/engines/execution_guard.rs:442`

```text
   437:     #[test]
   438:     fn rejects_buyback_when_buybacks_are_paused() {
   439:         let protocol = valid_protocol();
   440:         let mut treasury = valid_treasury();
   441: 
>  442:         treasury.buybacks_paused = true;
   443: 
   444:         assert_anchor_error(
   445:             authorize_release(&protocol, &treasury, ReleaseBucket::BuybackBurn, 1),
   446:             "BuybacksPaused",
   447:         );
```

### 4. Pause mutation

File: `programs/treasury-router/src/engines/execution_guard.rs:455`

```text
   450:     #[test]
   451:     fn buyback_pause_does_not_block_other_buckets() {
   452:         let protocol = valid_protocol();
   453:         let mut treasury = valid_treasury();
   454: 
>  455:         treasury.buybacks_paused = true;
   456: 
   457:         let authorization =
   458:             authorize_release(&protocol, &treasury, ReleaseBucket::Liquidity, 50).unwrap();
   459: 
   460:         assert_eq!(authorization.bucket, ReleaseBucket::Liquidity);
```

### 5. Signer constraint

File: `programs/treasury-router/src/engines/integrity_firewall.rs:74`

```text
    69: }
    70: 
    71: /// Read-only SPL token-account facts supplied to the firewall.
    72: ///
    73: /// This is intentionally plain data. The firewall does not receive mutable
>   74: /// accounts, signer authority, or CPI capability.
    75: #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    76: pub struct TokenAccountFacts {
    77:     pub key: Pubkey,
    78:     pub mint: Pubkey,
    79:     pub owner: Pubkey,
```

### 6. Authority comparison

File: `programs/treasury-router/src/engines/integrity_firewall.rs:167`

```text
   162: 
   163:     if protocol.version != PROTOCOL_VERSION {
   164:         fail(IntegrityInvariant::ProtocolVersion);
   165:     }
   166: 
>  167:     if protocol.authority == Pubkey::default() {
   168:         fail(IntegrityInvariant::ProtocolAuthority);
   169:     }
   170: 
   171:     /*
   172:      * ProtocolConfig
```

### 7. Authority assignment

File: `programs/treasury-router/src/engines/integrity_firewall.rs:167`

```text
   162: 
   163:     if protocol.version != PROTOCOL_VERSION {
   164:         fail(IntegrityInvariant::ProtocolVersion);
   165:     }
   166: 
>  167:     if protocol.authority == Pubkey::default() {
   168:         fail(IntegrityInvariant::ProtocolAuthority);
   169:     }
   170: 
   171:     /*
   172:      * ProtocolConfig
```

### 8. Destination mutation

File: `programs/treasury-router/src/engines/integrity_firewall.rs:440`

```text
   435: 
   436:         let settlement_mint = Pubkey::new_unique();
   437:         let founder_recipient = Pubkey::new_unique();
   438:         let company_recipient = Pubkey::new_unique();
   439: 
>  440:         let reserve_destination = Pubkey::new_unique();
   441:         let buyback_destination = Pubkey::new_unique();
   442:         let liquidity_destination = Pubkey::new_unique();
   443:         let company_destination = Pubkey::new_unique();
   444:         let founder_destination = Pubkey::new_unique();
   445: 
```

### 9. Destination mutation

File: `programs/treasury-router/src/engines/integrity_firewall.rs:441`

```text
   436:         let settlement_mint = Pubkey::new_unique();
   437:         let founder_recipient = Pubkey::new_unique();
   438:         let company_recipient = Pubkey::new_unique();
   439: 
   440:         let reserve_destination = Pubkey::new_unique();
>  441:         let buyback_destination = Pubkey::new_unique();
   442:         let liquidity_destination = Pubkey::new_unique();
   443:         let company_destination = Pubkey::new_unique();
   444:         let founder_destination = Pubkey::new_unique();
   445: 
   446:         let protocol = ProtocolState {
```

### 10. Destination mutation

File: `programs/treasury-router/src/engines/integrity_firewall.rs:442`

```text
   437:         let founder_recipient = Pubkey::new_unique();
   438:         let company_recipient = Pubkey::new_unique();
   439: 
   440:         let reserve_destination = Pubkey::new_unique();
   441:         let buyback_destination = Pubkey::new_unique();
>  442:         let liquidity_destination = Pubkey::new_unique();
   443:         let company_destination = Pubkey::new_unique();
   444:         let founder_destination = Pubkey::new_unique();
   445: 
   446:         let protocol = ProtocolState {
   447:             version: PROTOCOL_VERSION,
```

### 11. Destination mutation

File: `programs/treasury-router/src/engines/integrity_firewall.rs:443`

```text
   438:         let company_recipient = Pubkey::new_unique();
   439: 
   440:         let reserve_destination = Pubkey::new_unique();
   441:         let buyback_destination = Pubkey::new_unique();
   442:         let liquidity_destination = Pubkey::new_unique();
>  443:         let company_destination = Pubkey::new_unique();
   444:         let founder_destination = Pubkey::new_unique();
   445: 
   446:         let protocol = ProtocolState {
   447:             version: PROTOCOL_VERSION,
   448:             authority: Pubkey::new_unique(),
```

### 12. Destination mutation

File: `programs/treasury-router/src/engines/integrity_firewall.rs:444`

```text
   439: 
   440:         let reserve_destination = Pubkey::new_unique();
   441:         let buyback_destination = Pubkey::new_unique();
   442:         let liquidity_destination = Pubkey::new_unique();
   443:         let company_destination = Pubkey::new_unique();
>  444:         let founder_destination = Pubkey::new_unique();
   445: 
   446:         let protocol = ProtocolState {
   447:             version: PROTOCOL_VERSION,
   448:             authority: Pubkey::new_unique(),
   449:             protocol_config: protocol_config_key,
```

### 13. Signer constraint

File: `programs/treasury-router/src/engines/sentinel.rs:345`

```text
   340: ///
   341: /// `expected_protocol` must be the canonical ProtocolState public key supplied
   342: /// by the caller or monitoring layer.
   343: ///
   344: /// Sentinel deliberately accepts only plain values and references. It does not
>  345: /// receive mutable accounts or signer authority.
   346: pub fn evaluate_linkage(
   347:     expected_protocol: Pubkey,
   348:     protocol_config: &ProtocolConfig,
   349:     treasury: &TreasuryState,
   350: ) -> SentinelLinkageReport {
```

### 14. Configuration mutation

File: `programs/treasury-router/src/engines/sentinel.rs:357`

```text
   352: 
   353:     /*
   354:      * The configuration and treasury must both point to the same canonical
   355:      * ProtocolState.
   356:      */
>  357:     let protocol_config_link_valid = protocol_config.protocol == expected_protocol;
   358: 
   359:     if !protocol_config_link_valid {
   360:         failure_mask |= SentinelLinkageInvariant::ProtocolConfigLink.mask();
   361:     }
   362: 
```

### 15. Configuration mutation

File: `programs/treasury-router/src/engines/sentinel.rs:375`

```text
   370:      * Verify the exact locked 30/20/20/20/10 allocation.
   371:      *
   372:      * Merely totalling 10,000 basis points is insufficient: a corrupted
   373:      * configuration could still total 100% while changing the distribution.
   374:      */
>  375:     let locked_beavernomics_valid = protocol_config.reserve_bps == LOCKED_RESERVE_BPS
   376:         && protocol_config.buyback_burn_bps == LOCKED_BUYBACK_BURN_BPS
   377:         && protocol_config.liquidity_bps == LOCKED_LIQUIDITY_BPS
   378:         && protocol_config.company_bps == LOCKED_COMPANY_BPS
   379:         && protocol_config.founder_bps == LOCKED_FOUNDER_BPS;
   380: 
```

### 16. Configuration mutation

File: `programs/treasury-router/src/engines/sentinel.rs:376`

```text
   371:      *
   372:      * Merely totalling 10,000 basis points is insufficient: a corrupted
   373:      * configuration could still total 100% while changing the distribution.
   374:      */
   375:     let locked_beavernomics_valid = protocol_config.reserve_bps == LOCKED_RESERVE_BPS
>  376:         && protocol_config.buyback_burn_bps == LOCKED_BUYBACK_BURN_BPS
   377:         && protocol_config.liquidity_bps == LOCKED_LIQUIDITY_BPS
   378:         && protocol_config.company_bps == LOCKED_COMPANY_BPS
   379:         && protocol_config.founder_bps == LOCKED_FOUNDER_BPS;
   380: 
   381:     if !locked_beavernomics_valid {
```

### 17. Configuration mutation

File: `programs/treasury-router/src/engines/sentinel.rs:377`

```text
   372:      * Merely totalling 10,000 basis points is insufficient: a corrupted
   373:      * configuration could still total 100% while changing the distribution.
   374:      */
   375:     let locked_beavernomics_valid = protocol_config.reserve_bps == LOCKED_RESERVE_BPS
   376:         && protocol_config.buyback_burn_bps == LOCKED_BUYBACK_BURN_BPS
>  377:         && protocol_config.liquidity_bps == LOCKED_LIQUIDITY_BPS
   378:         && protocol_config.company_bps == LOCKED_COMPANY_BPS
   379:         && protocol_config.founder_bps == LOCKED_FOUNDER_BPS;
   380: 
   381:     if !locked_beavernomics_valid {
   382:         failure_mask |= SentinelLinkageInvariant::LockedBeavernomics.mask();
```

### 18. Configuration mutation

File: `programs/treasury-router/src/engines/sentinel.rs:378`

```text
   373:      * configuration could still total 100% while changing the distribution.
   374:      */
   375:     let locked_beavernomics_valid = protocol_config.reserve_bps == LOCKED_RESERVE_BPS
   376:         && protocol_config.buyback_burn_bps == LOCKED_BUYBACK_BURN_BPS
   377:         && protocol_config.liquidity_bps == LOCKED_LIQUIDITY_BPS
>  378:         && protocol_config.company_bps == LOCKED_COMPANY_BPS
   379:         && protocol_config.founder_bps == LOCKED_FOUNDER_BPS;
   380: 
   381:     if !locked_beavernomics_valid {
   382:         failure_mask |= SentinelLinkageInvariant::LockedBeavernomics.mask();
   383:     }
```

### 19. Configuration mutation

File: `programs/treasury-router/src/engines/sentinel.rs:379`

```text
   374:      */
   375:     let locked_beavernomics_valid = protocol_config.reserve_bps == LOCKED_RESERVE_BPS
   376:         && protocol_config.buyback_burn_bps == LOCKED_BUYBACK_BURN_BPS
   377:         && protocol_config.liquidity_bps == LOCKED_LIQUIDITY_BPS
   378:         && protocol_config.company_bps == LOCKED_COMPANY_BPS
>  379:         && protocol_config.founder_bps == LOCKED_FOUNDER_BPS;
   380: 
   381:     if !locked_beavernomics_valid {
   382:         failure_mask |= SentinelLinkageInvariant::LockedBeavernomics.mask();
   383:     }
   384: 
```

### 20. Signer constraint

File: `programs/treasury-router/src/instructions/buyback.rs:178`

```text
   173:             @ TreasuryRouterError::IntegrityFirewallViolation
   174:     )]
   175:     pub founder_destination: Box<Account<'info, TokenAccount>>,
   176: 
   177:     /// Permissionless transaction caller and fee payer.
>  178:     /// This signer does not need to match ProtocolState.authority.
   179:     pub token_program: Program<'info, Token>,
   180: }
   181: 
   182: /// Releases previously allocated buyback-and-burn funds.
   183: ///
```

### 21. Authority assignment

File: `programs/treasury-router/src/instructions/buyback.rs:271`

```text
   266: 
   267:     let transfer_accounts = TransferChecked {
   268:         from: ctx.accounts.settlement_vault.to_account_info(),
   269:         mint: ctx.accounts.settlement_mint.to_account_info(),
   270:         to: ctx.accounts.buyback_destination.to_account_info(),
>  271:         authority: ctx.accounts.treasury.to_account_info(),
   272:     };
   273: 
   274:     let cpi_context = CpiContext::new_with_signer(
   275:         ctx.accounts.token_program.key(),
   276:         transfer_accounts,
```

### 22. Configuration mutation

File: `programs/treasury-router/src/instructions/buyback.rs:30`

```text
    25:     #[account(
    26:         seeds = [PROTOCOL_SEED],
    27:         bump = protocol_state.bump,
    28:         constraint = protocol_state.treasury_state != Pubkey::default()
    29:             @ TreasuryRouterError::TreasuryNotInitialized,
>   30:         constraint = protocol_state.treasury_state == treasury.key()
    31:             @ TreasuryRouterError::InvalidTreasuryProtocol
    32:     )]
    33:     pub protocol_state: Box<Account<'info, ProtocolState>>,
    34: 
    35:     #[account(
```

### 23. Configuration mutation

File: `programs/treasury-router/src/instructions/buyback.rs:41`

```text
    36:         seeds = [
    37:             PROTOCOL_CONFIG_SEED,
    38:             protocol_state.key().as_ref()
    39:         ],
    40:         bump = protocol_config.bump,
>   41:         constraint = protocol_state.protocol_config == protocol_config.key()
    42:             @ TreasuryRouterError::IntegrityFirewallViolation,
    43:         constraint = protocol_config.protocol == protocol_state.key()
    44:             @ TreasuryRouterError::IntegrityFirewallViolation
    45:     )]
    46:     pub protocol_config: Box<Account<'info, ProtocolConfig>>,
```

### 24. Configuration mutation

File: `programs/treasury-router/src/instructions/buyback.rs:43`

```text
    38:             protocol_state.key().as_ref()
    39:         ],
    40:         bump = protocol_config.bump,
    41:         constraint = protocol_state.protocol_config == protocol_config.key()
    42:             @ TreasuryRouterError::IntegrityFirewallViolation,
>   43:         constraint = protocol_config.protocol == protocol_state.key()
    44:             @ TreasuryRouterError::IntegrityFirewallViolation
    45:     )]
    46:     pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    47: 
    48:     #[account(
```

### 25. Configuration mutation

File: `programs/treasury-router/src/instructions/buyback.rs:54`

```text
    49:         seeds = [
    50:             FOUNDER_STATE_SEED,
    51:             protocol_state.key().as_ref()
    52:         ],
    53:         bump = founder_state.bump,
>   54:         constraint = protocol_state.founder_state == founder_state.key()
    55:             @ TreasuryRouterError::IntegrityFirewallViolation,
    56:         constraint = founder_state.protocol == protocol_state.key()
    57:             @ TreasuryRouterError::IntegrityFirewallViolation
    58:     )]
    59:     pub founder_state: Box<Account<'info, FounderState>>,
```

### 26. Configuration mutation

File: `programs/treasury-router/src/instructions/buyback.rs:67`

```text
    62:         seeds = [
    63:             COMPANY_STATE_SEED,
    64:             protocol_state.key().as_ref()
    65:         ],
    66:         bump = company_state.bump,
>   67:         constraint = protocol_state.company_state == company_state.key()
    68:             @ TreasuryRouterError::IntegrityFirewallViolation,
    69:         constraint = company_state.protocol == protocol_state.key()
    70:             @ TreasuryRouterError::IntegrityFirewallViolation
    71:     )]
    72:     pub company_state: Box<Account<'info, CompanyState>>,
```

### 27. Configuration mutation

File: `programs/treasury-router/src/instructions/buyback.rs:96`

```text
    91:         seeds = [
    92:             EXECUTION_CONFIG_SEED,
    93:             protocol_state.key().as_ref()
    94:         ],
    95:         bump = execution_config.bump,
>   96:         constraint = execution_config.protocol_state == protocol_state.key()
    97:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    98:         constraint = execution_config.settlement_mint == settlement_mint.key()
    99:             @ TreasuryRouterError::InvalidSettlementMint
   100:     )]
   101:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
```

### 28. Configuration mutation

File: `programs/treasury-router/src/instructions/buyback.rs:98`

```text
    93:             protocol_state.key().as_ref()
    94:         ],
    95:         bump = execution_config.bump,
    96:         constraint = execution_config.protocol_state == protocol_state.key()
    97:             @ TreasuryRouterError::InvalidTreasuryProtocol,
>   98:         constraint = execution_config.settlement_mint == settlement_mint.key()
    99:             @ TreasuryRouterError::InvalidSettlementMint
   100:     )]
   101:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
   102: 
   103:     #[account(
```

### 29. Signer constraint

File: `programs/treasury-router/src/instructions/company.rs:100`

```text
    95:             @ TreasuryRouterError::InvalidExecutionDestination
    96:     )]
    97:     pub company_destination: Box<Account<'info, TokenAccount>>,
    98: 
    99:     /// Permissionless transaction caller and fee payer.
>  100:     /// This signer does not need to match ProtocolState.authority.
   101:     pub token_program: Program<'info, Token>,
   102: }
   103: 
   104: /// Releases previously allocated company funds.
   105: ///
```

### 30. Authority assignment

File: `programs/treasury-router/src/instructions/company.rs:131`

```text
   126: 
   127:     let transfer_accounts = TransferChecked {
   128:         from: ctx.accounts.settlement_vault.to_account_info(),
   129:         mint: ctx.accounts.settlement_mint.to_account_info(),
   130:         to: ctx.accounts.company_destination.to_account_info(),
>  131:         authority: ctx.accounts.treasury.to_account_info(),
   132:     };
   133: 
   134:     let cpi_context = CpiContext::new_with_signer(
   135:         ctx.accounts.token_program.key(),
   136:         transfer_accounts,
```

### 31. Configuration mutation

File: `programs/treasury-router/src/instructions/company.rs:20`

```text
    15:     #[account(
    16:         seeds = [PROTOCOL_SEED],
    17:         bump = protocol_state.bump,
    18:         constraint = protocol_state.treasury_state != Pubkey::default()
    19:             @ TreasuryRouterError::TreasuryNotInitialized,
>   20:         constraint = protocol_state.treasury_state == treasury.key()
    21:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    22:         constraint = protocol_state.company_state == company_state.key()
    23:             @ TreasuryRouterError::InvalidTreasuryProtocol
    24:     )]
    25:     pub protocol_state: Box<Account<'info, ProtocolState>>,
```

### 32. Configuration mutation

File: `programs/treasury-router/src/instructions/company.rs:22`

```text
    17:         bump = protocol_state.bump,
    18:         constraint = protocol_state.treasury_state != Pubkey::default()
    19:             @ TreasuryRouterError::TreasuryNotInitialized,
    20:         constraint = protocol_state.treasury_state == treasury.key()
    21:             @ TreasuryRouterError::InvalidTreasuryProtocol,
>   22:         constraint = protocol_state.company_state == company_state.key()
    23:             @ TreasuryRouterError::InvalidTreasuryProtocol
    24:     )]
    25:     pub protocol_state: Box<Account<'info, ProtocolState>>,
    26: 
    27:     #[account(
```

### 33. Configuration mutation

File: `programs/treasury-router/src/instructions/company.rs:60`

```text
    55:         seeds = [
    56:             EXECUTION_CONFIG_SEED,
    57:             protocol_state.key().as_ref()
    58:         ],
    59:         bump = execution_config.bump,
>   60:         constraint = execution_config.protocol_state == protocol_state.key()
    61:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    62:         constraint = execution_config.settlement_mint == settlement_mint.key()
    63:             @ TreasuryRouterError::InvalidSettlementMint
    64:     )]
    65:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
```

### 34. Configuration mutation

File: `programs/treasury-router/src/instructions/company.rs:62`

```text
    57:             protocol_state.key().as_ref()
    58:         ],
    59:         bump = execution_config.bump,
    60:         constraint = execution_config.protocol_state == protocol_state.key()
    61:             @ TreasuryRouterError::InvalidTreasuryProtocol,
>   62:         constraint = execution_config.settlement_mint == settlement_mint.key()
    63:             @ TreasuryRouterError::InvalidSettlementMint
    64:     )]
    65:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
    66: 
    67:     #[account(
```

### 35. Signer account

File: `programs/treasury-router/src/instructions/deposit_settlement.rs:64`

```text
    59:         constraint = settlement_vault.owner == treasury.key()
    60:             @ TreasuryRouterError::InvalidSettlementVault
    61:     )]
    62:     pub settlement_vault: Box<Account<'info, TokenAccount>>,
    63: 
>   64:     pub authority: Signer<'info>,
    65: 
    66:     pub token_program: Program<'info, Token>,
    67: }
    68: 
    69: pub fn handler(ctx: Context<DepositSettlement>, amount: u64) -> Result<()> {
```

### 36. Authority comparison

File: `programs/treasury-router/src/instructions/deposit_settlement.rs:15`

```text
    10: #[derive(Accounts)]
    11: pub struct DepositSettlement<'info> {
    12:     #[account(
    13:         seeds = [PROTOCOL_SEED],
    14:         bump = protocol_state.bump,
>   15:         has_one = authority,
    16:         constraint = protocol_state.treasury_state != Pubkey::default()
    17:             @ TreasuryRouterError::TreasuryNotInitialized,
    18:         constraint = protocol_state.treasury_state == treasury.key()
    19:             @ TreasuryRouterError::InvalidTreasuryProtocol
    20:     )]
```

### 37. Authority assignment

File: `programs/treasury-router/src/instructions/deposit_settlement.rs:81`

```text
    76: 
    77:     let transfer_accounts = TransferChecked {
    78:         from: ctx.accounts.source_token_account.to_account_info(),
    79:         mint: ctx.accounts.settlement_mint.to_account_info(),
    80:         to: ctx.accounts.settlement_vault.to_account_info(),
>   81:         authority: ctx.accounts.authority.to_account_info(),
    82:     };
    83: 
    84:     let cpi_context = CpiContext::new(ctx.accounts.token_program.key(), transfer_accounts);
    85: 
    86:     token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
```

### 38. Configuration mutation

File: `programs/treasury-router/src/instructions/deposit_settlement.rs:18`

```text
    13:         seeds = [PROTOCOL_SEED],
    14:         bump = protocol_state.bump,
    15:         has_one = authority,
    16:         constraint = protocol_state.treasury_state != Pubkey::default()
    17:             @ TreasuryRouterError::TreasuryNotInitialized,
>   18:         constraint = protocol_state.treasury_state == treasury.key()
    19:             @ TreasuryRouterError::InvalidTreasuryProtocol
    20:     )]
    21:     pub protocol_state: Box<Account<'info, ProtocolState>>,
    22: 
    23:     #[account(
```

### 39. Signer constraint

File: `programs/treasury-router/src/instructions/founder.rs:100`

```text
    95:             @ TreasuryRouterError::InvalidExecutionDestination
    96:     )]
    97:     pub founder_destination: Box<Account<'info, TokenAccount>>,
    98: 
    99:     /// Permissionless transaction caller and fee payer.
>  100:     /// This signer does not need to match ProtocolState.authority.
   101:     pub token_program: Program<'info, Token>,
   102: }
   103: 
   104: /// Releases founder funds already allocated during fee processing.
   105: ///
```

### 40. Authority assignment

File: `programs/treasury-router/src/instructions/founder.rs:131`

```text
   126: 
   127:     let transfer_accounts = TransferChecked {
   128:         from: ctx.accounts.settlement_vault.to_account_info(),
   129:         mint: ctx.accounts.settlement_mint.to_account_info(),
   130:         to: ctx.accounts.founder_destination.to_account_info(),
>  131:         authority: ctx.accounts.treasury.to_account_info(),
   132:     };
   133: 
   134:     let cpi_context = CpiContext::new_with_signer(
   135:         ctx.accounts.token_program.key(),
   136:         transfer_accounts,
```

### 41. Configuration mutation

File: `programs/treasury-router/src/instructions/founder.rs:20`

```text
    15:     #[account(
    16:         seeds = [PROTOCOL_SEED],
    17:         bump = protocol_state.bump,
    18:         constraint = protocol_state.treasury_state != Pubkey::default()
    19:             @ TreasuryRouterError::TreasuryNotInitialized,
>   20:         constraint = protocol_state.treasury_state == treasury.key()
    21:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    22:         constraint = protocol_state.founder_state == founder_state.key()
    23:             @ TreasuryRouterError::InvalidTreasuryProtocol
    24:     )]
    25:     pub protocol_state: Box<Account<'info, ProtocolState>>,
```

### 42. Configuration mutation

File: `programs/treasury-router/src/instructions/founder.rs:22`

```text
    17:         bump = protocol_state.bump,
    18:         constraint = protocol_state.treasury_state != Pubkey::default()
    19:             @ TreasuryRouterError::TreasuryNotInitialized,
    20:         constraint = protocol_state.treasury_state == treasury.key()
    21:             @ TreasuryRouterError::InvalidTreasuryProtocol,
>   22:         constraint = protocol_state.founder_state == founder_state.key()
    23:             @ TreasuryRouterError::InvalidTreasuryProtocol
    24:     )]
    25:     pub protocol_state: Box<Account<'info, ProtocolState>>,
    26: 
    27:     #[account(
```

### 43. Configuration mutation

File: `programs/treasury-router/src/instructions/founder.rs:60`

```text
    55:         seeds = [
    56:             EXECUTION_CONFIG_SEED,
    57:             protocol_state.key().as_ref()
    58:         ],
    59:         bump = execution_config.bump,
>   60:         constraint = execution_config.protocol_state == protocol_state.key()
    61:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    62:         constraint = execution_config.settlement_mint == settlement_mint.key()
    63:             @ TreasuryRouterError::InvalidSettlementMint
    64:     )]
    65:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
```

### 44. Configuration mutation

File: `programs/treasury-router/src/instructions/founder.rs:62`

```text
    57:             protocol_state.key().as_ref()
    58:         ],
    59:         bump = execution_config.bump,
    60:         constraint = execution_config.protocol_state == protocol_state.key()
    61:             @ TreasuryRouterError::InvalidTreasuryProtocol,
>   62:         constraint = execution_config.settlement_mint == settlement_mint.key()
    63:             @ TreasuryRouterError::InvalidSettlementMint
    64:     )]
    65:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
    66: 
    67:     #[account(
```

### 45. Signer account

File: `programs/treasury-router/src/instructions/initialize.rs:20`

```text
    15:         bump
    16:     )]
    17:     pub protocol_state: Account<'info, ProtocolState>,
    18: 
    19:     #[account(mut)]
>   20:     pub authority: Signer<'info>,
    21: 
    22:     pub system_program: Program<'info, System>,
    23: }
    24: 
    25: pub fn handler(ctx: Context<Initialize>) -> Result<()> {
```

### 46. Authority assignment

File: `programs/treasury-router/src/instructions/initialize.rs:30`

```text
    25: pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    26:     let protocol_state = &mut ctx.accounts.protocol_state;
    27:     let clock = Clock::get()?;
    28: 
    29:     protocol_state.version = PROTOCOL_VERSION;
>   30:     protocol_state.authority = ctx.accounts.authority.key();
    31: 
    32:     protocol_state.protocol_config = Pubkey::default();
    33:     protocol_state.treasury_state = Pubkey::default();
    34:     protocol_state.reserve_state = Pubkey::default();
    35:     protocol_state.liquidity_state = Pubkey::default();
```

### 47. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:29`

```text
    24: 
    25: pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    26:     let protocol_state = &mut ctx.accounts.protocol_state;
    27:     let clock = Clock::get()?;
    28: 
>   29:     protocol_state.version = PROTOCOL_VERSION;
    30:     protocol_state.authority = ctx.accounts.authority.key();
    31: 
    32:     protocol_state.protocol_config = Pubkey::default();
    33:     protocol_state.treasury_state = Pubkey::default();
    34:     protocol_state.reserve_state = Pubkey::default();
```

### 48. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:30`

```text
    25: pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    26:     let protocol_state = &mut ctx.accounts.protocol_state;
    27:     let clock = Clock::get()?;
    28: 
    29:     protocol_state.version = PROTOCOL_VERSION;
>   30:     protocol_state.authority = ctx.accounts.authority.key();
    31: 
    32:     protocol_state.protocol_config = Pubkey::default();
    33:     protocol_state.treasury_state = Pubkey::default();
    34:     protocol_state.reserve_state = Pubkey::default();
    35:     protocol_state.liquidity_state = Pubkey::default();
```

### 49. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:32`

```text
    27:     let clock = Clock::get()?;
    28: 
    29:     protocol_state.version = PROTOCOL_VERSION;
    30:     protocol_state.authority = ctx.accounts.authority.key();
    31: 
>   32:     protocol_state.protocol_config = Pubkey::default();
    33:     protocol_state.treasury_state = Pubkey::default();
    34:     protocol_state.reserve_state = Pubkey::default();
    35:     protocol_state.liquidity_state = Pubkey::default();
    36:     protocol_state.founder_state = Pubkey::default();
    37:     protocol_state.company_state = Pubkey::default();
```

### 50. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:33`

```text
    28: 
    29:     protocol_state.version = PROTOCOL_VERSION;
    30:     protocol_state.authority = ctx.accounts.authority.key();
    31: 
    32:     protocol_state.protocol_config = Pubkey::default();
>   33:     protocol_state.treasury_state = Pubkey::default();
    34:     protocol_state.reserve_state = Pubkey::default();
    35:     protocol_state.liquidity_state = Pubkey::default();
    36:     protocol_state.founder_state = Pubkey::default();
    37:     protocol_state.company_state = Pubkey::default();
    38:     protocol_state.buyback_state = Pubkey::default();
```

### 51. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:34`

```text
    29:     protocol_state.version = PROTOCOL_VERSION;
    30:     protocol_state.authority = ctx.accounts.authority.key();
    31: 
    32:     protocol_state.protocol_config = Pubkey::default();
    33:     protocol_state.treasury_state = Pubkey::default();
>   34:     protocol_state.reserve_state = Pubkey::default();
    35:     protocol_state.liquidity_state = Pubkey::default();
    36:     protocol_state.founder_state = Pubkey::default();
    37:     protocol_state.company_state = Pubkey::default();
    38:     protocol_state.buyback_state = Pubkey::default();
    39: 
```

### 52. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:35`

```text
    30:     protocol_state.authority = ctx.accounts.authority.key();
    31: 
    32:     protocol_state.protocol_config = Pubkey::default();
    33:     protocol_state.treasury_state = Pubkey::default();
    34:     protocol_state.reserve_state = Pubkey::default();
>   35:     protocol_state.liquidity_state = Pubkey::default();
    36:     protocol_state.founder_state = Pubkey::default();
    37:     protocol_state.company_state = Pubkey::default();
    38:     protocol_state.buyback_state = Pubkey::default();
    39: 
    40:     protocol_state.beaver_score = 0;
```

### 53. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:36`

```text
    31: 
    32:     protocol_state.protocol_config = Pubkey::default();
    33:     protocol_state.treasury_state = Pubkey::default();
    34:     protocol_state.reserve_state = Pubkey::default();
    35:     protocol_state.liquidity_state = Pubkey::default();
>   36:     protocol_state.founder_state = Pubkey::default();
    37:     protocol_state.company_state = Pubkey::default();
    38:     protocol_state.buyback_state = Pubkey::default();
    39: 
    40:     protocol_state.beaver_score = 0;
    41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
```

### 54. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:37`

```text
    32:     protocol_state.protocol_config = Pubkey::default();
    33:     protocol_state.treasury_state = Pubkey::default();
    34:     protocol_state.reserve_state = Pubkey::default();
    35:     protocol_state.liquidity_state = Pubkey::default();
    36:     protocol_state.founder_state = Pubkey::default();
>   37:     protocol_state.company_state = Pubkey::default();
    38:     protocol_state.buyback_state = Pubkey::default();
    39: 
    40:     protocol_state.beaver_score = 0;
    41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
    42:     protocol_state.paused = false;
```

### 55. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:38`

```text
    33:     protocol_state.treasury_state = Pubkey::default();
    34:     protocol_state.reserve_state = Pubkey::default();
    35:     protocol_state.liquidity_state = Pubkey::default();
    36:     protocol_state.founder_state = Pubkey::default();
    37:     protocol_state.company_state = Pubkey::default();
>   38:     protocol_state.buyback_state = Pubkey::default();
    39: 
    40:     protocol_state.beaver_score = 0;
    41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
    42:     protocol_state.paused = false;
    43:     protocol_state.bump = ctx.bumps.protocol_state;
```

### 56. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:40`

```text
    35:     protocol_state.liquidity_state = Pubkey::default();
    36:     protocol_state.founder_state = Pubkey::default();
    37:     protocol_state.company_state = Pubkey::default();
    38:     protocol_state.buyback_state = Pubkey::default();
    39: 
>   40:     protocol_state.beaver_score = 0;
    41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
    42:     protocol_state.paused = false;
    43:     protocol_state.bump = ctx.bumps.protocol_state;
    44:     protocol_state.initialized_at = clock.unix_timestamp;
    45:     protocol_state.reserved = [0; 64];
```

### 57. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:41`

```text
    36:     protocol_state.founder_state = Pubkey::default();
    37:     protocol_state.company_state = Pubkey::default();
    38:     protocol_state.buyback_state = Pubkey::default();
    39: 
    40:     protocol_state.beaver_score = 0;
>   41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
    42:     protocol_state.paused = false;
    43:     protocol_state.bump = ctx.bumps.protocol_state;
    44:     protocol_state.initialized_at = clock.unix_timestamp;
    45:     protocol_state.reserved = [0; 64];
    46: 
```

### 58. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:42`

```text
    37:     protocol_state.company_state = Pubkey::default();
    38:     protocol_state.buyback_state = Pubkey::default();
    39: 
    40:     protocol_state.beaver_score = 0;
    41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
>   42:     protocol_state.paused = false;
    43:     protocol_state.bump = ctx.bumps.protocol_state;
    44:     protocol_state.initialized_at = clock.unix_timestamp;
    45:     protocol_state.reserved = [0; 64];
    46: 
    47:     msg!(
```

### 59. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:43`

```text
    38:     protocol_state.buyback_state = Pubkey::default();
    39: 
    40:     protocol_state.beaver_score = 0;
    41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
    42:     protocol_state.paused = false;
>   43:     protocol_state.bump = ctx.bumps.protocol_state;
    44:     protocol_state.initialized_at = clock.unix_timestamp;
    45:     protocol_state.reserved = [0; 64];
    46: 
    47:     msg!(
    48:         "Rocket Beaver ProtocolState initialized: {}",
```

### 60. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:44`

```text
    39: 
    40:     protocol_state.beaver_score = 0;
    41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
    42:     protocol_state.paused = false;
    43:     protocol_state.bump = ctx.bumps.protocol_state;
>   44:     protocol_state.initialized_at = clock.unix_timestamp;
    45:     protocol_state.reserved = [0; 64];
    46: 
    47:     msg!(
    48:         "Rocket Beaver ProtocolState initialized: {}",
    49:         protocol_state.key()
```

### 61. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize.rs:45`

```text
    40:     protocol_state.beaver_score = 0;
    41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
    42:     protocol_state.paused = false;
    43:     protocol_state.bump = ctx.bumps.protocol_state;
    44:     protocol_state.initialized_at = clock.unix_timestamp;
>   45:     protocol_state.reserved = [0; 64];
    46: 
    47:     msg!(
    48:         "Rocket Beaver ProtocolState initialized: {}",
    49:         protocol_state.key()
    50:     );
```

### 62. Pause mutation

File: `programs/treasury-router/src/instructions/initialize.rs:42`

```text
    37:     protocol_state.company_state = Pubkey::default();
    38:     protocol_state.buyback_state = Pubkey::default();
    39: 
    40:     protocol_state.beaver_score = 0;
    41:     protocol_state.dam_level = INITIAL_DAM_LEVEL;
>   42:     protocol_state.paused = false;
    43:     protocol_state.bump = ctx.bumps.protocol_state;
    44:     protocol_state.initialized_at = clock.unix_timestamp;
    45:     protocol_state.reserved = [0; 64];
    46: 
    47:     msg!(
```

### 63. Signer account

File: `programs/treasury-router/src/instructions/initialize_company.rs:32`

```text
    27:         bump
    28:     )]
    29:     pub company_state: Account<'info, CompanyState>,
    30: 
    31:     #[account(mut)]
>   32:     pub authority: Signer<'info>,
    33: 
    34:     pub system_program: Program<'info, System>,
    35: }
    36: 
    37: pub fn handler(
```

### 64. Authority comparison

File: `programs/treasury-router/src/instructions/initialize_company.rs:15`

```text
    10: pub struct InitializeCompany<'info> {
    11:     #[account(
    12:         mut,
    13:         seeds = [PROTOCOL_SEED],
    14:         bump = protocol_state.bump,
>   15:         has_one = authority
    16:     )]
    17:     pub protocol_state: Account<'info, ProtocolState>,
    18: 
    19:     #[account(
    20:         init,
```

### 65. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_company.rs:49`

```text
    44:         !ctx.accounts.protocol_state.paused,
    45:         TreasuryRouterError::ProtocolPaused
    46:     );
    47: 
    48:     require!(
>   49:         ctx.accounts.protocol_state.company_state == Pubkey::default(),
    50:         TreasuryRouterError::CompanyAlreadyInitialized
    51:     );
    52: 
    53:     require!(
    54:         recipient != Pubkey::default(),
```

### 66. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_company.rs:81`

```text
    76:     company_state.period_duration = period_duration;
    77:     company_state.enabled = true;
    78:     company_state.bump = ctx.bumps.company_state;
    79:     company_state.reserved = [0; 64];
    80: 
>   81:     ctx.accounts.protocol_state.company_state = company_state.key();
    82: 
    83:     msg!("CompanyState initialized: {}", company_state.key());
    84:     msg!("Company recipient: {}", company_state.recipient);
    85:     msg!("Company period cap: {}", company_state.period_cap);
    86:     msg!(
```

### 67. Signer account

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:130`

```text
   125:         bump
   126:     )]
   127:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
   128: 
   129:     #[account(mut)]
>  130:     pub authority: Signer<'info>,
   131: 
   132:     pub system_program: Program<'info, System>,
   133: }
   134: 
   135: pub fn handler(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
```

### 68. Authority comparison

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:18`

```text
    13: #[derive(Accounts)]
    14: pub struct InitializeExecutionConfig<'info> {
    15:     #[account(
    16:         seeds = [PROTOCOL_SEED],
    17:         bump = protocol_state.bump,
>   18:         has_one = authority,
    19:         constraint = !protocol_state.paused
    20:             @ TreasuryRouterError::ProtocolPaused,
    21:         constraint = protocol_state.treasury_state == treasury.key()
    22:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    23:         constraint = protocol_state.founder_state == founder_state.key()
```

### 69. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:21`

```text
    16:         seeds = [PROTOCOL_SEED],
    17:         bump = protocol_state.bump,
    18:         has_one = authority,
    19:         constraint = !protocol_state.paused
    20:             @ TreasuryRouterError::ProtocolPaused,
>   21:         constraint = protocol_state.treasury_state == treasury.key()
    22:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    23:         constraint = protocol_state.founder_state == founder_state.key()
    24:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    25:         constraint = protocol_state.company_state == company_state.key()
    26:             @ TreasuryRouterError::InvalidTreasuryProtocol
```

### 70. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:23`

```text
    18:         has_one = authority,
    19:         constraint = !protocol_state.paused
    20:             @ TreasuryRouterError::ProtocolPaused,
    21:         constraint = protocol_state.treasury_state == treasury.key()
    22:             @ TreasuryRouterError::InvalidTreasuryProtocol,
>   23:         constraint = protocol_state.founder_state == founder_state.key()
    24:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    25:         constraint = protocol_state.company_state == company_state.key()
    26:             @ TreasuryRouterError::InvalidTreasuryProtocol
    27:     )]
    28:     pub protocol_state: Box<Account<'info, ProtocolState>>,
```

### 71. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:25`

```text
    20:             @ TreasuryRouterError::ProtocolPaused,
    21:         constraint = protocol_state.treasury_state == treasury.key()
    22:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    23:         constraint = protocol_state.founder_state == founder_state.key()
    24:             @ TreasuryRouterError::InvalidTreasuryProtocol,
>   25:         constraint = protocol_state.company_state == company_state.key()
    26:             @ TreasuryRouterError::InvalidTreasuryProtocol
    27:     )]
    28:     pub protocol_state: Box<Account<'info, ProtocolState>>,
    29: 
    30:     #[account(
```

### 72. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:225`

```text
   220:         TreasuryRouterError::InvalidExecutionDestination
   221:     );
   222: 
   223:     let execution_config = &mut ctx.accounts.execution_config;
   224: 
>  225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
```

### 73. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:226`

```text
   221:     );
   222: 
   223:     let execution_config = &mut ctx.accounts.execution_config;
   224: 
   225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
>  226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
```

### 74. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:228`

```text
   223:     let execution_config = &mut ctx.accounts.execution_config;
   224: 
   225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
>  228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
```

### 75. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:229`

```text
   224: 
   225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
>  229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
```

### 76. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:230`

```text
   225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
>  230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
   235:     execution_config.bump = ctx.bumps.execution_config;
```

### 77. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:231`

```text
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
>  231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
   235:     execution_config.bump = ctx.bumps.execution_config;
   236: 
```

### 78. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:232`

```text
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
>  232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
   235:     execution_config.bump = ctx.bumps.execution_config;
   236: 
   237:     msg!(
```

### 79. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:234`

```text
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
>  234:     execution_config.version = EXECUTION_CONFIG_VERSION;
   235:     execution_config.bump = ctx.bumps.execution_config;
   236: 
   237:     msg!(
   238:         "Immutable ExecutionConfig initialized: {}",
   239:         execution_config.key()
```

### 80. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:235`

```text
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
>  235:     execution_config.bump = ctx.bumps.execution_config;
   236: 
   237:     msg!(
   238:         "Immutable ExecutionConfig initialized: {}",
   239:         execution_config.key()
   240:     );
```

### 81. Destination mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:138`

```text
   133: }
   134: 
   135: pub fn handler(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
   136:     let settlement_vault = ctx.accounts.settlement_vault.key();
   137: 
>  138:     let reserve_destination = ctx.accounts.reserve_destination.key();
   139:     let buyback_destination = ctx.accounts.buyback_destination.key();
   140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
   141:     let company_destination = ctx.accounts.company_destination.key();
   142:     let founder_destination = ctx.accounts.founder_destination.key();
   143: 
```

### 82. Destination mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:139`

```text
   134: 
   135: pub fn handler(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
   136:     let settlement_vault = ctx.accounts.settlement_vault.key();
   137: 
   138:     let reserve_destination = ctx.accounts.reserve_destination.key();
>  139:     let buyback_destination = ctx.accounts.buyback_destination.key();
   140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
   141:     let company_destination = ctx.accounts.company_destination.key();
   142:     let founder_destination = ctx.accounts.founder_destination.key();
   143: 
   144:     // No allocation destination may point back to the treasury vault.
```

### 83. Destination mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:140`

```text
   135: pub fn handler(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
   136:     let settlement_vault = ctx.accounts.settlement_vault.key();
   137: 
   138:     let reserve_destination = ctx.accounts.reserve_destination.key();
   139:     let buyback_destination = ctx.accounts.buyback_destination.key();
>  140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
   141:     let company_destination = ctx.accounts.company_destination.key();
   142:     let founder_destination = ctx.accounts.founder_destination.key();
   143: 
   144:     // No allocation destination may point back to the treasury vault.
   145:     require_keys_neq!(
```

### 84. Destination mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:141`

```text
   136:     let settlement_vault = ctx.accounts.settlement_vault.key();
   137: 
   138:     let reserve_destination = ctx.accounts.reserve_destination.key();
   139:     let buyback_destination = ctx.accounts.buyback_destination.key();
   140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
>  141:     let company_destination = ctx.accounts.company_destination.key();
   142:     let founder_destination = ctx.accounts.founder_destination.key();
   143: 
   144:     // No allocation destination may point back to the treasury vault.
   145:     require_keys_neq!(
   146:         reserve_destination,
```

### 85. Destination mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:142`

```text
   137: 
   138:     let reserve_destination = ctx.accounts.reserve_destination.key();
   139:     let buyback_destination = ctx.accounts.buyback_destination.key();
   140:     let liquidity_destination = ctx.accounts.liquidity_destination.key();
   141:     let company_destination = ctx.accounts.company_destination.key();
>  142:     let founder_destination = ctx.accounts.founder_destination.key();
   143: 
   144:     // No allocation destination may point back to the treasury vault.
   145:     require_keys_neq!(
   146:         reserve_destination,
   147:         settlement_vault,
```

### 86. Destination mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:228`

```text
   223:     let execution_config = &mut ctx.accounts.execution_config;
   224: 
   225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
>  228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
```

### 87. Destination mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:229`

```text
   224: 
   225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
>  229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
```

### 88. Destination mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:230`

```text
   225:     execution_config.protocol_state = ctx.accounts.protocol_state.key();
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
>  230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
   235:     execution_config.bump = ctx.bumps.execution_config;
```

### 89. Destination mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:231`

```text
   226:     execution_config.settlement_mint = ctx.accounts.settlement_mint.key();
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
>  231:     execution_config.company_destination = company_destination;
   232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
   235:     execution_config.bump = ctx.bumps.execution_config;
   236: 
```

### 90. Destination mutation

File: `programs/treasury-router/src/instructions/initialize_execution_config.rs:232`

```text
   227: 
   228:     execution_config.reserve_destination = reserve_destination;
   229:     execution_config.buyback_destination = buyback_destination;
   230:     execution_config.liquidity_destination = liquidity_destination;
   231:     execution_config.company_destination = company_destination;
>  232:     execution_config.founder_destination = founder_destination;
   233: 
   234:     execution_config.version = EXECUTION_CONFIG_VERSION;
   235:     execution_config.bump = ctx.bumps.execution_config;
   236: 
   237:     msg!(
```

### 91. Signer account

File: `programs/treasury-router/src/instructions/initialize_founder.rs:32`

```text
    27:         bump
    28:     )]
    29:     pub founder_state: Account<'info, FounderState>,
    30: 
    31:     #[account(mut)]
>   32:     pub authority: Signer<'info>,
    33: 
    34:     pub system_program: Program<'info, System>,
    35: }
    36: 
    37: pub fn handler(
```

### 92. Authority comparison

File: `programs/treasury-router/src/instructions/initialize_founder.rs:15`

```text
    10: pub struct InitializeFounder<'info> {
    11:     #[account(
    12:         mut,
    13:         seeds = [PROTOCOL_SEED],
    14:         bump = protocol_state.bump,
>   15:         has_one = authority
    16:     )]
    17:     pub protocol_state: Account<'info, ProtocolState>,
    18: 
    19:     #[account(
    20:         init,
```

### 93. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_founder.rs:49`

```text
    44:         !ctx.accounts.protocol_state.paused,
    45:         TreasuryRouterError::ProtocolPaused
    46:     );
    47: 
    48:     require!(
>   49:         ctx.accounts.protocol_state.founder_state == Pubkey::default(),
    50:         TreasuryRouterError::FounderAlreadyInitialized
    51:     );
    52: 
    53:     require!(
    54:         recipient != Pubkey::default(),
```

### 94. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_founder.rs:82`

```text
    77:     founder_state.current_tier = 0;
    78:     founder_state.enabled = true;
    79:     founder_state.bump = ctx.bumps.founder_state;
    80:     founder_state.reserved = [0; 64];
    81: 
>   82:     ctx.accounts.protocol_state.founder_state = founder_state.key();
    83: 
    84:     msg!("FounderState initialized: {}", founder_state.key());
    85:     msg!("Founder recipient: {}", founder_state.recipient);
    86:     msg!("Founder period cap: {}", founder_state.period_cap);
    87:     msg!(
```

### 95. Signer account

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:36`

```text
    31:         bump
    32:     )]
    33:     pub protocol_config: Account<'info, ProtocolConfig>,
    34: 
    35:     #[account(mut)]
>   36:     pub authority: Signer<'info>,
    37: 
    38:     pub system_program: Program<'info, System>,
    39: }
    40: 
    41: pub fn handler(ctx: Context<InitializeProtocolConfig>) -> Result<()> {
```

### 96. Authority comparison

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:19`

```text
    14: pub struct InitializeProtocolConfig<'info> {
    15:     #[account(
    16:         mut,
    17:         seeds = [PROTOCOL_SEED],
    18:         bump = protocol_state.bump,
>   19:         has_one = authority
    20:     )]
    21:     pub protocol_state: Account<'info, ProtocolState>,
    22: 
    23:     #[account(
    24:         init,
```

### 97. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:48`

```text
    43:         !ctx.accounts.protocol_state.paused,
    44:         TreasuryRouterError::ProtocolPaused
    45:     );
    46: 
    47:     require!(
>   48:         ctx.accounts.protocol_state.protocol_config == Pubkey::default(),
    49:         TreasuryRouterError::ProtocolConfigAlreadyInitialized
    50:     );
    51: 
    52:     let total_bps = INITIAL_RESERVE_BPS
    53:         .checked_add(INITIAL_BUYBACK_BURN_BPS)
```

### 98. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:68`

```text
    63: 
    64:     let clock = Clock::get()?;
    65:     let protocol_key = ctx.accounts.protocol_state.key();
    66:     let protocol_config = &mut ctx.accounts.protocol_config;
    67: 
>   68:     protocol_config.version = PROTOCOL_CONFIG_VERSION;
    69:     protocol_config.protocol = protocol_key;
    70: 
    71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
```

### 99. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:69`

```text
    64:     let clock = Clock::get()?;
    65:     let protocol_key = ctx.accounts.protocol_state.key();
    66:     let protocol_config = &mut ctx.accounts.protocol_config;
    67: 
    68:     protocol_config.version = PROTOCOL_CONFIG_VERSION;
>   69:     protocol_config.protocol = protocol_key;
    70: 
    71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
```

### 100. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:71`

```text
    66:     let protocol_config = &mut ctx.accounts.protocol_config;
    67: 
    68:     protocol_config.version = PROTOCOL_CONFIG_VERSION;
    69:     protocol_config.protocol = protocol_key;
    70: 
>   71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
```

### 101. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:72`

```text
    67: 
    68:     protocol_config.version = PROTOCOL_CONFIG_VERSION;
    69:     protocol_config.protocol = protocol_key;
    70: 
    71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
>   72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
    77:     protocol_config.updates_enabled = false;
```

### 102. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:73`

```text
    68:     protocol_config.version = PROTOCOL_CONFIG_VERSION;
    69:     protocol_config.protocol = protocol_key;
    70: 
    71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
>   73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
    77:     protocol_config.updates_enabled = false;
    78:     protocol_config.bump = ctx.bumps.protocol_config;
```

### 103. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:74`

```text
    69:     protocol_config.protocol = protocol_key;
    70: 
    71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
>   74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
    77:     protocol_config.updates_enabled = false;
    78:     protocol_config.bump = ctx.bumps.protocol_config;
    79:     protocol_config.updated_at = clock.unix_timestamp;
```

### 104. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:75`

```text
    70: 
    71:     protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
>   75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
    77:     protocol_config.updates_enabled = false;
    78:     protocol_config.bump = ctx.bumps.protocol_config;
    79:     protocol_config.updated_at = clock.unix_timestamp;
    80:     protocol_config.reserved = [0; 64];
```

### 105. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:77`

```text
    72:     protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
>   77:     protocol_config.updates_enabled = false;
    78:     protocol_config.bump = ctx.bumps.protocol_config;
    79:     protocol_config.updated_at = clock.unix_timestamp;
    80:     protocol_config.reserved = [0; 64];
    81: 
    82:     ctx.accounts.protocol_state.protocol_config = protocol_config.key();
```

### 106. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:78`

```text
    73:     protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
    77:     protocol_config.updates_enabled = false;
>   78:     protocol_config.bump = ctx.bumps.protocol_config;
    79:     protocol_config.updated_at = clock.unix_timestamp;
    80:     protocol_config.reserved = [0; 64];
    81: 
    82:     ctx.accounts.protocol_state.protocol_config = protocol_config.key();
    83: 
```

### 107. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:79`

```text
    74:     protocol_config.company_bps = INITIAL_COMPANY_BPS;
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
    77:     protocol_config.updates_enabled = false;
    78:     protocol_config.bump = ctx.bumps.protocol_config;
>   79:     protocol_config.updated_at = clock.unix_timestamp;
    80:     protocol_config.reserved = [0; 64];
    81: 
    82:     ctx.accounts.protocol_state.protocol_config = protocol_config.key();
    83: 
    84:     msg!("ProtocolConfig initialized: {}", protocol_config.key());
```

### 108. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:80`

```text
    75:     protocol_config.founder_bps = INITIAL_FOUNDER_BPS;
    76: 
    77:     protocol_config.updates_enabled = false;
    78:     protocol_config.bump = ctx.bumps.protocol_config;
    79:     protocol_config.updated_at = clock.unix_timestamp;
>   80:     protocol_config.reserved = [0; 64];
    81: 
    82:     ctx.accounts.protocol_state.protocol_config = protocol_config.key();
    83: 
    84:     msg!("ProtocolConfig initialized: {}", protocol_config.key());
    85:     msg!(
```

### 109. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_protocol_config.rs:82`

```text
    77:     protocol_config.updates_enabled = false;
    78:     protocol_config.bump = ctx.bumps.protocol_config;
    79:     protocol_config.updated_at = clock.unix_timestamp;
    80:     protocol_config.reserved = [0; 64];
    81: 
>   82:     ctx.accounts.protocol_state.protocol_config = protocol_config.key();
    83: 
    84:     msg!("ProtocolConfig initialized: {}", protocol_config.key());
    85:     msg!(
    86:         "Allocations: reserve={} buyback={} liquidity={} company={} founder={}",
    87:         protocol_config.reserve_bps,
```

### 110. Signer account

File: `programs/treasury-router/src/instructions/initialize_treasury.rs:53`

```text
    48:         token::authority = treasury_state
    49:     )]
    50:     pub settlement_vault: Account<'info, TokenAccount>,
    51: 
    52:     #[account(mut)]
>   53:     pub authority: Signer<'info>,
    54: 
    55:     pub token_program: Program<'info, Token>,
    56:     pub system_program: Program<'info, System>,
    57: }
    58: 
```

### 111. Authority comparison

File: `programs/treasury-router/src/instructions/initialize_treasury.rs:19`

```text
    14: pub struct InitializeTreasury<'info> {
    15:     #[account(
    16:         mut,
    17:         seeds = [PROTOCOL_SEED],
    18:         bump = protocol_state.bump,
>   19:         has_one = authority
    20:     )]
    21:     pub protocol_state: Account<'info, ProtocolState>,
    22: 
    23:     #[account(
    24:         init,
```

### 112. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_treasury.rs:66`

```text
    61:         !ctx.accounts.protocol_state.paused,
    62:         TreasuryRouterError::ProtocolPaused
    63:     );
    64: 
    65:     require!(
>   66:         ctx.accounts.protocol_state.treasury_state == Pubkey::default(),
    67:         TreasuryRouterError::TreasuryAlreadyInitialized
    68:     );
    69: 
    70:     let treasury_state = &mut ctx.accounts.treasury_state;
    71: 
```

### 113. Configuration mutation

File: `programs/treasury-router/src/instructions/initialize_treasury.rs:110`

```text
   105:     require!(
   106:         treasury_state.execution_accounting_is_valid(),
   107:         TreasuryRouterError::AccountingInvariantViolation
   108:     );
   109: 
>  110:     ctx.accounts.protocol_state.treasury_state = treasury_state.key();
   111: 
   112:     msg!("TreasuryState initialized: {}", treasury_state.key());
   113:     msg!("Settlement mint: {}", ctx.accounts.settlement_mint.key());
   114:     msg!("Settlement vault: {}", ctx.accounts.settlement_vault.key());
   115:     msg!("Settlement vault authority: {}", treasury_state.key());
```

### 114. Pause mutation

File: `programs/treasury-router/src/instructions/initialize_treasury.rs:101`

```text
    96:     treasury_state.released_founder = 0;
    97: 
    98:     treasury_state.last_processed_at = 0;
    99:     treasury_state.processing_epoch = 0;
   100:     treasury_state.waterfall_stage = INITIAL_WATERFALL_STAGE;
>  101:     treasury_state.buybacks_paused = false;
   102:     treasury_state.bump = ctx.bumps.treasury_state;
   103:     treasury_state.reserved = [0; 24];
   104: 
   105:     require!(
   106:         treasury_state.execution_accounting_is_valid(),
```

### 115. Signer constraint

File: `programs/treasury-router/src/instructions/liquidity.rs:178`

```text
   173:             @ TreasuryRouterError::IntegrityFirewallViolation
   174:     )]
   175:     pub founder_destination: Box<Account<'info, TokenAccount>>,
   176: 
   177:     /// Permissionless transaction caller and fee payer.
>  178:     /// This signer does not need to match ProtocolState.authority.
   179:     pub token_program: Program<'info, Token>,
   180: }
   181: 
   182: /// Releases previously allocated liquidity funds.
   183: ///
```

### 116. Authority assignment

File: `programs/treasury-router/src/instructions/liquidity.rs:268`

```text
   263: 
   264:     let transfer_accounts = TransferChecked {
   265:         from: ctx.accounts.settlement_vault.to_account_info(),
   266:         mint: ctx.accounts.settlement_mint.to_account_info(),
   267:         to: ctx.accounts.liquidity_destination.to_account_info(),
>  268:         authority: ctx.accounts.treasury.to_account_info(),
   269:     };
   270: 
   271:     let cpi_context = CpiContext::new_with_signer(
   272:         ctx.accounts.token_program.key(),
   273:         transfer_accounts,
```

### 117. Configuration mutation

File: `programs/treasury-router/src/instructions/liquidity.rs:30`

```text
    25:     #[account(
    26:         seeds = [PROTOCOL_SEED],
    27:         bump = protocol_state.bump,
    28:         constraint = protocol_state.treasury_state != Pubkey::default()
    29:             @ TreasuryRouterError::TreasuryNotInitialized,
>   30:         constraint = protocol_state.treasury_state == treasury.key()
    31:             @ TreasuryRouterError::InvalidTreasuryProtocol
    32:     )]
    33:     pub protocol_state: Box<Account<'info, ProtocolState>>,
    34: 
    35:     #[account(
```

### 118. Configuration mutation

File: `programs/treasury-router/src/instructions/liquidity.rs:41`

```text
    36:         seeds = [
    37:             PROTOCOL_CONFIG_SEED,
    38:             protocol_state.key().as_ref()
    39:         ],
    40:         bump = protocol_config.bump,
>   41:         constraint = protocol_state.protocol_config == protocol_config.key()
    42:             @ TreasuryRouterError::IntegrityFirewallViolation,
    43:         constraint = protocol_config.protocol == protocol_state.key()
    44:             @ TreasuryRouterError::IntegrityFirewallViolation
    45:     )]
    46:     pub protocol_config: Box<Account<'info, ProtocolConfig>>,
```

### 119. Configuration mutation

File: `programs/treasury-router/src/instructions/liquidity.rs:43`

```text
    38:             protocol_state.key().as_ref()
    39:         ],
    40:         bump = protocol_config.bump,
    41:         constraint = protocol_state.protocol_config == protocol_config.key()
    42:             @ TreasuryRouterError::IntegrityFirewallViolation,
>   43:         constraint = protocol_config.protocol == protocol_state.key()
    44:             @ TreasuryRouterError::IntegrityFirewallViolation
    45:     )]
    46:     pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    47: 
    48:     #[account(
```

### 120. Configuration mutation

File: `programs/treasury-router/src/instructions/liquidity.rs:54`

```text
    49:         seeds = [
    50:             FOUNDER_STATE_SEED,
    51:             protocol_state.key().as_ref()
    52:         ],
    53:         bump = founder_state.bump,
>   54:         constraint = protocol_state.founder_state == founder_state.key()
    55:             @ TreasuryRouterError::IntegrityFirewallViolation,
    56:         constraint = founder_state.protocol == protocol_state.key()
    57:             @ TreasuryRouterError::IntegrityFirewallViolation
    58:     )]
    59:     pub founder_state: Box<Account<'info, FounderState>>,
```

### 121. Configuration mutation

File: `programs/treasury-router/src/instructions/liquidity.rs:67`

```text
    62:         seeds = [
    63:             COMPANY_STATE_SEED,
    64:             protocol_state.key().as_ref()
    65:         ],
    66:         bump = company_state.bump,
>   67:         constraint = protocol_state.company_state == company_state.key()
    68:             @ TreasuryRouterError::IntegrityFirewallViolation,
    69:         constraint = company_state.protocol == protocol_state.key()
    70:             @ TreasuryRouterError::IntegrityFirewallViolation
    71:     )]
    72:     pub company_state: Box<Account<'info, CompanyState>>,
```

### 122. Configuration mutation

File: `programs/treasury-router/src/instructions/liquidity.rs:96`

```text
    91:         seeds = [
    92:             EXECUTION_CONFIG_SEED,
    93:             protocol_state.key().as_ref()
    94:         ],
    95:         bump = execution_config.bump,
>   96:         constraint = execution_config.protocol_state == protocol_state.key()
    97:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    98:         constraint = execution_config.settlement_mint == settlement_mint.key()
    99:             @ TreasuryRouterError::InvalidSettlementMint
   100:     )]
   101:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
```

### 123. Configuration mutation

File: `programs/treasury-router/src/instructions/liquidity.rs:98`

```text
    93:             protocol_state.key().as_ref()
    94:         ],
    95:         bump = execution_config.bump,
    96:         constraint = execution_config.protocol_state == protocol_state.key()
    97:             @ TreasuryRouterError::InvalidTreasuryProtocol,
>   98:         constraint = execution_config.settlement_mint == settlement_mint.key()
    99:             @ TreasuryRouterError::InvalidSettlementMint
   100:     )]
   101:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
   102: 
   103:     #[account(
```

### 124. Configuration mutation

File: `programs/treasury-router/src/instructions/process_fees.rs:23`

```text
    18:     #[account(
    19:         mut,
    20:         seeds = [PROTOCOL_SEED],
    21:         bump = protocol_state.bump,
    22:         has_one = protocol_config,
>   23:         constraint = protocol_state.treasury_state == treasury.key(),
    24:         constraint = protocol_state.founder_state == founder_state.key(),
    25:         constraint = protocol_state.company_state == company_state.key()
    26:     )]
    27:     pub protocol_state: Box<Account<'info, ProtocolState>>,
    28: 
```

### 125. Configuration mutation

File: `programs/treasury-router/src/instructions/process_fees.rs:24`

```text
    19:         mut,
    20:         seeds = [PROTOCOL_SEED],
    21:         bump = protocol_state.bump,
    22:         has_one = protocol_config,
    23:         constraint = protocol_state.treasury_state == treasury.key(),
>   24:         constraint = protocol_state.founder_state == founder_state.key(),
    25:         constraint = protocol_state.company_state == company_state.key()
    26:     )]
    27:     pub protocol_state: Box<Account<'info, ProtocolState>>,
    28: 
    29:     #[account(
```

### 126. Configuration mutation

File: `programs/treasury-router/src/instructions/process_fees.rs:25`

```text
    20:         seeds = [PROTOCOL_SEED],
    21:         bump = protocol_state.bump,
    22:         has_one = protocol_config,
    23:         constraint = protocol_state.treasury_state == treasury.key(),
    24:         constraint = protocol_state.founder_state == founder_state.key(),
>   25:         constraint = protocol_state.company_state == company_state.key()
    26:     )]
    27:     pub protocol_state: Box<Account<'info, ProtocolState>>,
    28: 
    29:     #[account(
    30:         seeds = [
```

### 127. Configuration mutation

File: `programs/treasury-router/src/instructions/process_fees.rs:35`

```text
    30:         seeds = [
    31:             PROTOCOL_CONFIG_SEED,
    32:             protocol_state.key().as_ref()
    33:         ],
    34:         bump = protocol_config.bump,
>   35:         constraint = protocol_config.protocol == protocol_state.key()
    36:     )]
    37:     pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    38: 
    39:     #[account(
    40:         mut,
```

### 128. Configuration mutation

File: `programs/treasury-router/src/instructions/process_fees.rs:523`

```text
   518:         waterfall_evaluation.stage,
   519:     )?;
   520: 
   521:     let dam_evaluation = dam::evaluate_adaptive(waterfall_evaluation.stage, pre_dam_health_score);
   522: 
>  523:     protocol_state.dam_level = dam_evaluation.level.as_u8();
   524: 
   525:     let previous_beaver_score = protocol_state.beaver_score;
   526: 
   527:     let beaver_score_evaluation = beaver_score::evaluate(
   528:         treasury,
```

### 129. Configuration mutation

File: `programs/treasury-router/src/instructions/process_fees.rs:534`

```text
   529:         waterfall_evaluation.reserve_ratio_bps,
   530:         waterfall_evaluation.stage,
   531:         dam_evaluation.level,
   532:     )?;
   533: 
>  534:     protocol_state.beaver_score = beaver_score_evaluation.total_score;
   535: 
   536:     Ok(FeeCycleOutcome {
   537:         pre_allocation_waterfall,
   538:         adaptive_allocation,
   539:         requested_company_amount,
```

### 130. Signer constraint

File: `programs/treasury-router/src/instructions/reserve.rs:178`

```text
   173:             @ TreasuryRouterError::IntegrityFirewallViolation
   174:     )]
   175:     pub founder_destination: Box<Account<'info, TokenAccount>>,
   176: 
   177:     /// Permissionless transaction caller and fee payer.
>  178:     /// This signer does not need to match ProtocolState.authority.
   179:     pub token_program: Program<'info, Token>,
   180: }
   181: 
   182: /// Executes a reserve release previously allocated by fee processing.
   183: ///
```

### 131. Authority assignment

File: `programs/treasury-router/src/instructions/reserve.rs:271`

```text
   266: 
   267:     let transfer_accounts = TransferChecked {
   268:         from: ctx.accounts.settlement_vault.to_account_info(),
   269:         mint: ctx.accounts.settlement_mint.to_account_info(),
   270:         to: ctx.accounts.reserve_destination.to_account_info(),
>  271:         authority: ctx.accounts.treasury.to_account_info(),
   272:     };
   273: 
   274:     let cpi_context = CpiContext::new_with_signer(
   275:         ctx.accounts.token_program.key(),
   276:         transfer_accounts,
```

### 132. Configuration mutation

File: `programs/treasury-router/src/instructions/reserve.rs:30`

```text
    25:     #[account(
    26:         seeds = [PROTOCOL_SEED],
    27:         bump = protocol_state.bump,
    28:         constraint = protocol_state.treasury_state != Pubkey::default()
    29:             @ TreasuryRouterError::TreasuryNotInitialized,
>   30:         constraint = protocol_state.treasury_state == treasury.key()
    31:             @ TreasuryRouterError::InvalidTreasuryProtocol
    32:     )]
    33:     pub protocol_state: Box<Account<'info, ProtocolState>>,
    34: 
    35:     #[account(
```

### 133. Configuration mutation

File: `programs/treasury-router/src/instructions/reserve.rs:41`

```text
    36:         seeds = [
    37:             PROTOCOL_CONFIG_SEED,
    38:             protocol_state.key().as_ref()
    39:         ],
    40:         bump = protocol_config.bump,
>   41:         constraint = protocol_state.protocol_config == protocol_config.key()
    42:             @ TreasuryRouterError::IntegrityFirewallViolation,
    43:         constraint = protocol_config.protocol == protocol_state.key()
    44:             @ TreasuryRouterError::IntegrityFirewallViolation
    45:     )]
    46:     pub protocol_config: Box<Account<'info, ProtocolConfig>>,
```

### 134. Configuration mutation

File: `programs/treasury-router/src/instructions/reserve.rs:43`

```text
    38:             protocol_state.key().as_ref()
    39:         ],
    40:         bump = protocol_config.bump,
    41:         constraint = protocol_state.protocol_config == protocol_config.key()
    42:             @ TreasuryRouterError::IntegrityFirewallViolation,
>   43:         constraint = protocol_config.protocol == protocol_state.key()
    44:             @ TreasuryRouterError::IntegrityFirewallViolation
    45:     )]
    46:     pub protocol_config: Box<Account<'info, ProtocolConfig>>,
    47: 
    48:     #[account(
```

### 135. Configuration mutation

File: `programs/treasury-router/src/instructions/reserve.rs:54`

```text
    49:         seeds = [
    50:             FOUNDER_STATE_SEED,
    51:             protocol_state.key().as_ref()
    52:         ],
    53:         bump = founder_state.bump,
>   54:         constraint = protocol_state.founder_state == founder_state.key()
    55:             @ TreasuryRouterError::IntegrityFirewallViolation,
    56:         constraint = founder_state.protocol == protocol_state.key()
    57:             @ TreasuryRouterError::IntegrityFirewallViolation
    58:     )]
    59:     pub founder_state: Box<Account<'info, FounderState>>,
```

### 136. Configuration mutation

File: `programs/treasury-router/src/instructions/reserve.rs:67`

```text
    62:         seeds = [
    63:             COMPANY_STATE_SEED,
    64:             protocol_state.key().as_ref()
    65:         ],
    66:         bump = company_state.bump,
>   67:         constraint = protocol_state.company_state == company_state.key()
    68:             @ TreasuryRouterError::IntegrityFirewallViolation,
    69:         constraint = company_state.protocol == protocol_state.key()
    70:             @ TreasuryRouterError::IntegrityFirewallViolation
    71:     )]
    72:     pub company_state: Box<Account<'info, CompanyState>>,
```

### 137. Configuration mutation

File: `programs/treasury-router/src/instructions/reserve.rs:96`

```text
    91:         seeds = [
    92:             EXECUTION_CONFIG_SEED,
    93:             protocol_state.key().as_ref()
    94:         ],
    95:         bump = execution_config.bump,
>   96:         constraint = execution_config.protocol_state == protocol_state.key()
    97:             @ TreasuryRouterError::InvalidTreasuryProtocol,
    98:         constraint = execution_config.settlement_mint == settlement_mint.key()
    99:             @ TreasuryRouterError::InvalidSettlementMint
   100:     )]
   101:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
```

### 138. Configuration mutation

File: `programs/treasury-router/src/instructions/reserve.rs:98`

```text
    93:             protocol_state.key().as_ref()
    94:         ],
    95:         bump = execution_config.bump,
    96:         constraint = execution_config.protocol_state == protocol_state.key()
    97:             @ TreasuryRouterError::InvalidTreasuryProtocol,
>   98:         constraint = execution_config.settlement_mint == settlement_mint.key()
    99:             @ TreasuryRouterError::InvalidSettlementMint
   100:     )]
   101:     pub execution_config: Box<Account<'info, ExecutionConfig>>,
   102: 
   103:     #[account(
```

## Automated preliminary verdict

✅ **PRELIMINARY PASS:** Release handlers are permissionless, and all remaining signers are currently classified as initialization-only or depositor-controlled.

A final launch verdict still requires manual review of:

1. Whether initialization can occur more than once.
2. Whether configuration accounts can ever be replaced or mutated.
3. Whether the program remains upgradeable.
4. Whether any pause mechanism creates permanent human discretion.
5. Whether initialization authority is permanently irrelevant after setup.
