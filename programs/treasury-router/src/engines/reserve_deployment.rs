//! Deterministic RBVR reserve-surplus deployment logic.
//!
//! This engine performs accounting only. It does not move tokens and does not
//! depend on a signer, clock, oracle or mutable administrator.
//!
//! Core invariant:
//!
//! ```text
//! remaining_reserve >= reserve_floor
//! ```
//!
//! Only value above the calculated reserve floor may ever become deployable.

/// Basis-point denominator representing 100%.
pub const BASIS_POINTS_DENOMINATOR: u16 = 10_000;

/// Classification of the current reserve position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReserveDeploymentStage {
    /// Reserve balance is below the required floor.
    Filling,

    /// Reserve balance exactly equals the required floor.
    Healthy,

    /// Reserve balance exceeds the floor and contains deployable surplus.
    Surplus,
}

/// Immutable inputs used to calculate the reserve floor and deployment limit.
///
/// The effective floor is:
///
/// ```text
/// max(
///     minimum_reserve_floor,
///     liquidity_reference × liquidity_floor_bps / 10_000
/// )
/// ```
///
/// `surplus_deployment_bps` controls how much of the current surplus may be
/// deployed during one evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReserveDeploymentPolicy {
    pub minimum_reserve_floor: u64,
    pub liquidity_floor_bps: u16,
    pub surplus_deployment_bps: u16,
}

/// Complete accounting result of one deterministic reserve evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReserveDeploymentEvaluation {
    pub stage: ReserveDeploymentStage,

    pub reserve_balance: u64,
    pub reserve_floor: u64,

    /// Reserve value strictly above the floor before deployment.
    pub gross_surplus: u64,

    /// Maximum value that may be deployed during this evaluation.
    pub deployable_amount: u64,

    /// Reserve balance remaining after deploying `deployable_amount`.
    pub remaining_reserve: u64,
}

/// Errors produced by invalid policy inputs or impossible arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReserveDeploymentError {
    InvalidLiquidityFloorRate,
    InvalidSurplusDeploymentRate,
    ArithmeticOverflow,
    FloorInvariantViolation,
}

/// Calculate a basis-point share without overflowing `u64`.
fn calculate_bps_share(amount: u64, basis_points: u16) -> Result<u64, ReserveDeploymentError> {
    if basis_points > BASIS_POINTS_DENOMINATOR {
        return Err(ReserveDeploymentError::InvalidLiquidityFloorRate);
    }

    let product = u128::from(amount)
        .checked_mul(u128::from(basis_points))
        .ok_or(ReserveDeploymentError::ArithmeticOverflow)?;

    let result = product / u128::from(BASIS_POINTS_DENOMINATOR);

    u64::try_from(result).map_err(|_| ReserveDeploymentError::ArithmeticOverflow)
}

/// Calculate the effective reserve floor.
///
/// The floor scales with the supplied liquidity reference but can never fall
/// below `minimum_reserve_floor`.
pub fn calculate_reserve_floor(
    minimum_reserve_floor: u64,
    liquidity_reference: u64,
    liquidity_floor_bps: u16,
) -> Result<u64, ReserveDeploymentError> {
    let liquidity_floor = calculate_bps_share(liquidity_reference, liquidity_floor_bps)?;

    Ok(minimum_reserve_floor.max(liquidity_floor))
}

