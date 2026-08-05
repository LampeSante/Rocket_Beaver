# RBVR Global Invariant Engineering Map

Generated: **2026-07-30 20:18:05 UTC**

This report captures the exact APIs and test helpers required to build
the deterministic global state-machine invariant suite.

## `programs/treasury-router/src/engines/release.rs`

- Structs found: **1**
- Enums found: **0**
- Functions found: **17**
- Relevant functions selected: **17**

### Relevant structs

#### `ReleaseTransition` — line 9

```rust
pub struct ReleaseTransition {
    pub bucket: ReleaseBucket,
    pub amount: u64,
    pub previous_pending_balance: u64,
    pub remaining_pending_balance: u64,
    pub previous_released_balance: u64,
    pub total_released_balance: u64,
}
```

### Relevant functions and tests

#### `process_release` — line 29

```rust
pub fn process_release(
    treasury: &mut TreasuryState,
    bucket: ReleaseBucket,
    amount: u64,
) -> Result<ReleaseTransition> {
    require!(amount > 0, TreasuryRouterError::InvalidReleaseAmount);

    // Reject any corrupted treasury before calculating or mutating balances.
    require!(
        treasury.execution_accounting_is_valid(),
        TreasuryRouterError::AccountingInvariantViolation
    );

    let (previous_pending_balance, previous_released_balance) =
        balances_for_bucket(treasury, bucket);

    require!(
        amount <= previous_pending_balance,
        TreasuryRouterError::InsufficientPendingBalance
    );

    let remaining_pending_balance = previous_pending_balance
        .checked_sub(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let total_released_balance = previous_released_balance
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    set_balances_for_bucket(
        treasury,
        bucket,
        remaining_pending_balance,
        total_released_balance,
    );

    require!(
        bucket_accounting_is_valid(treasury, bucket),
        TreasuryRouterError::AccountingInvariantViolation
    );

    require!(
        treasury.execution_accounting_is_valid(),
        TreasuryRouterError::AccountingInvariantViolation
    );

    Ok(ReleaseTransition {
        bucket,
        amount,
        previous_pending_balance,
        remaining_pending_balance,
        previous_released_balance,
        total_released_balance,
    })
}
```

#### `balances_for_bucket` — line 86

```rust
pub fn balances_for_bucket(treasury: &TreasuryState, bucket: ReleaseBucket) -> (u64, u64) {
    match bucket {
        ReleaseBucket::Reserve => (treasury.pending_reserve, treasury.released_reserve),
        ReleaseBucket::BuybackBurn => (
            treasury.pending_buyback_burn,
            treasury.released_buyback_burn,
        ),
        ReleaseBucket::Liquidity => (treasury.pending_liquidity, treasury.released_liquidity),
        ReleaseBucket::Company => (treasury.pending_company, treasury.released_company),
        ReleaseBucket::Founder => (treasury.pending_founder, treasury.released_founder),
    }
}
```

#### `set_balances_for_bucket` — line 99

```rust
fn set_balances_for_bucket(
    treasury: &mut TreasuryState,
    bucket: ReleaseBucket,
    pending: u64,
    released: u64,
) {
    match bucket {
        ReleaseBucket::Reserve => {
            treasury.pending_reserve = pending;
            treasury.released_reserve = released;
        }
        ReleaseBucket::BuybackBurn => {
            treasury.pending_buyback_burn = pending;
            treasury.released_buyback_burn = released;
        }
        ReleaseBucket::Liquidity => {
            treasury.pending_liquidity = pending;
            treasury.released_liquidity = released;
        }
        ReleaseBucket::Company => {
            treasury.pending_company = pending;
            treasury.released_company = released;
        }
        ReleaseBucket::Founder => {
            treasury.pending_founder = pending;
            treasury.released_founder = released;
        }
    }
}
```

#### `bucket_accounting_is_valid` — line 129

```rust
fn bucket_accounting_is_valid(treasury: &TreasuryState, bucket: ReleaseBucket) -> bool {
    match bucket {
        ReleaseBucket::Reserve => treasury.reserve_accounting_is_valid(),
        ReleaseBucket::BuybackBurn => treasury.buyback_accounting_is_valid(),
        ReleaseBucket::Liquidity => treasury.liquidity_accounting_is_valid(),
        ReleaseBucket::Company => treasury.company_accounting_is_valid(),
        ReleaseBucket::Founder => treasury.founder_accounting_is_valid(),
    }
}
```

#### `valid_treasury` — line 144

```rust
    fn valid_treasury() -> TreasuryState {
        TreasuryState {
            version: 1,
            protocol: Pubkey::new_unique(),
            settlement_mint: Pubkey::new_unique(),
            settlement_vault: Pubkey::new_unique(),

            total_fees_received: 1_500,
            total_fees_allocated: 1_500,

            pending_reserve: 300,
            pending_buyback_burn: 250,
            pending_liquidity: 350,
            pending_company: 400,
            pending_founder: 200,

            lifetime_reserve: 350,
            lifetime_buyback_burn: 300,
            lifetime_liquidity: 400,
            lifetime_company: 500,
            lifetime_founder: 250,

            released_reserve: 50,
            released_buyback_burn: 50,
            released_liquidity: 50,
            released_company: 100,
            released_founder: 50,

            last_processed_at: 1,
            processing_epoch: 1,
            waterfall_stage: WaterfallStage::Normal.as_u8(),
            buybacks_paused: false,
            bump: 255,
            reserved: [0; 24],
        }
    }
```

#### `assert_release` — line 181

```rust
    fn assert_release(bucket: ReleaseBucket, expected_pending: u64, expected_released: u64) {
        let mut treasury = valid_treasury();

        let before_reserve = (treasury.pending_reserve, treasury.released_reserve);
        let before_buyback = (
            treasury.pending_buyback_burn,
            treasury.released_buyback_burn,
        );
        let before_liquidity = (treasury.pending_liquidity, treasury.released_liquidity);
        let before_company = (treasury.pending_company, treasury.released_company);
        let before_founder = (treasury.pending_founder, treasury.released_founder);

        let transition = process_release(&mut treasury, bucket, 25).unwrap();

        assert_eq!(transition.bucket, bucket);
        assert_eq!(transition.amount, 25);
        assert_eq!(transition.remaining_pending_balance, expected_pending);
        assert_eq!(transition.total_released_balance, expected_released);
        assert!(treasury.execution_accounting_is_valid());

        if bucket != ReleaseBucket::Reserve {
            assert_eq!(
                (treasury.pending_reserve, treasury.released_reserve),
                before_reserve
            );
        }

        if bucket != ReleaseBucket::BuybackBurn {
            assert_eq!(
                (
                    treasury.pending_buyback_burn,
                    treasury.released_buyback_burn
                ),
                before_buyback
            );
        }

        if bucket != ReleaseBucket::Liquidity {
            assert_eq!(
                (treasury.pending_liquidity, treasury.released_liquidity),
                before_liquidity
            );
        }

        if bucket != ReleaseBucket::Company {
            assert_eq!(
                (treasury.pending_company, treasury.released_company),
                before_company
            );
        }

        if bucket != ReleaseBucket::Founder {
            assert_eq!(
                (treasury.pending_founder, treasury.released_founder),
                before_founder
            );
        }
    }
```

#### `processes_reserve_release` — line 241

```rust
    fn processes_reserve_release() {
        assert_release(ReleaseBucket::Reserve, 275, 75);
    }
```

#### `processes_buyback_release` — line 246

```rust
    fn processes_buyback_release() {
        assert_release(ReleaseBucket::BuybackBurn, 225, 75);
    }
```

#### `processes_liquidity_release` — line 251

```rust
    fn processes_liquidity_release() {
        assert_release(ReleaseBucket::Liquidity, 325, 75);
    }
```

#### `processes_company_release` — line 256

```rust
    fn processes_company_release() {
        assert_release(ReleaseBucket::Company, 375, 125);
    }
```

#### `processes_founder_release` — line 261

```rust
    fn processes_founder_release() {
        assert_release(ReleaseBucket::Founder, 175, 75);
    }
```

#### `rejects_zero_release_without_mutating_treasury` — line 266

```rust
    fn rejects_zero_release_without_mutating_treasury() {
        let mut treasury = valid_treasury();
        let before = balances_for_bucket(&treasury, ReleaseBucket::Reserve);

        assert!(process_release(&mut treasury, ReleaseBucket::Reserve, 0).is_err());
        assert_eq!(
            balances_for_bucket(&treasury, ReleaseBucket::Reserve),
            before
        );
    }
```

#### `rejects_release_above_pending_without_mutating_treasury` — line 278

```rust
    fn rejects_release_above_pending_without_mutating_treasury() {
        let mut treasury = valid_treasury();
        let before = balances_for_bucket(&treasury, ReleaseBucket::Founder);

        assert!(process_release(&mut treasury, ReleaseBucket::Founder, 201).is_err());

        assert_eq!(
            balances_for_bucket(&treasury, ReleaseBucket::Founder),
            before
        );
    }
```

#### `rejects_corrupted_selected_bucket_before_mutation` — line 291

```rust
    fn rejects_corrupted_selected_bucket_before_mutation() {
        let mut treasury = valid_treasury();
        treasury.lifetime_company += 1;

        let before = balances_for_bucket(&treasury, ReleaseBucket::Company);

        assert!(process_release(&mut treasury, ReleaseBucket::Company, 25).is_err());

        assert_eq!(
            balances_for_bucket(&treasury, ReleaseBucket::Company),
            before
        );
    }
```

#### `rejects_corrupted_unrelated_bucket_before_mutation` — line 306

```rust
    fn rejects_corrupted_unrelated_bucket_before_mutation() {
        let mut treasury = valid_treasury();
        treasury.lifetime_founder += 1;

        let before = balances_for_bucket(&treasury, ReleaseBucket::Reserve);

        assert!(process_release(&mut treasury, ReleaseBucket::Reserve, 25).is_err());

        assert_eq!(
            balances_for_bucket(&treasury, ReleaseBucket::Reserve),
            before
        );
    }
```

#### `rejects_released_balance_overflow_without_mutation` — line 321

```rust
    fn rejects_released_balance_overflow_without_mutation() {
        let mut treasury = valid_treasury();

        treasury.pending_reserve = 1;
        treasury.released_reserve = u64::MAX;
        treasury.lifetime_reserve = u64::MAX;

        let before = balances_for_bucket(&treasury, ReleaseBucket::Reserve);

        // The pre-existing state is itself invalid because pending + released
        // overflows, so the transition must fail closed before mutation.
        assert!(process_release(&mut treasury, ReleaseBucket::Reserve, 1).is_err());

        assert_eq!(
            balances_for_bucket(&treasury, ReleaseBucket::Reserve),
            before
        );
    }
```

