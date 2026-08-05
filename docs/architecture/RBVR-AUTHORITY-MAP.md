# RBVR Authority and Autonomy Map

- Generated: 2026-07-29T18:59:09-04:00
- Branch: phase2-autonomy
- Commit: 988930c6d6b17ebe791e2bf1ff3c793420dd185d

## Security objective

RBVR intends to use fixed destinations, deterministic accounting and
one-time initialization, with administrative authority removed only after
deployment, validation and treasury-control verification.

Removing signer checks without replacing them with deterministic execution
conditions could expose treasury releases to arbitrary callers.

## Protocol authority definition

programs/treasury-router/src/events/mod.rs:11:    pub authority: Pubkey,
programs/treasury-router/src/events/mod.rs:33:    pub authority: Pubkey,
programs/treasury-router/src/events/mod.rs:55:    pub authority: Pubkey,
programs/treasury-router/src/events/mod.rs:78:    pub authority: Pubkey,
programs/treasury-router/src/events/mod.rs:105:    pub authority: Pubkey,
programs/treasury-router/src/state/protocol.rs:9:    pub authority: Pubkey,
programs/treasury-router/src/instructions/deposit_settlement.rs:15:        has_one = authority,
programs/treasury-router/src/instructions/deposit_settlement.rs:64:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize_company.rs:15:        has_one = authority
programs/treasury-router/src/instructions/initialize_company.rs:32:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/reserve.rs:28:        has_one = authority,
programs/treasury-router/src/instructions/reserve.rs:178:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize_founder.rs:15:        has_one = authority
programs/treasury-router/src/instructions/initialize_founder.rs:32:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/founder.rs:18:        has_one = authority,
programs/treasury-router/src/instructions/founder.rs:100:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/company.rs:18:        has_one = authority,
programs/treasury-router/src/instructions/company.rs:100:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize_execution_config.rs:18:        has_one = authority,
programs/treasury-router/src/instructions/initialize_execution_config.rs:130:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize.rs:20:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize.rs:30:    protocol_state.authority = ctx.accounts.authority.key();
programs/treasury-router/src/instructions/initialize_treasury.rs:19:        has_one = authority
programs/treasury-router/src/instructions/initialize_treasury.rs:53:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize_protocol_config.rs:19:        has_one = authority
programs/treasury-router/src/instructions/initialize_protocol_config.rs:36:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/liquidity.rs:28:        has_one = authority,
programs/treasury-router/src/instructions/liquidity.rs:178:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/buyback.rs:28:        has_one = authority,
programs/treasury-router/src/instructions/buyback.rs:178:    pub authority: Signer<'info>,

## Public program entrypoints

