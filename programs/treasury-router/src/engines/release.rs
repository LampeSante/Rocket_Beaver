use anchor_lang::prelude::*;

use crate::{
    engines::execution_guard::ReleaseBucket, errors::TreasuryRouterError, state::TreasuryState,
};

/// Accounting result produced by a successful treasury release transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReleaseTransition {
    pub bucket: ReleaseBucket,
    pub amount: u64,
    pub previous_pending_balance: u64,
    pub remaining_pending_balance: u64,
    pub previous_released_balance: u64,
    pub total_released_balance: u64,
}

/// Applies the accounting transition for an already-authorized treasury release.
///
/// This function does not:
/// - authorize the release;
/// - apply the Waterfall or Dam;
/// - transfer SPL tokens;
/// - emit an event.
///
/// The calling instruction must run the Execution Guard before transferring
/// tokens. Solana instruction atomicity ensures that a later accounting failure
/// also rolls back the preceding token CPI.
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

/// Returns pending and released balances for the selected treasury bucket.
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

fn bucket_accounting_is_valid(treasury: &TreasuryState, bucket: ReleaseBucket) -> bool {
    match bucket {
        ReleaseBucket::Reserve => treasury.reserve_accounting_is_valid(),
        ReleaseBucket::BuybackBurn => treasury.buyback_accounting_is_valid(),
        ReleaseBucket::Liquidity => treasury.liquidity_accounting_is_valid(),
        ReleaseBucket::Company => treasury.company_accounting_is_valid(),
        ReleaseBucket::Founder => treasury.founder_accounting_is_valid(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::waterfall::WaterfallStage;

    fn valid_treasury() -> TreasuryState {
        TreasuryState {
            version: 1,
            protocol: Pubkey::new_unique(),
            settlement_mint: Pubkey::new_unique(),
            settlement_vault: Pubkey::new_unique(),

            total_fees_received: 1_800,
            total_fees_allocated: 1_800,

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

    #[test]
    fn processes_reserve_release() {
        assert_release(ReleaseBucket::Reserve, 275, 75);
    }

    #[test]
    fn processes_buyback_release() {
        assert_release(ReleaseBucket::BuybackBurn, 225, 75);
    }

    #[test]
    fn processes_liquidity_release() {
        assert_release(ReleaseBucket::Liquidity, 325, 75);
    }

    #[test]
    fn processes_company_release() {
        assert_release(ReleaseBucket::Company, 375, 125);
    }

    #[test]
    fn processes_founder_release() {
        assert_release(ReleaseBucket::Founder, 175, 75);
    }

    #[test]
    fn rejects_zero_release_without_mutating_treasury() {
        let mut treasury = valid_treasury();
        let before = balances_for_bucket(&treasury, ReleaseBucket::Reserve);

        assert!(process_release(&mut treasury, ReleaseBucket::Reserve, 0).is_err());
        assert_eq!(
            balances_for_bucket(&treasury, ReleaseBucket::Reserve),
            before
        );
    }

    #[test]
    fn rejects_release_above_pending_without_mutating_treasury() {
        let mut treasury = valid_treasury();
        let before = balances_for_bucket(&treasury, ReleaseBucket::Founder);

        assert!(process_release(&mut treasury, ReleaseBucket::Founder, 201).is_err());

        assert_eq!(
            balances_for_bucket(&treasury, ReleaseBucket::Founder),
            before
        );
    }

    #[test]
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

    #[test]
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

    #[test]
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

    #[test]
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

    #[derive(Debug, PartialEq, Eq)]
    struct AccountingSnapshot {
        total_fees_received: u64,
        total_fees_allocated: u64,

        pending_reserve: u64,
        pending_buyback_burn: u64,
        pending_liquidity: u64,
        pending_company: u64,
        pending_founder: u64,

        lifetime_reserve: u64,
        lifetime_buyback_burn: u64,
        lifetime_liquidity: u64,
        lifetime_company: u64,
        lifetime_founder: u64,

        released_reserve: u64,
        released_buyback_burn: u64,
        released_liquidity: u64,
        released_company: u64,
        released_founder: u64,
    }

    fn accounting_snapshot(treasury: &TreasuryState) -> AccountingSnapshot {
        AccountingSnapshot {
            total_fees_received: treasury.total_fees_received,
            total_fees_allocated: treasury.total_fees_allocated,

            pending_reserve: treasury.pending_reserve,
            pending_buyback_burn: treasury.pending_buyback_burn,
            pending_liquidity: treasury.pending_liquidity,
            pending_company: treasury.pending_company,
            pending_founder: treasury.pending_founder,

            lifetime_reserve: treasury.lifetime_reserve,
            lifetime_buyback_burn: treasury.lifetime_buyback_burn,
            lifetime_liquidity: treasury.lifetime_liquidity,
            lifetime_company: treasury.lifetime_company,
            lifetime_founder: treasury.lifetime_founder,

            released_reserve: treasury.released_reserve,
            released_buyback_burn: treasury.released_buyback_burn,
            released_liquidity: treasury.released_liquidity,
            released_company: treasury.released_company,
            released_founder: treasury.released_founder,
        }
    }

    fn checked_bucket_lifetime_total(treasury: &TreasuryState) -> u64 {
        treasury
            .lifetime_reserve
            .checked_add(treasury.lifetime_buyback_burn)
            .and_then(|value| value.checked_add(treasury.lifetime_liquidity))
            .and_then(|value| value.checked_add(treasury.lifetime_company))
            .and_then(|value| value.checked_add(treasury.lifetime_founder))
            .expect("valid fixture lifetime totals must not overflow")
    }

    fn checked_pending_total(treasury: &TreasuryState) -> u64 {
        treasury
            .pending_reserve
            .checked_add(treasury.pending_buyback_burn)
            .and_then(|value| value.checked_add(treasury.pending_liquidity))
            .and_then(|value| value.checked_add(treasury.pending_company))
            .and_then(|value| value.checked_add(treasury.pending_founder))
            .expect("valid fixture pending totals must not overflow")
    }

    fn checked_released_total(treasury: &TreasuryState) -> u64 {
        treasury
            .released_reserve
            .checked_add(treasury.released_buyback_burn)
            .and_then(|value| value.checked_add(treasury.released_liquidity))
            .and_then(|value| value.checked_add(treasury.released_company))
            .and_then(|value| value.checked_add(treasury.released_founder))
            .expect("valid fixture released totals must not overflow")
    }

    fn assert_global_accounting_invariants(treasury: &TreasuryState) {
        assert!(
            treasury.execution_accounting_is_valid(),
            "every bucket must satisfy lifetime = pending + released",
        );

        let lifetime_total = checked_bucket_lifetime_total(treasury);
        let pending_total = checked_pending_total(treasury);
        let released_total = checked_released_total(treasury);

        assert_eq!(
            lifetime_total, treasury.total_fees_allocated,
            "sum of bucket lifetime allocations must equal total allocated fees",
        );

        assert!(
            treasury.total_fees_allocated <= treasury.total_fees_received,
            "allocated fees may never exceed received fees",
        );

        assert_eq!(
            pending_total
                .checked_add(released_total)
                .expect("valid global totals must not overflow"),
            lifetime_total,
            "global pending plus released must equal global lifetime allocation",
        );
    }

    fn collect_release_permutations(
        buckets: &mut [ReleaseBucket; 5],
        index: usize,
        permutations: &mut Vec<[ReleaseBucket; 5]>,
    ) {
        if index == buckets.len() {
            permutations.push(*buckets);
            return;
        }

        for swap_index in index..buckets.len() {
            buckets.swap(index, swap_index);
            collect_release_permutations(buckets, index + 1, permutations);
            buckets.swap(index, swap_index);
        }
    }

    #[test]
    fn all_release_orders_reach_the_same_final_treasury_state() {
        let mut buckets = [
            ReleaseBucket::Reserve,
            ReleaseBucket::BuybackBurn,
            ReleaseBucket::Liquidity,
            ReleaseBucket::Company,
            ReleaseBucket::Founder,
        ];

        let mut permutations = Vec::new();
        collect_release_permutations(&mut buckets, 0, &mut permutations);

        assert_eq!(
            permutations.len(),
            120,
            "five buckets must produce exactly 5! release orders",
        );

        let mut expected_final_snapshot: Option<AccountingSnapshot> = None;

        for (permutation_index, permutation) in permutations.iter().enumerate() {
            let mut treasury = valid_treasury();

            let initial_lifetime_total = checked_bucket_lifetime_total(&treasury);
            let initial_received = treasury.total_fees_received;
            let initial_allocated = treasury.total_fees_allocated;

            assert_global_accounting_invariants(&treasury);

            for bucket in permutation {
                let before = accounting_snapshot(&treasury);
                let before_lifetime_total = checked_bucket_lifetime_total(&treasury);

                let amount = balances_for_bucket(&treasury, *bucket).0;

                assert!(
                    amount > 0,
                    "every fixture bucket must begin with a pending balance",
                );

                let transition = process_release(&mut treasury, *bucket, amount).unwrap();

                assert_eq!(transition.bucket, *bucket);
                assert_eq!(transition.amount, amount);
                assert_eq!(transition.remaining_pending_balance, 0);

                assert_global_accounting_invariants(&treasury);

                assert_eq!(
                    checked_bucket_lifetime_total(&treasury),
                    before_lifetime_total,
                    "release must not alter lifetime allocations",
                );

                assert_eq!(
                    treasury.total_fees_received, initial_received,
                    "release must not alter received-fee accounting",
                );

                assert_eq!(
                    treasury.total_fees_allocated, initial_allocated,
                    "release must not alter allocated-fee accounting",
                );

                match bucket {
                    ReleaseBucket::Reserve => {
                        assert_eq!(treasury.pending_buyback_burn, before.pending_buyback_burn);
                        assert_eq!(treasury.pending_liquidity, before.pending_liquidity);
                        assert_eq!(treasury.pending_company, before.pending_company);
                        assert_eq!(treasury.pending_founder, before.pending_founder);
                    }
                    ReleaseBucket::BuybackBurn => {
                        assert_eq!(treasury.pending_reserve, before.pending_reserve);
                        assert_eq!(treasury.pending_liquidity, before.pending_liquidity);
                        assert_eq!(treasury.pending_company, before.pending_company);
                        assert_eq!(treasury.pending_founder, before.pending_founder);
                    }
                    ReleaseBucket::Liquidity => {
                        assert_eq!(treasury.pending_reserve, before.pending_reserve);
                        assert_eq!(treasury.pending_buyback_burn, before.pending_buyback_burn);
                        assert_eq!(treasury.pending_company, before.pending_company);
                        assert_eq!(treasury.pending_founder, before.pending_founder);
                    }
                    ReleaseBucket::Company => {
                        assert_eq!(treasury.pending_reserve, before.pending_reserve);
                        assert_eq!(treasury.pending_buyback_burn, before.pending_buyback_burn);
                        assert_eq!(treasury.pending_liquidity, before.pending_liquidity);
                        assert_eq!(treasury.pending_founder, before.pending_founder);
                    }
                    ReleaseBucket::Founder => {
                        assert_eq!(treasury.pending_reserve, before.pending_reserve);
                        assert_eq!(treasury.pending_buyback_burn, before.pending_buyback_burn);
                        assert_eq!(treasury.pending_liquidity, before.pending_liquidity);
                        assert_eq!(treasury.pending_company, before.pending_company);
                    }
                }
            }

            assert_eq!(
                checked_pending_total(&treasury),
                0,
                "all pending balances must reach zero",
            );

            assert_eq!(
                checked_released_total(&treasury),
                initial_lifetime_total,
                "fully released treasury must release its lifetime allocation",
            );

            assert_eq!(treasury.released_reserve, treasury.lifetime_reserve);
            assert_eq!(
                treasury.released_buyback_burn,
                treasury.lifetime_buyback_burn
            );
            assert_eq!(treasury.released_liquidity, treasury.lifetime_liquidity);
            assert_eq!(treasury.released_company, treasury.lifetime_company);
            assert_eq!(treasury.released_founder, treasury.lifetime_founder);

            let final_snapshot = accounting_snapshot(&treasury);

            if let Some(expected) = &expected_final_snapshot {
                assert_eq!(
                    &final_snapshot, expected,
                    "release permutation {permutation_index} reached a different final state",
                );
            } else {
                expected_final_snapshot = Some(final_snapshot);
            }
        }
    }
}