#### `full_pending_balance_can_be_released` — line 341

```rust
    fn full_pending_balance_can_be_released() {
        let mut treasury = valid_treasury();
        let amount = treasury.pending_buyback_burn;

        let transition =
            process_release(&mut treasury, ReleaseBucket::BuybackBurn, amount).unwrap();

        assert_eq!(transition.remaining_pending_balance, 0);
        assert_eq!(
            transition.total_released_balance,
            treasury.lifetime_buyback_burn
        );
        assert!(treasury.buyback_accounting_is_valid());
        assert!(treasury.execution_accounting_is_valid());
    }
```

## `programs/treasury-router/src/engines/execution_guard.rs`

- Structs found: **1**
- Enums found: **1**
- Functions found: **30**
- Relevant functions selected: **26**

### Relevant structs

#### `ExecutionAuthorization` — line 24

```rust
pub struct ExecutionAuthorization {
    pub bucket: ReleaseBucket,
    pub requested_amount: u64,
    pub pending_balance: u64,
    pub maximum_release: u64,
    pub waterfall_stage: WaterfallStage,
    pub dam_level: DamLevel,
    pub release_bps: u16,
    pub reserve_ratio_bps: u16,
}
```

### Relevant enums

#### `ReleaseBucket` — line 15

```rust
pub enum ReleaseBucket {
    Reserve,
    BuybackBurn,
    Liquidity,
    Company,
    Founder,
}
```

### Relevant functions and tests

#### `authorize_release` — line 40

```rust
pub fn authorize_release(
    protocol: &ProtocolState,
    treasury: &TreasuryState,
    bucket: ReleaseBucket,
    requested_amount: u64,
) -> Result<ExecutionAuthorization> {
    require!(!protocol.paused, TreasuryRouterError::ProtocolPaused);

    require!(
        requested_amount > 0,
        TreasuryRouterError::InvalidReleaseAmount
    );

    validate_accounting_integrity(treasury)?;

    require!(
        treasury.execution_accounting_is_valid(),
        TreasuryRouterError::AccountingInvariantViolation
    );

    if bucket == ReleaseBucket::BuybackBurn {
        require!(
            !treasury.buybacks_paused,
            TreasuryRouterError::BuybacksPaused
        );
    }

    let waterfall_evaluation = waterfall::evaluate(treasury)?;

    let pre_dam_health_score = beaver_score::pre_dam_health_score(
        treasury,
        waterfall_evaluation.reserve_ratio_bps,
        waterfall_evaluation.stage,
    )?;

    let dam_evaluation = dam::evaluate_adaptive(waterfall_evaluation.stage, pre_dam_health_score);

    require!(
        dam_evaluation.release_bps <= 10_000,
        TreasuryRouterError::InvalidDamReleaseRate
    );

    require!(
        dam_evaluation.level != DamLevel::Filling,
        TreasuryRouterError::DamClosed
    );

    let pending_balance = pending_balance(treasury, bucket);

    require!(
        requested_amount <= pending_balance,
        TreasuryRouterError::InsufficientPendingBalance
    );

    let maximum_release = calculate_maximum_release(pending_balance, dam_evaluation.release_bps)?;

    require!(
        requested_amount <= maximum_release,
        TreasuryRouterError::ReleaseLimitExceeded
    );

    Ok(ExecutionAuthorization {
        bucket,
        requested_amount,
        pending_balance,
        maximum_release,
        waterfall_stage: waterfall_evaluation.stage,
        dam_level: dam_evaluation.level,
        release_bps: dam_evaluation.release_bps,
        reserve_ratio_bps: waterfall_evaluation.reserve_ratio_bps,
    })
}
```

#### `authorize_autonomous_release` — line 119

```rust
pub fn authorize_autonomous_release(
    protocol: &ProtocolState,
    treasury: &TreasuryState,
    bucket: ReleaseBucket,
) -> Result<ExecutionAuthorization> {
    require!(!protocol.paused, TreasuryRouterError::ProtocolPaused);

    validate_accounting_integrity(treasury)?;

    require!(
        treasury.execution_accounting_is_valid(),
        TreasuryRouterError::AccountingInvariantViolation
    );

    if bucket == ReleaseBucket::BuybackBurn {
        require!(
            !treasury.buybacks_paused,
            TreasuryRouterError::BuybacksPaused
        );
    }

    let waterfall_evaluation = waterfall::evaluate(treasury)?;

    let pre_dam_health_score = beaver_score::pre_dam_health_score(
        treasury,
        waterfall_evaluation.reserve_ratio_bps,
        waterfall_evaluation.stage,
    )?;

    let dam_evaluation = dam::evaluate_adaptive(waterfall_evaluation.stage, pre_dam_health_score);

    require!(
        dam_evaluation.release_bps <= 10_000,
        TreasuryRouterError::InvalidDamReleaseRate
    );

    require!(
        dam_evaluation.level != DamLevel::Filling,
        TreasuryRouterError::DamClosed
    );

    let pending_balance = pending_balance(treasury, bucket);

    require!(
        pending_balance > 0,
        TreasuryRouterError::InsufficientPendingBalance
    );

    let maximum_release = calculate_maximum_release(pending_balance, dam_evaluation.release_bps)?;

    require!(
        maximum_release > 0,
        TreasuryRouterError::ReleaseLimitExceeded
    );

    Ok(ExecutionAuthorization {
        bucket,
        requested_amount: maximum_release,
        pending_balance,
        maximum_release,
        waterfall_stage: waterfall_evaluation.stage,
        dam_level: dam_evaluation.level,
        release_bps: dam_evaluation.release_bps,
        reserve_ratio_bps: waterfall_evaluation.reserve_ratio_bps,
    })
}
```

#### `validate_accounting_integrity` — line 190

```rust
fn validate_accounting_integrity(treasury: &TreasuryState) -> Result<()> {
    require!(
        treasury.total_fees_allocated <= treasury.total_fees_received,
        TreasuryRouterError::AccountingInvariantViolation
    );

    require!(
        treasury.total_fees_allocated == treasury.total_fees_received,
        TreasuryRouterError::AccountingNotSettled
    );

    Ok(())
}
```

#### `pending_balance` — line 205

```rust
pub fn pending_balance(treasury: &TreasuryState, bucket: ReleaseBucket) -> u64 {
    match bucket {
        ReleaseBucket::Reserve => treasury.pending_reserve,
        ReleaseBucket::BuybackBurn => treasury.pending_buyback_burn,
        ReleaseBucket::Liquidity => treasury.pending_liquidity,
        ReleaseBucket::Company => treasury.pending_company,
        ReleaseBucket::Founder => treasury.pending_founder,
    }
}
```

#### `calculate_maximum_release` — line 219

```rust
pub fn calculate_maximum_release(pending_balance: u64, release_bps: u16) -> Result<u64> {
    require!(
        release_bps <= 10_000,
        TreasuryRouterError::InvalidDamReleaseRate
    );

    let numerator = u128::from(pending_balance)
        .checked_mul(u128::from(release_bps))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let maximum_release = numerator
        .checked_div(10_000)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    u64::try_from(maximum_release).map_err(|_| TreasuryRouterError::ArithmeticOverflow.into())
}
```

#### `valid_protocol` — line 240

```rust
    fn valid_protocol() -> ProtocolState {
        ProtocolState {
            version: 1,
            authority: Pubkey::new_unique(),
            protocol_config: Pubkey::new_unique(),
            treasury_state: Pubkey::new_unique(),
            reserve_state: Pubkey::new_unique(),
            liquidity_state: Pubkey::new_unique(),
            founder_state: Pubkey::new_unique(),
            company_state: Pubkey::new_unique(),
            buyback_state: Pubkey::new_unique(),
            beaver_score: 10_000,
            dam_level: DamLevel::Overflow.as_u8(),
            paused: false,
            bump: 255,
            initialized_at: 1,
            reserved: [0; 64],
        }
    }
```

#### `valid_treasury` — line 274

```rust
    fn valid_treasury() -> TreasuryState {
        TreasuryState {
            version: 1,
            protocol: Pubkey::new_unique(),
            settlement_mint: Pubkey::new_unique(),
            settlement_vault: Pubkey::new_unique(),

            total_fees_received: 1_400,
            total_fees_allocated: 1_400,

            pending_reserve: 1_000,
            pending_buyback_burn: 100,
            pending_liquidity: 100,
            pending_company: 100,
            pending_founder: 100,

            lifetime_reserve: 1_000,
            lifetime_buyback_burn: 100,
            lifetime_liquidity: 100,
            lifetime_company: 100,
            lifetime_founder: 100,

            released_reserve: 0,
            released_buyback_burn: 0,
            released_liquidity: 0,
            released_company: 0,
            released_founder: 0,

            last_processed_at: 1,
            processing_epoch: 1,
            waterfall_stage: WaterfallStage::Normal.as_u8(),
            buybacks_paused: false,
            bump: 254,
            reserved: [0; 24],
        }
    }
```

#### `assert_anchor_error` — line 311

```rust
    fn assert_anchor_error(result: Result<ExecutionAuthorization>, expected_error_name: &str) {
        let error = match result {
            Ok(_) => {
                panic!("Expected Anchor error {expected_error_name}, but authorization succeeded.")
            }
            Err(error) => error,
        };

        match error {
            anchor_lang::error::Error::AnchorError(anchor_error) => {
                assert_eq!(
                    anchor_error.error_name, expected_error_name,
                    "Unexpected Anchor error: {anchor_error:?}"
                );
            }
            other => panic!("Expected AnchorError {expected_error_name}, received {other:?}."),
        }
    }
```

#### `full_dam_allows_full_pending_balance` — line 331

```rust
    fn full_dam_allows_full_pending_balance() {
        assert_eq!(calculate_maximum_release(1_000, 10_000).unwrap(), 1_000);
    }
```

#### `release_calculation_rounds_down` — line 356

```rust
    fn release_calculation_rounds_down() {
        assert_eq!(calculate_maximum_release(3, 2_500).unwrap(), 0);

        assert_eq!(calculate_maximum_release(7, 5_000).unwrap(), 3);
    }
```

#### `rejects_invalid_dam_release_rate` — line 363

```rust
    fn rejects_invalid_dam_release_rate() {
        assert_anchor_error(
            calculate_maximum_release(1_000, 10_001).map(|maximum_release| {
                ExecutionAuthorization {
                    bucket: ReleaseBucket::Reserve,
                    requested_amount: 1,
                    pending_balance: 1_000,
                    maximum_release,
                    waterfall_stage: WaterfallStage::Normal,
                    dam_level: DamLevel::Overflow,
                    release_bps: 10_001,
                    reserve_ratio_bps: 10_000,
                }
            }),
            "InvalidDamReleaseRate",
        );
    }
```