19:    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
23:    pub fn initialize_protocol_config(ctx: Context<InitializeProtocolConfig>) -> Result<()> {
27:    pub fn initialize_founder(
36:    pub fn initialize_company(
45:    pub fn initialize_treasury(ctx: Context<InitializeTreasury>) -> Result<()> {
49:    pub fn initialize_execution_config(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
53:    pub fn deposit_settlement(ctx: Context<DepositSettlement>, amount: u64) -> Result<()> {
57:    pub fn process_fees(ctx: Context<ProcessFees>) -> Result<()> {
61:    pub fn authorize_reserve_execution(
68:    pub fn authorize_buyback_execution(
75:    pub fn authorize_liquidity_execution(
82:    pub fn authorize_founder_execution(
89:    pub fn authorize_company_execution(

## Authority-gated instructions

The following files contain both an authority signer and an Anchor
`has_one = authority` relationship.


### buyback.rs

```text
24:pub struct AuthorizeBuybackExecution<'info> {
28:        has_one = authority,
178:    pub authority: Signer<'info>,
192:pub fn handler(ctx: Context<AuthorizeBuybackExecution>, amount: u64) -> Result<()> {
```

### company.rs

```text
14:pub struct AuthorizeCompanyExecution<'info> {
18:        has_one = authority,
100:    pub authority: Signer<'info>,
112:pub fn handler(ctx: Context<AuthorizeCompanyExecution>, amount: u64) -> Result<()> {
```

### deposit_settlement.rs

```text
11:pub struct DepositSettlement<'info> {
15:        has_one = authority,
64:    pub authority: Signer<'info>,
69:pub fn handler(ctx: Context<DepositSettlement>, amount: u64) -> Result<()> {
```

### founder.rs

```text
14:pub struct AuthorizeFounderExecution<'info> {
18:        has_one = authority,
100:    pub authority: Signer<'info>,
112:pub fn handler(ctx: Context<AuthorizeFounderExecution>, amount: u64) -> Result<()> {
```

### initialize_company.rs

```text
10:pub struct InitializeCompany<'info> {
15:        has_one = authority
32:    pub authority: Signer<'info>,
37:pub fn handler(
```

### initialize_execution_config.rs

```text
14:pub struct InitializeExecutionConfig<'info> {
18:        has_one = authority,
130:    pub authority: Signer<'info>,
135:pub fn handler(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
```

### initialize_founder.rs

```text
10:pub struct InitializeFounder<'info> {
15:        has_one = authority
32:    pub authority: Signer<'info>,
37:pub fn handler(
```

### initialize_protocol_config.rs

```text
14:pub struct InitializeProtocolConfig<'info> {
19:        has_one = authority
36:    pub authority: Signer<'info>,
41:pub fn handler(ctx: Context<InitializeProtocolConfig>) -> Result<()> {
```

### initialize_treasury.rs

```text
14:pub struct InitializeTreasury<'info> {
19:        has_one = authority
53:    pub authority: Signer<'info>,
59:pub fn handler(ctx: Context<InitializeTreasury>) -> Result<()> {
```

### liquidity.rs

```text
24:pub struct AuthorizeLiquidityExecution<'info> {
28:        has_one = authority,
178:    pub authority: Signer<'info>,
189:pub fn handler(ctx: Context<AuthorizeLiquidityExecution>, amount: u64) -> Result<()> {
```

### reserve.rs

```text
24:pub struct AuthorizeReserveExecution<'info> {
28:        has_one = authority,
178:    pub authority: Signer<'info>,
192:pub fn handler(ctx: Context<AuthorizeReserveExecution>, amount: u64) -> Result<()> {
```

## Treasury-signing CPI paths

These instructions use the treasury PDA as the SPL-token transfer authority.
The caller authority and the treasury PDA signer are separate controls:
the user signs the instruction, while the program signs the token CPI using
treasury PDA seeds.

programs/treasury-router/src/instructions/deposit_settlement.rs:86:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/reserve.rs:263:    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];
programs/treasury-router/src/instructions/reserve.rs:265:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/reserve.rs:274:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/reserve.rs:280:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/founder.rs:123:    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];
programs/treasury-router/src/instructions/founder.rs:125:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/founder.rs:134:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/founder.rs:140:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/company.rs:123:    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];
programs/treasury-router/src/instructions/company.rs:125:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/company.rs:134:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/company.rs:140:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/liquidity.rs:260:    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];
programs/treasury-router/src/instructions/liquidity.rs:262:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/liquidity.rs:271:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/liquidity.rs:277:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/buyback.rs:263:    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];
programs/treasury-router/src/instructions/buyback.rs:265:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/buyback.rs:274:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/buyback.rs:280:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;

## Initialization paths

programs/treasury-router/src/instructions/initialize_company.rs:10:pub struct InitializeCompany<'info> {
programs/treasury-router/src/instructions/initialize_company.rs:15:        has_one = authority
programs/treasury-router/src/instructions/initialize_company.rs:20:        init,
programs/treasury-router/src/instructions/initialize_company.rs:32:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize_company.rs:37:pub fn handler(
programs/treasury-router/src/instructions/initialize_founder.rs:10:pub struct InitializeFounder<'info> {
programs/treasury-router/src/instructions/initialize_founder.rs:15:        has_one = authority
programs/treasury-router/src/instructions/initialize_founder.rs:20:        init,
programs/treasury-router/src/instructions/initialize_founder.rs:32:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize_founder.rs:37:pub fn handler(
programs/treasury-router/src/instructions/initialize_execution_config.rs:14:pub struct InitializeExecutionConfig<'info> {
programs/treasury-router/src/instructions/initialize_execution_config.rs:18:        has_one = authority,
programs/treasury-router/src/instructions/initialize_execution_config.rs:118:        init,
programs/treasury-router/src/instructions/initialize_execution_config.rs:130:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize_execution_config.rs:135:pub fn handler(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
programs/treasury-router/src/instructions/initialize.rs:9:pub struct Initialize<'info> {
programs/treasury-router/src/instructions/initialize.rs:11:        init,
programs/treasury-router/src/instructions/initialize.rs:20:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize.rs:25:pub fn handler(ctx: Context<Initialize>) -> Result<()> {
programs/treasury-router/src/instructions/initialize_treasury.rs:14:pub struct InitializeTreasury<'info> {
programs/treasury-router/src/instructions/initialize_treasury.rs:19:        has_one = authority
programs/treasury-router/src/instructions/initialize_treasury.rs:24:        init,
programs/treasury-router/src/instructions/initialize_treasury.rs:40:        init,
programs/treasury-router/src/instructions/initialize_treasury.rs:53:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize_treasury.rs:59:pub fn handler(ctx: Context<InitializeTreasury>) -> Result<()> {
programs/treasury-router/src/instructions/initialize_protocol_config.rs:14:pub struct InitializeProtocolConfig<'info> {
programs/treasury-router/src/instructions/initialize_protocol_config.rs:19:        has_one = authority
programs/treasury-router/src/instructions/initialize_protocol_config.rs:24:        init,
programs/treasury-router/src/instructions/initialize_protocol_config.rs:36:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize_protocol_config.rs:41:pub fn handler(ctx: Context<InitializeProtocolConfig>) -> Result<()> {

## Mutable configuration and authority-changing paths

programs/treasury-router/src/engines/sentinel.rs:573:    fn configuration_updates_enabled_fails_closed() {

## Preliminary classification

### Initialization-only authority

Likely candidates:

- initialize
- initialize_protocol_config
- initialize_treasury
- initialize_founder
- initialize_company
- initialize_execution_config

These should remain authority-controlled during deployment. They should
become unusable after their PDA accounts have been initialized.

### Deposit authority

`deposit_settlement` currently links the depositor to the protocol authority.
That means ordinary third parties cannot fund the settlement vault directly.

This may be intentional during testing, but it is not necessary for a fully
autonomous protocol if deposits should be open to anyone. Its source account
already has to be owned by the transaction signer.

### Fee processing

`process_fees` does not appear in the authority-gated list from the previous
automated audit. This is consistent with deterministic permissionless
processing, provided its state and account constraints fully prevent caller
discretion.

### Release authority

The following release instructions currently require the protocol authority:

- reserve
- buyback
- liquidity
- company
- founder

These checks cannot simply be deleted. A permissionless caller must not be
able to select arbitrary release timing or amounts.

Before converting these instructions to permissionless execution, RBVR needs
one of these deterministic models:

1. Anyone may trigger a release, but the program calculates the exact amount.
2. Anyone may trigger a release only after a fixed interval and only up to an
   on-chain deterministic amount.
3. A dedicated automation signer triggers releases, with no power to change
   destinations, allocations, caps or accounting rules.
4. Release authority remains temporarily controlled until the complete
   automation and fail-closed rules are deployed and tested.

## Recommended current decision

Keep release authority temporarily.

Do not revoke or replace it until:

- release amounts are computed entirely on-chain;
- time and period conditions are enforced on-chain;
- all destinations are immutable;
- configuration updates are permanently disabled;
- the program upgrade authority has an explicit final disposition;
- permissionless callers cannot accelerate founder or company compensation;
- repeat calls cannot release the same pending balance twice;
- the complete autonomous path has integration and fuzz coverage.