/// Evaluate the current reserve and calculate the maximum safe deployment.
///
/// This function is deterministic and side-effect free.
pub fn evaluate_reserve_deployment(
    reserve_balance: u64,
    liquidity_reference: u64,
    policy: ReserveDeploymentPolicy,
) -> Result<ReserveDeploymentEvaluation, ReserveDeploymentError> {
    if policy.liquidity_floor_bps > BASIS_POINTS_DENOMINATOR {
        return Err(ReserveDeploymentError::InvalidLiquidityFloorRate);
    }

    if policy.surplus_deployment_bps > BASIS_POINTS_DENOMINATOR {
        return Err(ReserveDeploymentError::InvalidSurplusDeploymentRate);
    }

    let reserve_floor = calculate_reserve_floor(
        policy.minimum_reserve_floor,
        liquidity_reference,
        policy.liquidity_floor_bps,
    )?;

    let stage = if reserve_balance < reserve_floor {
        ReserveDeploymentStage::Filling
    } else if reserve_balance == reserve_floor {
        ReserveDeploymentStage::Healthy
    } else {
        ReserveDeploymentStage::Surplus
    };

    let gross_surplus = reserve_balance.saturating_sub(reserve_floor);

    let deployable_amount = if stage == ReserveDeploymentStage::Surplus {
        let product = u128::from(gross_surplus)
            .checked_mul(u128::from(policy.surplus_deployment_bps))
            .ok_or(ReserveDeploymentError::ArithmeticOverflow)?;

        let calculated = product / u128::from(BASIS_POINTS_DENOMINATOR);

        u64::try_from(calculated).map_err(|_| ReserveDeploymentError::ArithmeticOverflow)?
    } else {
        0
    };

    let remaining_reserve = reserve_balance
        .checked_sub(deployable_amount)
        .ok_or(ReserveDeploymentError::ArithmeticOverflow)?;

    if deployable_amount > gross_surplus
        || (reserve_balance >= reserve_floor && remaining_reserve < reserve_floor)
    {
        return Err(ReserveDeploymentError::FloorInvariantViolation);
    }

    Ok(ReserveDeploymentEvaluation {
        stage,
        reserve_balance,
        reserve_floor,
        gross_surplus,
        deployable_amount,
        remaining_reserve,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(
        minimum_reserve_floor: u64,
        liquidity_floor_bps: u16,
        surplus_deployment_bps: u16,
    ) -> ReserveDeploymentPolicy {
        ReserveDeploymentPolicy {
            minimum_reserve_floor,
            liquidity_floor_bps,
            surplus_deployment_bps,
        }
    }

    #[test]
    fn reserve_below_floor_is_filling_and_deploys_nothing() {
        let result = evaluate_reserve_deployment(900, 0, policy(1_000, 0, 10_000)).unwrap();

        assert_eq!(result.stage, ReserveDeploymentStage::Filling);
        assert_eq!(result.reserve_floor, 1_000);
        assert_eq!(result.gross_surplus, 0);
        assert_eq!(result.deployable_amount, 0);
        assert_eq!(result.remaining_reserve, 900);
    }

    #[test]
    fn reserve_at_floor_is_healthy_and_deploys_nothing() {
        let result = evaluate_reserve_deployment(1_000, 0, policy(1_000, 0, 10_000)).unwrap();

        assert_eq!(result.stage, ReserveDeploymentStage::Healthy);
        assert_eq!(result.gross_surplus, 0);
        assert_eq!(result.deployable_amount, 0);
        assert_eq!(result.remaining_reserve, 1_000);
    }

    #[test]
    fn full_surplus_can_be_deployed_without_touching_floor() {
        let result = evaluate_reserve_deployment(1_500, 0, policy(1_000, 0, 10_000)).unwrap();

        assert_eq!(result.stage, ReserveDeploymentStage::Surplus);
        assert_eq!(result.gross_surplus, 500);
        assert_eq!(result.deployable_amount, 500);
        assert_eq!(result.remaining_reserve, 1_000);
    }

    #[test]
    fn deployment_rate_throttles_surplus_release() {
        let result = evaluate_reserve_deployment(2_000, 0, policy(1_000, 0, 2_500)).unwrap();

        assert_eq!(result.gross_surplus, 1_000);
        assert_eq!(result.deployable_amount, 250);
        assert_eq!(result.remaining_reserve, 1_750);
    }

    #[test]
    fn zero_deployment_rate_retains_all_surplus() {
        let result = evaluate_reserve_deployment(2_000, 0, policy(1_000, 0, 0)).unwrap();

        assert_eq!(result.stage, ReserveDeploymentStage::Surplus);
        assert_eq!(result.gross_surplus, 1_000);
        assert_eq!(result.deployable_amount, 0);
        assert_eq!(result.remaining_reserve, 2_000);
    }

    #[test]
    fn liquidity_reference_can_raise_effective_floor() {
        let result =
            evaluate_reserve_deployment(3_000, 10_000, policy(1_000, 2_000, 10_000)).unwrap();

        // 20% of 10,000 is 2,000, which exceeds the minimum floor.
        assert_eq!(result.reserve_floor, 2_000);
        assert_eq!(result.gross_surplus, 1_000);
        assert_eq!(result.deployable_amount, 1_000);
        assert_eq!(result.remaining_reserve, 2_000);
    }

    #[test]
    fn minimum_floor_wins_when_liquidity_floor_is_lower() {
        let result =
            evaluate_reserve_deployment(2_000, 10_000, policy(1_500, 1_000, 10_000)).unwrap();

        // 10% of 10,000 is only 1,000.
        assert_eq!(result.reserve_floor, 1_500);
        assert_eq!(result.gross_surplus, 500);
        assert_eq!(result.remaining_reserve, 1_500);
    }

    #[test]
    fn invalid_liquidity_floor_rate_fails_closed() {
        let result = evaluate_reserve_deployment(2_000, 10_000, policy(1_000, 10_001, 5_000));

        assert_eq!(
            result,
            Err(ReserveDeploymentError::InvalidLiquidityFloorRate),
        );
    }

    #[test]
    fn invalid_deployment_rate_fails_closed() {
        let result = evaluate_reserve_deployment(2_000, 10_000, policy(1_000, 1_000, 10_001));

        assert_eq!(
            result,
            Err(ReserveDeploymentError::InvalidSurplusDeploymentRate),
        );
    }

    #[test]
    fn u64_maximum_values_do_not_overflow() {
        let result =
            evaluate_reserve_deployment(u64::MAX, u64::MAX, policy(0, 5_000, 5_000)).unwrap();

        assert_eq!(
            result.reserve_floor,
            (u64::MAX as u128 * 5_000 / 10_000) as u64,
        );

        assert!(result.deployable_amount <= result.gross_surplus);
        assert!(result.remaining_reserve >= result.reserve_floor);
    }

    #[test]
    fn rounding_always_favours_retaining_reserve() {
        let result = evaluate_reserve_deployment(1_003, 0, policy(1_000, 0, 3_333)).unwrap();

        // floor(3 × 33.33%) = 0
        assert_eq!(result.gross_surplus, 3);
        assert_eq!(result.deployable_amount, 0);
        assert_eq!(result.remaining_reserve, 1_003);
    }

    #[test]
    fn deployment_never_breaches_floor_across_rate_matrix() {
        let reserve_balances = [0, 1, 999, 1_000, 1_001, 10_000, 1_000_000, u64::MAX];

        let liquidity_references = [0, 1, 10_000, 1_000_000, u64::MAX];

        let floor_rates = [0, 1, 1_000, 2_500, 5_000, 10_000];

        let deployment_rates = [0, 1, 2_500, 5_000, 7_500, 10_000];

        for reserve_balance in reserve_balances {
            for liquidity_reference in liquidity_references {
                for liquidity_floor_bps in floor_rates {
                    for surplus_deployment_bps in deployment_rates {
                        let result = evaluate_reserve_deployment(
                            reserve_balance,
                            liquidity_reference,
                            policy(1_000, liquidity_floor_bps, surplus_deployment_bps),
                        )
                        .unwrap();

                        assert!(result.deployable_amount <= result.gross_surplus,);

                        if reserve_balance >= result.reserve_floor {
                            assert!(result.remaining_reserve >= result.reserve_floor,);
                        } else {
                            assert_eq!(result.deployable_amount, 0);
                        }
                    }
                }
            }
        }
    }
}