#### `authorizes_valid_reserve_release` — line 382

```rust
    fn authorizes_valid_reserve_release() {
        let protocol = valid_protocol();
        let treasury = valid_treasury();

        let authorization =
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 750).unwrap();

        assert_eq!(authorization.bucket, ReleaseBucket::Reserve);
        assert_eq!(authorization.requested_amount, 750);
        assert_eq!(authorization.pending_balance, 1_000);
        assert_eq!(authorization.maximum_release, 1_000);
        assert_eq!(authorization.waterfall_stage, WaterfallStage::Normal);
        assert_eq!(authorization.dam_level, DamLevel::Overflow);
        assert_eq!(authorization.release_bps, 10_000);
        assert_eq!(authorization.reserve_ratio_bps, 7_142);
    }
```

#### `authorizes_valid_buyback_release` — line 400

```rust
    fn authorizes_valid_buyback_release() {
        let protocol = valid_protocol();
        let treasury = valid_treasury();

        let authorization =
            authorize_release(&protocol, &treasury, ReleaseBucket::BuybackBurn, 100).unwrap();

        assert_eq!(authorization.bucket, ReleaseBucket::BuybackBurn);
        assert_eq!(authorization.requested_amount, 100);
        assert_eq!(authorization.pending_balance, 100);
        assert_eq!(authorization.maximum_release, 100);
    }
```

#### `rejects_release_when_protocol_is_paused` — line 414

```rust
    fn rejects_release_when_protocol_is_paused() {
        let mut protocol = valid_protocol();
        let treasury = valid_treasury();

        protocol.paused = true;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
            "ProtocolPaused",
        );
    }
```

#### `rejects_zero_release_amount` — line 427

```rust
    fn rejects_zero_release_amount() {
        let protocol = valid_protocol();
        let treasury = valid_treasury();

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 0),
            "InvalidReleaseAmount",
        );
    }
```

#### `rejects_buyback_when_buybacks_are_paused` — line 438

```rust
    fn rejects_buyback_when_buybacks_are_paused() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.buybacks_paused = true;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::BuybackBurn, 1),
            "BuybacksPaused",
        );
    }
```

#### `buyback_pause_does_not_block_other_buckets` — line 451

```rust
    fn buyback_pause_does_not_block_other_buckets() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.buybacks_paused = true;

        let authorization =
            authorize_release(&protocol, &treasury, ReleaseBucket::Liquidity, 50).unwrap();

        assert_eq!(authorization.bucket, ReleaseBucket::Liquidity);
        assert_eq!(authorization.requested_amount, 50);
    }
```

#### `rejects_release_exceeding_pending_balance` — line 465

```rust
    fn rejects_release_exceeding_pending_balance() {
        let protocol = valid_protocol();
        let treasury = valid_treasury();

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Company, 101),
            "InsufficientPendingBalance",
        );
    }
```

#### `rejects_release_when_dam_is_closed` — line 476

```rust
    fn rejects_release_when_dam_is_closed() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.pending_reserve = 40;
        treasury.pending_buyback_burn = 240;
        treasury.pending_liquidity = 240;
        treasury.pending_company = 240;
        treasury.pending_founder = 240;

        treasury.lifetime_reserve = 40;
        treasury.lifetime_buyback_burn = 240;
        treasury.lifetime_liquidity = 240;
        treasury.lifetime_company = 240;
        treasury.lifetime_founder = 240;

        treasury.total_fees_received = 1_000;
        treasury.total_fees_allocated = 1_000;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Company, 1),
            "DamClosed",
        );
    }
```

#### `rejects_release_exceeding_current_dam_limit` — line 502

```rust
    fn rejects_release_exceeding_current_dam_limit() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        // 200 / 1,000 = 20%, which is Caution.
        // Caution maps to a 75% release limit.
        treasury.pending_reserve = 200;
        treasury.pending_buyback_burn = 0;
        treasury.pending_liquidity = 0;
        treasury.pending_company = 800;
        treasury.pending_founder = 0;

        treasury.lifetime_reserve = 200;
        treasury.lifetime_buyback_burn = 0;
        treasury.lifetime_liquidity = 0;
        treasury.lifetime_company = 800;
        treasury.lifetime_founder = 0;

        treasury.total_fees_received = 1_000;
        treasury.total_fees_allocated = 1_000;

        // Company pending balance is 800.
        // 75% of 800 is 600.
        // A request of 601 is within pending but above the Dam limit.
        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Company, 601),
            "ReleaseLimitExceeded",
        );
    }
```

#### `authorizes_release_at_exact_dam_limit` — line 533

```rust
    fn authorizes_release_at_exact_dam_limit() {
        // The Adaptive Dam tightens this Caution-stage fixture to
        // Controlled: 5,000 BPS of 800 pending units equals 400.
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.pending_reserve = 200;
        treasury.pending_buyback_burn = 0;
        treasury.pending_liquidity = 0;
        treasury.pending_company = 800;
        treasury.pending_founder = 0;

        treasury.lifetime_reserve = 200;
        treasury.lifetime_buyback_burn = 0;
        treasury.lifetime_liquidity = 0;
        treasury.lifetime_company = 800;
        treasury.lifetime_founder = 0;

        treasury.total_fees_received = 1_000;
        treasury.total_fees_allocated = 1_000;

        let authorization =
            authorize_release(&protocol, &treasury, ReleaseBucket::Company, 400).unwrap();

        assert_eq!(authorization.requested_amount, 400);
        assert_eq!(authorization.pending_balance, 800);
        assert_eq!(authorization.maximum_release, 400);
        assert_eq!(authorization.waterfall_stage, WaterfallStage::Caution);
        assert_eq!(authorization.dam_level, DamLevel::Controlled);
        assert_eq!(authorization.release_bps, 5_000);
        assert_eq!(authorization.reserve_ratio_bps, 2_000);
    }
```

#### `rejects_when_allocated_fees_exceed_received_fees` — line 567

```rust
    fn rejects_when_allocated_fees_exceed_received_fees() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.total_fees_received = 1_399;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
            "AccountingInvariantViolation",
        );
    }
```

#### `rejects_when_total_accounting_is_not_settled` — line 580

```rust
    fn rejects_when_total_accounting_is_not_settled() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        treasury.total_fees_received = 1_401;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
            "AccountingNotSettled",
        );
    }
```

#### `rejects_invalid_reserve_bucket_accounting` — line 593

```rust
    fn rejects_invalid_reserve_bucket_accounting() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        // Lifetime must equal pending + released.
        treasury.lifetime_reserve = 999;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
            "AccountingInvariantViolation",
        );
    }
```

#### `rejects_invalid_non_requested_bucket_accounting` — line 607

```rust
    fn rejects_invalid_non_requested_bucket_accounting() {
        let protocol = valid_protocol();
        let mut treasury = valid_treasury();

        // A corrupted founder bucket must block a reserve release.
        // The guard protects the entire treasury, not only the requested bucket.
        treasury.lifetime_founder = 99;

        assert_anchor_error(
            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 1),
            "AccountingInvariantViolation",
        );
    }
```

#### `pending_balance_returns_correct_bucket_amounts` — line 622

```rust
    fn pending_balance_returns_correct_bucket_amounts() {
        let treasury = valid_treasury();

        assert_eq!(pending_balance(&treasury, ReleaseBucket::Reserve), 1_000);
        assert_eq!(pending_balance(&treasury, ReleaseBucket::BuybackBurn), 100);
        assert_eq!(pending_balance(&treasury, ReleaseBucket::Liquidity), 100);
        assert_eq!(pending_balance(&treasury, ReleaseBucket::Company), 100);
        assert_eq!(pending_balance(&treasury, ReleaseBucket::Founder), 100);
    }
```

## `programs/treasury-router/src/instructions/process_fees.rs`

- Structs found: **2**
- Enums found: **0**
- Functions found: **17**
- Relevant functions selected: **9**

### Relevant functions and tests

#### `handler` — line 83

```rust
pub fn handler(ctx: Context<ProcessFees>) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    // Fail closed before any accounting mutation. Sentinel V2 verifies the
    // exact locked 30/20/20/20/10 configuration, immutable configuration flag,
    // canonical protocol linkage, and lifetime-allocation conservation.
    let pre_linkage_report = sentinel::evaluate_linkage(
        ctx.accounts.protocol_state.key(),
        &ctx.accounts.protocol_config,
        &ctx.accounts.treasury,
    );

    require!(
        pre_linkage_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    let pre_sentinel_report =
        sentinel::evaluate(&ctx.accounts.protocol_config, &ctx.accounts.treasury)?;

    require!(
        pre_sentinel_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    let total_released = ctx
        .accounts
        .treasury
        .released_reserve
        .checked_add(ctx.accounts.treasury.released_buyback_burn)
        .and_then(|value| value.checked_add(ctx.accounts.treasury.released_liquidity))
        .and_then(|value| value.checked_add(ctx.accounts.treasury.released_company))
        .and_then(|value| value.checked_add(ctx.accounts.treasury.released_founder))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let total_received = ctx
        .accounts
        .settlement_vault
        .amount
        .checked_add(total_released)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let amount = total_received
        .checked_sub(ctx.accounts.treasury.total_fees_allocated)
        .ok_or(TreasuryRouterError::AccountingInvariantViolation)?;

    require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);

    let config = &ctx.accounts.protocol_config;

    let total_bps = u32::from(config.reserve_bps)
        .checked_add(u32::from(config.buyback_burn_bps))
        .and_then(|value| value.checked_add(u32::from(config.liquidity_bps)))
        .and_then(|value| value.checked_add(u32::from(config.company_bps)))
        .and_then(|value| value.checked_add(u32::from(config.founder_bps)))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        total_bps == u32::from(BPS_DENOMINATOR),
        TreasuryRouterError::InvalidAllocationConfiguration
    );

    let clock = Clock::get()?;

    let FeeCycleOutcome {
        pre_allocation_waterfall,
        adaptive_allocation,
        requested_company_amount,
        requested_founder_amount,
        company_amount,
        founder_amount,
        reserve_deposit,
        liquidity_deposit,
        buyback_deposit,
        previous_waterfall_stage,
        waterfall_evaluation,
        previous_dam_level,
        pre_dam_health_score,
        dam_evaluation,
        previous_beaver_score,
        beaver_score_evaluation,
    } = process_fee_cycle(
        &mut ctx.accounts.protocol_state,
        &mut ctx.accounts.treasury,
        &mut ctx.accounts.founder_state,
        &mut ctx.accounts.company_state,
        amount,
        clock.unix_timestamp,
    )?;

    // Re-run both Sentinel layers after every state mutation. A failed
    // invariant aborts the transaction atomically and rolls all changes back.
    let post_linkage_report = sentinel::evaluate_linkage(
        ctx.accounts.protocol_state.key(),
        &ctx.accounts.protocol_config,
        &ctx.accounts.treasury,
    );

    require!(
        post_linkage_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    let post_sentinel_report =
        sentinel::evaluate(&ctx.accounts.protocol_config, &ctx.accounts.treasury)?;

    require!(
        post_sentinel_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    msg!("Beavernomics fee accounting completed");
    msg!("Gross fee amount: {}", amount);

    msg!(
        "Pre-allocation Waterfall stage: {} ({})",
        pre_allocation_waterfall.stage.as_u8(),
        pre_allocation_waterfall.stage.label()
    );

    msg!(
        "Adaptive allocation BPS R/B/L/C/F: {}/{}/{}/{}/{}",
        adaptive_allocation.reserve_bps,
        adaptive_allocation.buyback_burn_bps,
        adaptive_allocation.liquidity_bps,
        adaptive_allocation.company_bps,
        adaptive_allocation.founder_bps
    );

    msg!(
        "Reserve allocation including rounding: {}",
        reserve_deposit.normal_amount
    );

    msg!("Total reserve allocation: {}", reserve_deposit.total_amount);

    msg!(
        "Liquidity allocation deposited: {}",
        liquidity_deposit.amount
    );

    msg!(
        "Pending liquidity balance: {}",
        liquidity_deposit.pending_balance
    );

    msg!(
        "Lifetime liquidity allocation: {}",
        liquidity_deposit.lifetime_total
    );

    msg!(
        "Buyback-and-burn allocation deposited: {}",
        buyback_deposit.amount
    );

    msg!(
        "Pending buyback-and-burn balance: {}",
        buyback_deposit.pending_balance
    );

    msg!(
        "Lifetime buyback-and-burn allocation: {}",
        buyback_deposit.lifetime_total
    );

    msg!("Company requested allocation: {}", requested_company_amount);

    msg!("Company actual allocation: {}", company_amount);

    msg!(
        "Company spent during current period: {}",
        ctx.accounts.company_state.spent_current_period
    );

    msg!(
        "Company period cap: {}",
        ctx.accounts.company_state.period_cap
    );

    msg!("Founder requested allocation: {}", requested_founder_amount);

    msg!("Founder actual allocation: {}", founder_amount);

    msg!(
        "Founder earned during current period: {}",
        ctx.accounts.founder_state.earned_current_period
    );

    msg!(
        "Founder period cap: {}",
        ctx.accounts.founder_state.period_cap
    );

    msg!("Waterfall previous stage: {}", previous_waterfall_stage);

    msg!(
        "Waterfall current stage: {} ({})",
        waterfall_evaluation.stage.as_u8(),
        waterfall_evaluation.stage.label()
    );

    msg!(
        "Waterfall reserve ratio: {} basis points",
        waterfall_evaluation.reserve_ratio_bps
    );

    msg!(
        "Waterfall pending reserve: {}",
        waterfall_evaluation.pending_reserve
    );

    msg!(
        "Waterfall total pending allocations: {}",
        waterfall_evaluation.total_pending
    );

    msg!("Dam previous level: {}", previous_dam_level);

    msg!(
        "Pre-Dam Health Score: {} / {}",
        pre_dam_health_score,
        beaver_score::PRE_DAM_MAX_POINTS
    );

    msg!(
        "Dam current level: {} ({})",
        dam_evaluation.level.as_u8(),
        dam_evaluation.level.label()
    );

    msg!(
        "Dam maximum release rate: {} basis points",
        dam_evaluation.release_bps
    );

    msg!("Beaver Score previous value: {}", previous_beaver_score);

    msg!(
        "Beaver Score current value: {} / {}",
        beaver_score_evaluation.total_score,
        beaver_score::MAX_BEAVER_SCORE
    );

    msg!(
        "Beaver Score Waterfall component: {} / {}",
        beaver_score_evaluation.waterfall_points,
        beaver_score::WATERFALL_MAX_POINTS
    );

    msg!(
        "Beaver Score Dam component: {} / {}",
        beaver_score_evaluation.dam_points,
        beaver_score::DAM_MAX_POINTS
    );

    msg!(
        "Beaver Score reserve component: {} / {}",
        beaver_score_evaluation.reserve_points,
        beaver_score::RESERVE_MAX_POINTS
    );

    msg!(
        "Beaver Score accounting component: {} / {}",
        beaver_score_evaluation.accounting_points,
        beaver_score::ACCOUNTING_MAX_POINTS
    );

    msg!(
        "Processing epoch: {}",
        ctx.accounts.treasury.processing_epoch
    );

    Ok(())
}
```

#### `process_fee_cycle` — line 393

```rust
pub fn process_fee_cycle(
    protocol_state: &mut ProtocolState,
    treasury: &mut TreasuryState,
    founder_state: &mut FounderState,
    company_state: &mut CompanyState,
    amount: u64,
    now: i64,
) -> Result<FeeCycleOutcome> {
    require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);

    // Evaluate health before applying this cycle so the transaction cannot
    // improve the stage used to allocate itself.
    let pre_allocation_waterfall = waterfall::evaluate(treasury)?;

    let adaptive_allocation = waterfall::allocation_for_stage(pre_allocation_waterfall.stage);

    require!(
        adaptive_allocation.total_bps() == u32::from(BPS_DENOMINATOR),
        TreasuryRouterError::InvalidAllocationConfiguration
    );

    let base_reserve_amount = calculate_share(amount, adaptive_allocation.reserve_bps)?;

    let buyback_amount = calculate_share(amount, adaptive_allocation.buyback_burn_bps)?;

    let liquidity_amount = calculate_share(amount, adaptive_allocation.liquidity_bps)?;

    let requested_company_amount = calculate_share(amount, adaptive_allocation.company_bps)?;

    let requested_founder_amount = calculate_share(amount, adaptive_allocation.founder_bps)?;

    let allocated_before_remainder = base_reserve_amount
        .checked_add(buyback_amount)
        .and_then(|value| value.checked_add(liquidity_amount))
        .and_then(|value| value.checked_add(requested_company_amount))
        .and_then(|value| value.checked_add(requested_founder_amount))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let rounding_remainder = amount
        .checked_sub(allocated_before_remainder)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let normal_reserve_amount = base_reserve_amount
        .checked_add(rounding_remainder)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let founder_allocation = founder::allocate(founder_state, requested_founder_amount, now)?;

    let company_allocation = company::allocate(company_state, requested_company_amount, now)?;

    let founder_amount = founder_allocation.founder_amount;
    let company_amount = company_allocation.company_amount;

    // Locked Bevernomics rule: all Company and Founder cap overflow is
    // redirected to Liquidity Growth.
    let final_liquidity_amount = liquidity_amount
        .checked_add(founder_allocation.liquidity_overflow_amount)
        .and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let final_allocated_amount = normal_reserve_amount
        .checked_add(buyback_amount)
        .and_then(|value| value.checked_add(final_liquidity_amount))
        .and_then(|value| value.checked_add(company_amount))
        .and_then(|value| value.checked_add(founder_amount))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        final_allocated_amount == amount,
        TreasuryRouterError::InvalidAllocationConfiguration
    );

    treasury.total_fees_received = treasury
        .total_fees_received
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.total_fees_allocated = treasury
        .total_fees_allocated
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let reserve_deposit = reserve::deposit_fee_allocation(treasury, normal_reserve_amount)?;

    let liquidity_deposit = liquidity::deposit(treasury, final_liquidity_amount)?;

    let buyback_deposit = buyback::deposit(treasury, buyback_amount)?;

    treasury.pending_company = treasury
        .pending_company
        .checked_add(company_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.pending_founder = treasury
        .pending_founder
        .checked_add(founder_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.lifetime_company = treasury
        .lifetime_company
        .checked_add(company_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.lifetime_founder = treasury
        .lifetime_founder
        .checked_add(founder_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.processing_epoch = treasury
        .processing_epoch
        .checked_add(1)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.last_processed_at = now;

    let previous_waterfall_stage = treasury.waterfall_stage;
    let waterfall_evaluation = waterfall::evaluate(treasury)?;

    treasury.waterfall_stage = waterfall_evaluation.stage.as_u8();

    let previous_dam_level = protocol_state.dam_level;

    let pre_dam_health_score = beaver_score::pre_dam_health_score(
        treasury,
        waterfall_evaluation.reserve_ratio_bps,
        waterfall_evaluation.stage,
    )?;

    let dam_evaluation = dam::evaluate_adaptive(waterfall_evaluation.stage, pre_dam_health_score);

    protocol_state.dam_level = dam_evaluation.level.as_u8();

    let previous_beaver_score = protocol_state.beaver_score;

    let beaver_score_evaluation = beaver_score::evaluate(
        treasury,
        waterfall_evaluation.reserve_ratio_bps,
        waterfall_evaluation.stage,
        dam_evaluation.level,
    )?;

    protocol_state.beaver_score = beaver_score_evaluation.total_score;

    Ok(FeeCycleOutcome {
        pre_allocation_waterfall,
        adaptive_allocation,
        requested_company_amount,
        requested_founder_amount,
        company_amount,
        founder_amount,
        reserve_deposit,
        liquidity_deposit,
        buyback_deposit,
        previous_waterfall_stage,
        waterfall_evaluation,
        previous_dam_level,
        pre_dam_health_score,
        dam_evaluation,
        previous_beaver_score,
        beaver_score_evaluation,
    })
}
```

#### `calculate_share` — line 556

```rust
fn calculate_share(amount: u64, basis_points: u16) -> Result<u64> {
    let numerator = u128::from(amount)
        .checked_mul(u128::from(basis_points))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let share = numerator
        .checked_div(u128::from(BPS_DENOMINATOR))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    u64::try_from(share).map_err(|_| TreasuryRouterError::ArithmeticOverflow.into())
}
```

#### `calculate_share_handles_u64_max_without_multiplication_overflow` — line 623

```rust
    fn calculate_share_handles_u64_max_without_multiplication_overflow() {
        let result = calculate_share(u64::MAX, BPS_DENOMINATOR).unwrap();

        assert_eq!(result, u64::MAX);
    }
```

#### `company_cap_overflow_redirected_to_liquidity_is_conserved` — line 675

```rust
    fn company_cap_overflow_redirected_to_liquidity_is_conserved() {
        let normal_reserve: u64 = 300;
        let buyback: u64 = 200;
        let base_liquidity: u64 = 200;
        let requested_company: u64 = 200;
        let founder = 100;

        let actual_company: u64 = 50;
        let company_overflow = requested_company - actual_company;

        let final_liquidity = base_liquidity.checked_add(company_overflow).unwrap();

        let total = calculate_total(
            normal_reserve,
            buyback,
            final_liquidity,
            actual_company,
            founder,
        )
        .unwrap();

        assert_eq!(company_overflow, 150);
        assert_eq!(final_liquidity, 350);
        assert_eq!(total, 1_000);
    }
```

#### `founder_cap_overflow_redirected_to_liquidity_is_conserved` — line 702

```rust
    fn founder_cap_overflow_redirected_to_liquidity_is_conserved() {
        let normal_reserve: u64 = 300;
        let buyback: u64 = 200;
        let base_liquidity: u64 = 200;
        let company = 200;
        let requested_founder: u64 = 100;

        let actual_founder: u64 = 25;
        let founder_overflow = requested_founder - actual_founder;

        let final_liquidity = base_liquidity.checked_add(founder_overflow).unwrap();

        let total = calculate_total(
            normal_reserve,
            buyback,
            final_liquidity,
            company,
            actual_founder,
        )
        .unwrap();

        assert_eq!(founder_overflow, 75);
        assert_eq!(final_liquidity, 275);
        assert_eq!(total, 1_000);
    }
```

#### `simultaneous_company_and_founder_overflow_is_conserved` — line 729

```rust
    fn simultaneous_company_and_founder_overflow_is_conserved() {
        let normal_reserve: u64 = 300;
        let buyback: u64 = 200;
        let base_liquidity: u64 = 200;

        let requested_company: u64 = 200;
        let actual_company: u64 = 50;
        let company_overflow = requested_company - actual_company;

        let requested_founder: u64 = 100;
        let actual_founder: u64 = 25;
        let founder_overflow = requested_founder - actual_founder;

        let final_liquidity = base_liquidity
            .checked_add(company_overflow)
            .and_then(|value| value.checked_add(founder_overflow))
            .unwrap();

        let total = calculate_total(
            normal_reserve,
            buyback,
            final_liquidity,
            actual_company,
            actual_founder,
        )
        .unwrap();

        assert_eq!(company_overflow, 150);
        assert_eq!(founder_overflow, 75);
        assert_eq!(final_liquidity, 425);
        assert_eq!(total, 1_000);
    }
```

#### `fully_capped_company_and_founder_allocations_go_to_liquidity` — line 763

```rust
    fn fully_capped_company_and_founder_allocations_go_to_liquidity() {
        let normal_reserve: u64 = 300;
        let buyback: u64 = 200;
        let base_liquidity: u64 = 200;

        let actual_company: u64 = 0;
        let company_overflow: u64 = 200;

        let actual_founder: u64 = 0;
        let founder_overflow: u64 = 100;

        let final_liquidity = base_liquidity
            .checked_add(company_overflow)
            .and_then(|value| value.checked_add(founder_overflow))
            .unwrap();

        let total = calculate_total(
            normal_reserve,
            buyback,
            final_liquidity,
            actual_company,
            actual_founder,
        )
        .unwrap();

        assert_eq!(final_liquidity, 500);
        assert_eq!(total, 1_000);
    }
```

#### `allocation_total_fails_on_overflow` — line 793

```rust
    fn allocation_total_fails_on_overflow() {
        let result = calculate_total(u64::MAX, 1, 0, 0, 0);

        assert!(result.is_err());
    }
```

## `programs/treasury-router/src/state/treasury.rs`

- Structs found: **1**
- Enums found: **0**
- Functions found: **32**
- Relevant functions selected: **32**

### Relevant structs

#### `TreasuryState` — line 4

```rust
pub struct TreasuryState {
    /// State layout version for future migrations.
    pub version: u16,

    /// Root ProtocolState controlling this treasury.
    pub protocol: Pubkey,

    /// Mint in which fees are accounted.
    pub settlement_mint: Pubkey,

    /// SPL Token vault holding the treasury's settlement tokens.
    pub settlement_vault: Pubkey,

    /// Total fee units received by the treasury engine.
    pub total_fees_received: u64,

    /// Total fee units assigned to protocol buckets.
    pub total_fees_allocated: u64,

    /// Amounts assigned but not yet transferred or spent.
    pub pending_reserve: u64,
    pub pending_buyback_burn: u64,
    pub pending_liquidity: u64,
    pub pending_company: u64,
    pub pending_founder: u64,

    /// Lifetime allocations for transparent protocol accounting.
    pub lifetime_reserve: u64,
    pub lifetime_buyback_burn: u64,
    pub lifetime_liquidity: u64,
    pub lifetime_company: u64,
    pub lifetime_founder: u64,

    /// Lifetime settlement-token amounts actually released from the treasury.
    pub released_reserve: u64,
    pub released_buyback_burn: u64,
    pub released_liquidity: u64,
    pub released_company: u64,
    pub released_founder: u64,

    /// Last successful fee-processing timestamp.
    pub last_processed_at: i64,

    /// Increments after every successful accounting cycle.
    pub processing_epoch: u64,

    /// Current Survival Waterfall operating stage.
    pub waterfall_stage: u8,

    /// Buyback execution can be paused independently.
    pub buybacks_paused: bool,

    /// Canonical treasury PDA bump.
    pub bump: u8,

    /// Reserved account space for compatible future fields.
    pub reserved: [u8; 24],
}
```

### Relevant functions and tests

#### `accounting_bucket_is_valid` — line 84

```rust
    fn accounting_bucket_is_valid(pending: u64, released: u64, lifetime: u64) -> bool {
        pending
            .checked_add(released)
            .is_some_and(|total| total == lifetime)
    }
```

#### `reserve_accounting_is_valid` — line 90

```rust
    pub fn reserve_accounting_is_valid(&self) -> bool {
        Self::accounting_bucket_is_valid(
            self.pending_reserve,
            self.released_reserve,
            self.lifetime_reserve,
        )
    }
```

#### `buyback_accounting_is_valid` — line 98

```rust
    pub fn buyback_accounting_is_valid(&self) -> bool {
        Self::accounting_bucket_is_valid(
            self.pending_buyback_burn,
            self.released_buyback_burn,
            self.lifetime_buyback_burn,
        )
    }
```

#### `liquidity_accounting_is_valid` — line 106

```rust
    pub fn liquidity_accounting_is_valid(&self) -> bool {
        Self::accounting_bucket_is_valid(
            self.pending_liquidity,
            self.released_liquidity,
            self.lifetime_liquidity,
        )
    }
```

#### `company_accounting_is_valid` — line 114

```rust
    pub fn company_accounting_is_valid(&self) -> bool {
        Self::accounting_bucket_is_valid(
            self.pending_company,
            self.released_company,
            self.lifetime_company,
        )
    }
```

#### `founder_accounting_is_valid` — line 122

```rust
    pub fn founder_accounting_is_valid(&self) -> bool {
        Self::accounting_bucket_is_valid(
            self.pending_founder,
            self.released_founder,
            self.lifetime_founder,
        )
    }
```

#### `execution_accounting_is_valid` — line 130

```rust
    pub fn execution_accounting_is_valid(&self) -> bool {
        self.reserve_accounting_is_valid()
            && self.buyback_accounting_is_valid()
            && self.liquidity_accounting_is_valid()
            && self.company_accounting_is_valid()
            && self.founder_accounting_is_valid()
    }
```

#### `valid_treasury` — line 143

```rust
    fn valid_treasury() -> TreasuryState {
        TreasuryState {
            version: 1,
            protocol: Pubkey::new_unique(),
            settlement_mint: Pubkey::new_unique(),
            settlement_vault: Pubkey::new_unique(),

            total_fees_received: 1_500,
            total_fees_allocated: 1_500,

            pending_reserve: 250,
            pending_buyback_burn: 200,
            pending_liquidity: 300,
            pending_company: 400,
            pending_founder: 100,

            lifetime_reserve: 300,
            lifetime_buyback_burn: 250,
            lifetime_liquidity: 350,
            lifetime_company: 500,
            lifetime_founder: 100,

            released_reserve: 50,
            released_buyback_burn: 50,
            released_liquidity: 50,
            released_company: 100,
            released_founder: 0,

            last_processed_at: 1,
            processing_epoch: 1,
            waterfall_stage: 0,
            buybacks_paused: false,
            bump: 255,
            reserved: [0; 24],
        }
    }
```

#### `reserve_accounting_is_valid_when_lifetime_equals_pending_plus_released` — line 181

```rust
    fn reserve_accounting_is_valid_when_lifetime_equals_pending_plus_released() {
        let treasury = valid_treasury();

        assert!(treasury.reserve_accounting_is_valid());
    }
```

#### `reserve_accounting_is_invalid_when_values_do_not_balance` — line 188

```rust
    fn reserve_accounting_is_invalid_when_values_do_not_balance() {
        let mut treasury = valid_treasury();

        treasury.lifetime_reserve = 299;

        assert!(!treasury.reserve_accounting_is_valid());
    }
```

#### `reserve_accounting_fails_closed_on_overflow` — line 197

```rust
    fn reserve_accounting_fails_closed_on_overflow() {
        let mut treasury = valid_treasury();

        treasury.pending_reserve = u64::MAX;
        treasury.released_reserve = 1;
        treasury.lifetime_reserve = 0;

        assert!(!treasury.reserve_accounting_is_valid());
    }
```

#### `buyback_accounting_is_valid_when_lifetime_equals_pending_plus_released` — line 208

```rust
    fn buyback_accounting_is_valid_when_lifetime_equals_pending_plus_released() {
        let treasury = valid_treasury();

        assert!(treasury.buyback_accounting_is_valid());
    }
```

#### `buyback_accounting_is_invalid_when_values_do_not_balance` — line 215

```rust
    fn buyback_accounting_is_invalid_when_values_do_not_balance() {
        let mut treasury = valid_treasury();

        treasury.lifetime_buyback_burn = 249;

        assert!(!treasury.buyback_accounting_is_valid());
    }
```

#### `buyback_accounting_fails_closed_on_overflow` — line 224

```rust
    fn buyback_accounting_fails_closed_on_overflow() {
        let mut treasury = valid_treasury();

        treasury.pending_buyback_burn = u64::MAX;
        treasury.released_buyback_burn = 1;
        treasury.lifetime_buyback_burn = 0;

        assert!(!treasury.buyback_accounting_is_valid());
    }
```

#### `liquidity_accounting_is_valid_when_lifetime_equals_pending_plus_released` — line 235

```rust
    fn liquidity_accounting_is_valid_when_lifetime_equals_pending_plus_released() {
        let treasury = valid_treasury();

        assert!(treasury.liquidity_accounting_is_valid());
    }
```

#### `liquidity_accounting_is_invalid_when_values_do_not_balance` — line 242

```rust
    fn liquidity_accounting_is_invalid_when_values_do_not_balance() {
        let mut treasury = valid_treasury();

        treasury.lifetime_liquidity = 349;

        assert!(!treasury.liquidity_accounting_is_valid());
    }
```

#### `liquidity_accounting_fails_closed_on_overflow` — line 251

```rust
    fn liquidity_accounting_fails_closed_on_overflow() {
        let mut treasury = valid_treasury();

        treasury.pending_liquidity = u64::MAX;
        treasury.released_liquidity = 1;
        treasury.lifetime_liquidity = 0;

        assert!(!treasury.liquidity_accounting_is_valid());
    }
```

#### `company_accounting_is_valid_when_lifetime_equals_pending_plus_released` — line 262

```rust
    fn company_accounting_is_valid_when_lifetime_equals_pending_plus_released() {
        let treasury = valid_treasury();

        assert!(treasury.company_accounting_is_valid());
    }
```

#### `company_accounting_is_invalid_when_values_do_not_balance` — line 269

```rust
    fn company_accounting_is_invalid_when_values_do_not_balance() {
        let mut treasury = valid_treasury();

        treasury.lifetime_company = 499;

        assert!(!treasury.company_accounting_is_valid());
    }
```

#### `company_accounting_fails_closed_on_overflow` — line 278

```rust
    fn company_accounting_fails_closed_on_overflow() {
        let mut treasury = valid_treasury();

        treasury.pending_company = u64::MAX;
        treasury.released_company = 1;
        treasury.lifetime_company = 0;

        assert!(!treasury.company_accounting_is_valid());
    }
```

#### `founder_accounting_is_valid_when_lifetime_equals_pending_plus_released` — line 289

```rust
    fn founder_accounting_is_valid_when_lifetime_equals_pending_plus_released() {
        let treasury = valid_treasury();

        assert!(treasury.founder_accounting_is_valid());
    }
```

#### `founder_accounting_is_invalid_when_values_do_not_balance` — line 296

```rust
    fn founder_accounting_is_invalid_when_values_do_not_balance() {
        let mut treasury = valid_treasury();

        treasury.lifetime_founder = 99;

        assert!(!treasury.founder_accounting_is_valid());
    }
```

#### `founder_accounting_fails_closed_on_overflow` — line 305

```rust
    fn founder_accounting_fails_closed_on_overflow() {
        let mut treasury = valid_treasury();

        treasury.pending_founder = u64::MAX;
        treasury.released_founder = 1;
        treasury.lifetime_founder = 0;

        assert!(!treasury.founder_accounting_is_valid());
    }
```

#### `execution_accounting_is_valid_when_every_bucket_balances` — line 316

```rust
    fn execution_accounting_is_valid_when_every_bucket_balances() {
        let treasury = valid_treasury();

        assert!(treasury.execution_accounting_is_valid());
    }
```

#### `execution_accounting_is_invalid_when_reserve_is_corrupted` — line 323

```rust
    fn execution_accounting_is_invalid_when_reserve_is_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_reserve += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }
```

#### `execution_accounting_is_invalid_when_buyback_is_corrupted` — line 332

```rust
    fn execution_accounting_is_invalid_when_buyback_is_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_buyback_burn += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }
```

#### `execution_accounting_is_invalid_when_liquidity_is_corrupted` — line 341

```rust
    fn execution_accounting_is_invalid_when_liquidity_is_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_liquidity += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }
```

#### `execution_accounting_is_invalid_when_company_is_corrupted` — line 350

```rust
    fn execution_accounting_is_invalid_when_company_is_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_company += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }
```

#### `execution_accounting_is_invalid_when_founder_is_corrupted` — line 359

```rust
    fn execution_accounting_is_invalid_when_founder_is_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_founder += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }
```

#### `execution_accounting_is_invalid_when_multiple_buckets_are_corrupted` — line 368

```rust
    fn execution_accounting_is_invalid_when_multiple_buckets_are_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_reserve += 1;
        treasury.lifetime_liquidity += 1;
        treasury.lifetime_founder += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }
```

#### `overflow_with_max_lifetime_fails_closed_for_every_bucket` — line 379

```rust
    fn overflow_with_max_lifetime_fails_closed_for_every_bucket() {
        let mut treasury = valid_treasury();

        treasury.pending_reserve = u64::MAX;
        treasury.released_reserve = u64::MAX;
        treasury.lifetime_reserve = u64::MAX;

        treasury.pending_buyback_burn = u64::MAX;
        treasury.released_buyback_burn = u64::MAX;
        treasury.lifetime_buyback_burn = u64::MAX;

        treasury.pending_liquidity = u64::MAX;
        treasury.released_liquidity = u64::MAX;
        treasury.lifetime_liquidity = u64::MAX;

        treasury.pending_company = u64::MAX;
        treasury.released_company = u64::MAX;
        treasury.lifetime_company = u64::MAX;

        treasury.pending_founder = u64::MAX;
        treasury.released_founder = u64::MAX;
        treasury.lifetime_founder = u64::MAX;

        assert!(!treasury.reserve_accounting_is_valid());
        assert!(!treasury.buyback_accounting_is_valid());
        assert!(!treasury.liquidity_accounting_is_valid());
        assert!(!treasury.company_accounting_is_valid());
        assert!(!treasury.founder_accounting_is_valid());
        assert!(!treasury.execution_accounting_is_valid());
    }
```

#### `zero_balances_are_valid_accounting` — line 411

```rust
    fn zero_balances_are_valid_accounting() {
        let mut treasury = valid_treasury();

        treasury.pending_reserve = 0;
        treasury.pending_buyback_burn = 0;
        treasury.pending_liquidity = 0;
        treasury.pending_company = 0;
        treasury.pending_founder = 0;

        treasury.lifetime_reserve = 0;
        treasury.lifetime_buyback_burn = 0;
        treasury.lifetime_liquidity = 0;
        treasury.lifetime_company = 0;
        treasury.lifetime_founder = 0;

        treasury.released_reserve = 0;
        treasury.released_buyback_burn = 0;
        treasury.released_liquidity = 0;
        treasury.released_company = 0;
        treasury.released_founder = 0;

        assert!(treasury.execution_accounting_is_valid());
    }
```

## `programs/treasury-router/src/state/protocol.rs`

- Structs found: **1**
- Enums found: **0**
- Functions found: **0**
- Relevant functions selected: **0**

### Relevant structs

#### `ProtocolState` — line 4

```rust
pub struct ProtocolState {
    /// State layout version for future migrations.
    pub version: u16,

    /// Current protocol administration authority.
    pub authority: Pubkey,

    /// Module account references. These remain Pubkey::default()
    /// until each module is initialized.
    pub protocol_config: Pubkey,
    pub treasury_state: Pubkey,
    pub reserve_state: Pubkey,
    pub liquidity_state: Pubkey,
    pub founder_state: Pubkey,
    pub company_state: Pubkey,
    pub buyback_state: Pubkey,

    /// Protocol health indicators.
    pub beaver_score: u16,
    pub dam_level: u8,

    /// Emergency protocol pause.
    pub paused: bool,

    /// Canonical PDA bump.
    pub bump: u8,

    /// Unix timestamp when the protocol was initialized.
    pub initialized_at: i64,

    /// Reserved account space for compatible future fields.
    pub reserved: [u8; 64],
}
```

## `programs/treasury-router/src/state/protocol_config.rs`

- Structs found: **1**
- Enums found: **0**
- Functions found: **1**
- Relevant functions selected: **0**

### Relevant structs

#### `ProtocolConfig` — line 4

```rust
pub struct ProtocolConfig {
    /// State layout version.
    pub version: u16,

    /// Root ProtocolState that owns this configuration.
    pub protocol: Pubkey,

    /// Allocation of collected fee proceeds, in basis points.
    pub reserve_bps: u16,
    pub buyback_burn_bps: u16,
    pub liquidity_bps: u16,
    pub company_bps: u16,
    pub founder_bps: u16,

    /// Whether configuration changes are currently permitted.
    pub updates_enabled: bool,

    /// Canonical PDA bump.
    pub bump: u8,

    /// Timestamp of the most recent configuration update.
    pub updated_at: i64,

    /// Reserved space for future compatible fields.
    pub reserved: [u8; 64],
}
```

## `programs/treasury-router/src/state/company.rs`

- Structs found: **1**
- Enums found: **0**
- Functions found: **0**
- Relevant functions selected: **0**

### Relevant structs

#### `CompanyState` — line 4

```rust
pub struct CompanyState {
    /// State layout version for future migrations.
    pub version: u16,

    /// Root ProtocolState controlling this company account.
    pub protocol: Pubkey,

    /// Wallet authorized to receive company allocations.
    pub recipient: Pubkey,

    /// Maximum company allocation during the current accounting period.
    pub period_cap: u64,

    /// Company allocation credited during the current period.
    pub spent_current_period: u64,

    /// Lifetime company allocation credited by the protocol.
    pub lifetime_spent: u64,

    /// Unix timestamp marking the start of the current period.
    pub period_started_at: i64,

    /// Length of each accounting period in seconds.
    pub period_duration: i64,

    /// Allows company allocations to be disabled independently.
    pub enabled: bool,

    /// Canonical CompanyState PDA bump.
    pub bump: u8,

    /// Reserved account space for compatible future fields.
    pub reserved: [u8; 64],
}
```

## `programs/treasury-router/src/state/founder.rs`

- Structs found: **1**
- Enums found: **0**
- Functions found: **0**
- Relevant functions selected: **0**

### Relevant structs

#### `FounderState` — line 4

```rust
pub struct FounderState {
    /// State layout version for future migrations.
    pub version: u16,

    /// Root ProtocolState controlling this founder account.
    pub protocol: Pubkey,

    /// Wallet authorized to receive founder allocations.
    pub recipient: Pubkey,

    /// Maximum founder allocation during the current accounting period.
    pub period_cap: u64,

    /// Founder allocation credited during the current period.
    pub earned_current_period: u64,

    /// Lifetime founder allocation credited by the protocol.
    pub lifetime_earned: u64,

    /// Unix timestamp marking the start of the current period.
    pub period_started_at: i64,

    /// Length of each accounting period in seconds.
    pub period_duration: i64,

    /// Progressive compensation tier. Initially zero.
    pub current_tier: u8,

    /// Allows founder allocations to be disabled independently.
    pub enabled: bool,

    /// Canonical FounderState PDA bump.
    pub bump: u8,

    /// Reserved account space for compatible future fields.
    pub reserved: [u8; 64],
}
```

## Treasury field-location inventory

| Field | Source locations |
|---|---|
| `total_received` | `programs/treasury-router/src/instructions/process_fees.rs:121`, `programs/treasury-router/src/instructions/process_fees.rs:128` |
| `total_allocated` | **Not found** |
| `lifetime_reserve` | `programs/treasury-router/src/engines/beaver_score.rs:185`, `programs/treasury-router/src/engines/beaver_score.rs:286`, `programs/treasury-router/src/engines/execution_guard.rs:290`, `programs/treasury-router/src/engines/execution_guard.rs:486`, `programs/treasury-router/src/engines/execution_guard.rs:514`, `programs/treasury-router/src/engines/execution_guard.rs:545`, `programs/treasury-router/src/engines/execution_guard.rs:598`, `programs/treasury-router/src/engines/integrity_firewall.rs:493`, `programs/treasury-router/src/engines/release.rs:160`, `programs/treasury-router/src/engines/release.rs:326`, `programs/treasury-router/src/engines/reserve.rs:32`, `programs/treasury-router/src/engines/reserve.rs:33`, `programs/treasury-router/src/engines/sentinel.rs:419`, `programs/treasury-router/src/engines/sentinel.rs:516`, `programs/treasury-router/src/engines/sentinel.rs:638` |
| `pending_reserve` | `programs/treasury-router/src/engines/beaver_score.rs:180`, `programs/treasury-router/src/engines/beaver_score.rs:280`, `programs/treasury-router/src/engines/execution_guard.rs:207`, `programs/treasury-router/src/engines/execution_guard.rs:284`, `programs/treasury-router/src/engines/execution_guard.rs:480`, `programs/treasury-router/src/engines/execution_guard.rs:508`, `programs/treasury-router/src/engines/execution_guard.rs:539`, `programs/treasury-router/src/engines/integrity_firewall.rs:487`, `programs/treasury-router/src/engines/release.rs:88`, `programs/treasury-router/src/engines/release.rs:107`, `programs/treasury-router/src/engines/release.rs:154`, `programs/treasury-router/src/engines/release.rs:184`, `programs/treasury-router/src/engines/release.rs:203`, `programs/treasury-router/src/engines/release.rs:324`, `programs/treasury-router/src/engines/reserve.rs:27` |
| `released_reserve` | `programs/treasury-router/src/engines/beaver_score.rs:190`, `programs/treasury-router/src/engines/beaver_score.rs:292`, `programs/treasury-router/src/engines/execution_guard.rs:296`, `programs/treasury-router/src/engines/integrity_firewall.rs:499`, `programs/treasury-router/src/engines/release.rs:88`, `programs/treasury-router/src/engines/release.rs:108`, `programs/treasury-router/src/engines/release.rs:166`, `programs/treasury-router/src/engines/release.rs:184`, `programs/treasury-router/src/engines/release.rs:203`, `programs/treasury-router/src/engines/release.rs:325`, `programs/treasury-router/src/engines/sentinel.rs:510`, `programs/treasury-router/src/instructions/initialize_treasury.rs:92`, `programs/treasury-router/src/instructions/process_fees.rs:114`, `programs/treasury-router/src/state/treasury.rs:38`, `programs/treasury-router/src/state/treasury.rs:93` |
| `lifetime_buyback` | **Not found** |
| `pending_buyback` | **Not found** |
| `released_buyback` | **Not found** |
| `lifetime_liquidity` | `programs/treasury-router/src/engines/beaver_score.rs:187`, `programs/treasury-router/src/engines/beaver_score.rs:288`, `programs/treasury-router/src/engines/execution_guard.rs:292`, `programs/treasury-router/src/engines/execution_guard.rs:488`, `programs/treasury-router/src/engines/execution_guard.rs:516`, `programs/treasury-router/src/engines/execution_guard.rs:547`, `programs/treasury-router/src/engines/integrity_firewall.rs:495`, `programs/treasury-router/src/engines/liquidity.rs:27`, `programs/treasury-router/src/engines/liquidity.rs:28`, `programs/treasury-router/src/engines/liquidity.rs:35`, `programs/treasury-router/src/engines/release.rs:162`, `programs/treasury-router/src/engines/sentinel.rs:421`, `programs/treasury-router/src/engines/sentinel.rs:518`, `programs/treasury-router/src/instructions/initialize_treasury.rs:88`, `programs/treasury-router/src/state/treasury.rs:33` |
| `pending_liquidity` | `programs/treasury-router/src/engines/beaver_score.rs:182`, `programs/treasury-router/src/engines/beaver_score.rs:282`, `programs/treasury-router/src/engines/execution_guard.rs:209`, `programs/treasury-router/src/engines/execution_guard.rs:286`, `programs/treasury-router/src/engines/execution_guard.rs:482`, `programs/treasury-router/src/engines/execution_guard.rs:510`, `programs/treasury-router/src/engines/execution_guard.rs:541`, `programs/treasury-router/src/engines/integrity_firewall.rs:489`, `programs/treasury-router/src/engines/liquidity.rs:22`, `programs/treasury-router/src/engines/liquidity.rs:23`, `programs/treasury-router/src/engines/liquidity.rs:34`, `programs/treasury-router/src/engines/release.rs:93`, `programs/treasury-router/src/engines/release.rs:115`, `programs/treasury-router/src/engines/release.rs:156`, `programs/treasury-router/src/engines/release.rs:189` |
| `released_liquidity` | `programs/treasury-router/src/engines/beaver_score.rs:192`, `programs/treasury-router/src/engines/beaver_score.rs:294`, `programs/treasury-router/src/engines/execution_guard.rs:298`, `programs/treasury-router/src/engines/integrity_firewall.rs:501`, `programs/treasury-router/src/engines/release.rs:93`, `programs/treasury-router/src/engines/release.rs:116`, `programs/treasury-router/src/engines/release.rs:168`, `programs/treasury-router/src/engines/release.rs:189`, `programs/treasury-router/src/engines/release.rs:220`, `programs/treasury-router/src/engines/sentinel.rs:512`, `programs/treasury-router/src/instructions/initialize_treasury.rs:94`, `programs/treasury-router/src/instructions/process_fees.rs:116`, `programs/treasury-router/src/state/treasury.rs:40`, `programs/treasury-router/src/state/treasury.rs:109`, `programs/treasury-router/src/state/treasury.rs:167` |
| `lifetime_company` | `programs/treasury-router/src/engines/beaver_score.rs:188`, `programs/treasury-router/src/engines/beaver_score.rs:289`, `programs/treasury-router/src/engines/execution_guard.rs:293`, `programs/treasury-router/src/engines/execution_guard.rs:489`, `programs/treasury-router/src/engines/execution_guard.rs:517`, `programs/treasury-router/src/engines/execution_guard.rs:548`, `programs/treasury-router/src/engines/integrity_firewall.rs:496`, `programs/treasury-router/src/engines/release.rs:163`, `programs/treasury-router/src/engines/release.rs:293`, `programs/treasury-router/src/engines/sentinel.rs:422`, `programs/treasury-router/src/engines/sentinel.rs:519`, `programs/treasury-router/src/instructions/initialize_treasury.rs:89`, `programs/treasury-router/src/instructions/process_fees.rs:491`, `programs/treasury-router/src/instructions/process_fees.rs:492`, `programs/treasury-router/src/state/treasury.rs:34` |
| `pending_company` | `programs/treasury-router/src/engines/beaver_score.rs:183`, `programs/treasury-router/src/engines/beaver_score.rs:283`, `programs/treasury-router/src/engines/execution_guard.rs:210`, `programs/treasury-router/src/engines/execution_guard.rs:287`, `programs/treasury-router/src/engines/execution_guard.rs:483`, `programs/treasury-router/src/engines/execution_guard.rs:511`, `programs/treasury-router/src/engines/execution_guard.rs:542`, `programs/treasury-router/src/engines/integrity_firewall.rs:490`, `programs/treasury-router/src/engines/release.rs:94`, `programs/treasury-router/src/engines/release.rs:119`, `programs/treasury-router/src/engines/release.rs:157`, `programs/treasury-router/src/engines/release.rs:190`, `programs/treasury-router/src/engines/release.rs:227`, `programs/treasury-router/src/engines/sentinel.rs:507`, `programs/treasury-router/src/engines/waterfall.rs:85` |
| `released_company` | `programs/treasury-router/src/engines/beaver_score.rs:193`, `programs/treasury-router/src/engines/beaver_score.rs:295`, `programs/treasury-router/src/engines/execution_guard.rs:299`, `programs/treasury-router/src/engines/integrity_firewall.rs:502`, `programs/treasury-router/src/engines/release.rs:94`, `programs/treasury-router/src/engines/release.rs:120`, `programs/treasury-router/src/engines/release.rs:169`, `programs/treasury-router/src/engines/release.rs:190`, `programs/treasury-router/src/engines/release.rs:227`, `programs/treasury-router/src/engines/sentinel.rs:513`, `programs/treasury-router/src/instructions/initialize_treasury.rs:95`, `programs/treasury-router/src/instructions/process_fees.rs:117`, `programs/treasury-router/src/state/treasury.rs:41`, `programs/treasury-router/src/state/treasury.rs:117`, `programs/treasury-router/src/state/treasury.rs:168` |
| `lifetime_founder` | `programs/treasury-router/src/engines/beaver_score.rs:189`, `programs/treasury-router/src/engines/beaver_score.rs:290`, `programs/treasury-router/src/engines/execution_guard.rs:294`, `programs/treasury-router/src/engines/execution_guard.rs:490`, `programs/treasury-router/src/engines/execution_guard.rs:518`, `programs/treasury-router/src/engines/execution_guard.rs:549`, `programs/treasury-router/src/engines/execution_guard.rs:613`, `programs/treasury-router/src/engines/integrity_firewall.rs:497`, `programs/treasury-router/src/engines/release.rs:164`, `programs/treasury-router/src/engines/release.rs:308`, `programs/treasury-router/src/engines/sentinel.rs:423`, `programs/treasury-router/src/engines/sentinel.rs:520`, `programs/treasury-router/src/engines/sentinel.rs:624`, `programs/treasury-router/src/instructions/initialize_treasury.rs:90`, `programs/treasury-router/src/instructions/process_fees.rs:496` |
| `pending_founder` | `programs/treasury-router/src/engines/beaver_score.rs:184`, `programs/treasury-router/src/engines/beaver_score.rs:284`, `programs/treasury-router/src/engines/execution_guard.rs:211`, `programs/treasury-router/src/engines/execution_guard.rs:288`, `programs/treasury-router/src/engines/execution_guard.rs:484`, `programs/treasury-router/src/engines/execution_guard.rs:512`, `programs/treasury-router/src/engines/execution_guard.rs:543`, `programs/treasury-router/src/engines/integrity_firewall.rs:491`, `programs/treasury-router/src/engines/release.rs:95`, `programs/treasury-router/src/engines/release.rs:123`, `programs/treasury-router/src/engines/release.rs:158`, `programs/treasury-router/src/engines/release.rs:191`, `programs/treasury-router/src/engines/release.rs:234`, `programs/treasury-router/src/engines/sentinel.rs:508`, `programs/treasury-router/src/engines/waterfall.rs:86` |
| `released_founder` | `programs/treasury-router/src/engines/beaver_score.rs:194`, `programs/treasury-router/src/engines/beaver_score.rs:296`, `programs/treasury-router/src/engines/execution_guard.rs:300`, `programs/treasury-router/src/engines/integrity_firewall.rs:503`, `programs/treasury-router/src/engines/release.rs:95`, `programs/treasury-router/src/engines/release.rs:124`, `programs/treasury-router/src/engines/release.rs:170`, `programs/treasury-router/src/engines/release.rs:191`, `programs/treasury-router/src/engines/release.rs:234`, `programs/treasury-router/src/engines/sentinel.rs:514`, `programs/treasury-router/src/instructions/initialize_treasury.rs:96`, `programs/treasury-router/src/instructions/process_fees.rs:118`, `programs/treasury-router/src/state/treasury.rs:42`, `programs/treasury-router/src/state/treasury.rs:125`, `programs/treasury-router/src/state/treasury.rs:169` |

## Current baseline

Git commit: `988930c6d6b17ebe791e2bf1ff3c793420dd185d`

### `cargo test --workspace`

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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.53s
     Running unittests src/lib.rs (target/debug/deps/treasury_router-338f8d4c1464469d)

running 121 tests
test engines::beaver_score::pre_dam_health_tests::corrupted_accounting_removes_integrity_points ... ok
test engines::beaver_score::pre_dam_health_tests::emergency_pre_dam_health_retains_only_accounting_points ... ok
test engines::beaver_score::pre_dam_health_tests::ideal_pre_dam_health_scores_seven_hundred_fifty ... ok
test engines::beaver_score::tests::emergency_protocol_scores_only_accounting_points ... ok
test engines::beaver_score::tests::current_initial_allocation_scores_correctly ... ok
test engines::beaver_score::tests::ideal_protocol_scores_one_thousand ... ok
test engines::beaver_score::tests::inconsistent_accounting_loses_integrity_points ... ok
test engines::beaver_score::tests::reserve_score_is_capped ... ok
test engines::dam::adaptive_dam_tests::critically_low_health_closes_the_dam ... ok
test engines::dam::adaptive_dam_tests::emergency_waterfall_always_closes_the_dam ... ok
test engines::dam::adaptive_dam_tests::health_score_can_never_weaken_waterfall_protection ... ok
test engines::dam::adaptive_dam_tests::healthy_normal_protocol_allows_full_release ... ok
test engines::dam::adaptive_dam_tests::weak_health_tightens_a_normal_waterfall ... ok
test engines::dam::tests::caution_waterfall_uses_normal_dam_level ... ok
test engines::dam::tests::defensive_waterfall_uses_controlled_dam_level ... ok
test engines::dam::tests::emergency_waterfall_closes_dam ... ok
test engines::dam::tests::normal_waterfall_opens_dam_fully ... ok
test engines::dam::tests::survival_waterfall_restricts_dam ... ok
test engines::execution_guard::tests::authorizes_release_at_exact_dam_limit ... ok
test engines::execution_guard::tests::authorizes_valid_buyback_release ... ok
test engines::execution_guard::tests::authorizes_valid_reserve_release ... ok
test engines::execution_guard::tests::controlled_dam_allows_fifty_percent ... ok
test engines::execution_guard::tests::filling_dam_allows_nothing ... ok
test engines::execution_guard::tests::buyback_pause_does_not_block_other_buckets ... ok
test engines::execution_guard::tests::full_dam_allows_full_pending_balance ... ok
test engines::execution_guard::tests::normal_dam_allows_seventy_five_percent ... ok
test engines::execution_guard::tests::pending_balance_returns_correct_bucket_amounts ... ok
test engines::execution_guard::tests::rejects_buyback_when_buybacks_are_paused ... ok
test engines::execution_guard::tests::rejects_invalid_dam_release_rate ... ok
test engines::execution_guard::tests::rejects_invalid_non_requested_bucket_accounting ... ok
test engines::execution_guard::tests::rejects_invalid_reserve_bucket_accounting ... ok
test engines::execution_guard::tests::rejects_release_exceeding_current_dam_limit ... ok
test engines::execution_guard::tests::rejects_release_exceeding_pending_balance ... ok
test engines::execution_guard::tests::rejects_release_when_dam_is_closed ... ok
test engines::execution_guard::tests::rejects_release_when_protocol_is_paused ... ok
test engines::execution_guard::tests::rejects_when_allocated_fees_exceed_received_fees ... ok
test engines::execution_guard::tests::rejects_when_total_accounting_is_not_settled ... ok
test engines::execution_guard::tests::rejects_zero_release_amount ... ok
test engines::execution_guard::tests::release_calculation_rounds_down ... ok
test engines::execution_guard::tests::restricted_dam_allows_twenty_five_percent ... ok
test engines::integrity_firewall::tests::invariant_masks_are_unique_and_single_bit ... ok
test engines::release::tests::processes_company_release ... ok
test engines::integrity_firewall::tests::corrupted_treasury_bump_is_detected ... ok
test engines::release::tests::processes_founder_release ... ok
test engines::integrity_firewall::tests::valid_architecture_passes_every_integrity_check ... ok
test engines::release::tests::processes_liquidity_release ... ok
test engines::integrity_firewall::tests::duplicate_destinations_are_detected ... ok
test engines::integrity_firewall::tests::wrong_treasury_vault_owner_is_detected ... ok
test engines::release::tests::rejects_corrupted_unrelated_bucket_before_mutation ... ok
test engines::release::tests::rejects_release_above_pending_without_mutating_treasury ... ok
test engines::integrity_firewall::tests::destination_pointing_to_treasury_vault_is_detected ... ok
test engines::release::tests::full_pending_balance_can_be_released ... ok
test engines::release::tests::rejects_released_balance_overflow_without_mutation ... ok
test engines::release::tests::rejects_zero_release_without_mutating_treasury ... ok
test engines::sentinel::sentinel_v2_tests::allocated_fees_cannot_exceed_received_fees ... ok
test engines::release::tests::processes_buyback_release ... ok
test engines::release::tests::processes_reserve_release ... ok
test engines::release::tests::rejects_corrupted_selected_bucket_before_mutation ... ok
test engines::sentinel::sentinel_v2_tests::configuration_updates_enabled_fails_closed ... ok
test engines::sentinel::sentinel_v2_tests::allocation_total_of_one_hundred_percent_is_not_enough ... ok
test engines::integrity_firewall::tests::wrong_destination_mint_is_detected ... ok
test engines::integrity_firewall::tests::wrong_founder_destination_owner_is_detected ... ok
test engines::sentinel::sentinel_v2_tests::linkage_masks_are_unique ... ok
test engines::sentinel::sentinel_v2_tests::lifetime_sum_overflow_fails_closed ... ok
test engines::sentinel::sentinel_v2_tests::corrupted_lifetime_total_is_detected ... ok
test engines::sentinel::sentinel_v2_tests::valid_locked_protocol_passes_every_linkage_check ... ok
test engines::integrity_firewall::tests::wrong_program_id_fails_closed ... ok
test engines::sentinel::sentinel_v2_tests::wrong_protocol_links_are_detected_independently ... ok
test engines::sentinel::tests::canonical_dam_rates_are_never_above_one_hundred_percent ... ok
test engines::sentinel::tests::every_waterfall_stage_produces_a_consistent_dam_evaluation ... ok
test engines::sentinel::sentinel_v2_tests::missing_settlement_references_fail ... ok
test engines::sentinel::tests::invariant_failure_detection_is_exact ... ok
test engines::sentinel::tests::invariant_masks_are_unique_and_single_bit ... ok
test engines::sentinel::tests::zero_failure_mask_contains_no_failures ... ok
test engines::waterfall::adaptive_allocation_tests::defensive_profiles_prioritize_reserve ... ok
test engines::waterfall::adaptive_allocation_tests::every_profile_allocates_exactly_one_hundred_percent ... ok
test engines::waterfall::adaptive_allocation_tests::normal_profile_preserves_locked_beavernomics ... ok
test engines::waterfall::adaptive_allocation_tests::survival_and_emergency_stop_optional_allocations ... ok
test engines::waterfall::tests::classifies_caution_stage ... ok
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
test test_id ... ok
test state::treasury::tests::zero_balances_are_valid_accounting ... ok

test result: ok. 121 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests treasury_router

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Strict Clippy

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
```

### Working tree

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
?? RBVR-IMMUTABILITY-AUDIT.md
?? RBVR-SECURITY-AUDIT-LATEST.md
?? fuzz/fuzz_targets/process_release.rs
?? programs/treasury-router/src/engines/release.rs
?? rbvr-add-double-initialize-test-v2.py
?? rbvr-add-double-initialize-test.py
?? rbvr-authority-lockdown-audit.py
?? rbvr-authority-pass.sh
?? rbvr-autonomous-release-pass-v2.sh
?? rbvr-autonomous-release-pass-v3.sh
?? rbvr-autonomous-release-pass.sh
?? rbvr-deep-security-audit.sh
?? rbvr-double-initialize-pass-v2.sh
?? rbvr-double-initialize-pass.sh
?? rbvr-global-invariant-map.py
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

## Planned global invariant suite

The next patch will use the exact APIs above to exercise deterministic
multi-step state transitions and check after every successful release:

1. `lifetime == pending + released` for every bucket.
2. Sum of bucket lifetime totals equals `total_allocated`.
3. `total_allocated <= total_received`.
4. Releasing one bucket changes no other bucket.
5. Failed releases leave the complete treasury state unchanged.
6. Sequential partial releases conserve all accounting.
7. Full releases leave pending at zero without changing lifetime totals.
8. Every ordering of the five release buckets reaches the same final state.
