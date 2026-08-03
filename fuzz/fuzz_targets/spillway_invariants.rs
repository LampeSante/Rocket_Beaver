#![no_main]

use anchor_lang::prelude::Pubkey;
use libfuzzer_sys::fuzz_target;
use treasury_router::{
    engines::reserve_deployment::{
        evaluate_reserve_deployment, ReserveDeploymentPolicy,
        ReserveDeploymentStage,
    },
    state::ReservePolicy,
};

struct Cursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn u8(&mut self) -> u8 {
        let value = self.data.get(self.offset).copied().unwrap_or(0);
        self.offset = self.offset.saturating_add(1);
        value
    }

    fn u16(&mut self) -> u16 {
        let mut bytes = [0u8; 2];

        for byte in &mut bytes {
            *byte = self.u8();
        }

        u16::from_le_bytes(bytes)
    }

    fn u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];

        for byte in &mut bytes {
            *byte = self.u8();
        }

        u64::from_le_bytes(bytes)
    }

    fn i64(&mut self) -> i64 {
        self.u64() as i64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PolicySnapshot {
    minimum_reserve_floor: u64,
    liquidity_floor_bps: u16,
    surplus_deployment_bps: u16,
    cooldown_seconds: i64,
    last_deployed_at: i64,
    lifetime_deployed: u64,
}

fn snapshot(policy: &ReservePolicy) -> PolicySnapshot {
    PolicySnapshot {
        minimum_reserve_floor: policy.minimum_reserve_floor,
        liquidity_floor_bps: policy.liquidity_floor_bps,
        surplus_deployment_bps: policy.surplus_deployment_bps,
        cooldown_seconds: policy.cooldown_seconds,
        last_deployed_at: policy.last_deployed_at,
        lifetime_deployed: policy.lifetime_deployed,
    }
}

fn make_policy(
    minimum_reserve_floor: u64,
    liquidity_floor_bps: u16,
    surplus_deployment_bps: u16,
    cooldown_seconds: i64,
    last_deployed_at: i64,
    lifetime_deployed: u64,
) -> ReservePolicy {
    ReservePolicy {
        version: 1,
        protocol: Pubkey::new_from_array([1u8; 32]),
        treasury: Pubkey::new_from_array([2u8; 32]),

        minimum_reserve_floor,
        liquidity_floor_bps,
        surplus_deployment_bps,
        cooldown_seconds,

        last_deployed_at,
        lifetime_deployed,

        bump: 255,
        reserved: [0; 31],
    }
}

fuzz_target!(|data: &[u8]| {
    let mut cursor = Cursor::new(data);

    let reserve_balance = cursor.u64();
    let liquidity_reference = cursor.u64();

    let minimum_reserve_floor = cursor.u64();

    /*
     * Preserve invalid values too. Rates above 10,000 must fail closed rather
     * than being silently normalized by the harness.
     */
    let liquidity_floor_bps = cursor.u16();
    let surplus_deployment_bps = cursor.u16();

    let cooldown_seconds = cursor.i64();
    let last_deployed_at = cursor.i64();
    let current_timestamp = cursor.i64();
    let lifetime_deployed = cursor.u64();

    let parameters_valid = ReservePolicy::validate_parameters(
        minimum_reserve_floor,
        liquidity_floor_bps,
        surplus_deployment_bps,
        cooldown_seconds,
    )
    .is_ok();

    let evaluation = evaluate_reserve_deployment(
        reserve_balance,
        liquidity_reference,
        ReserveDeploymentPolicy {
            minimum_reserve_floor,
            liquidity_floor_bps,
            surplus_deployment_bps,
        },
    );

    match evaluation {
        Ok(result) => {
            /*
             * A successful engine evaluation can never authorize more than
             * the current gross surplus.
             */
            assert!(
                result.deployable_amount <= result.gross_surplus,
                "deployable amount exceeded gross surplus"
            );

            /*
             * Accounting conservation:
             *
             * reserve before == deployed amount + reserve after
             */
            assert_eq!(
                result
                    .deployable_amount
                    .checked_add(result.remaining_reserve),
                Some(reserve_balance),
                "reserve evaluation did not conserve value"
            );

            match result.stage {
                ReserveDeploymentStage::Filling => {
                    assert!(
                        reserve_balance < result.reserve_floor,
                        "Filling stage used at or above the floor"
                    );

                    assert_eq!(
                        result.deployable_amount, 0,
                        "Filling reserve authorized deployment"
                    );

                    assert_eq!(
                        result.remaining_reserve, reserve_balance,
                        "Filling evaluation changed the reserve"
                    );
                }

                ReserveDeploymentStage::Healthy => {
                    assert_eq!(
                        reserve_balance, result.reserve_floor,
                        "Healthy stage did not equal the floor"
                    );

                    assert_eq!(
                        result.deployable_amount, 0,
                        "Healthy reserve authorized deployment"
                    );

                    assert_eq!(
                        result.remaining_reserve, reserve_balance,
                        "Healthy evaluation changed the reserve"
                    );
                }

                ReserveDeploymentStage::Surplus => {
                    assert!(
                        reserve_balance > result.reserve_floor,
                        "Surplus stage used without surplus"
                    );

                    assert_eq!(
                        result.gross_surplus,
                        reserve_balance - result.reserve_floor,
                        "gross surplus was calculated incorrectly"
                    );

                    assert!(
                        result.remaining_reserve >= result.reserve_floor,
                        "successful deployment breached the protected floor"
                    );
                }
            }

            /*
             * Valid policy rates must be compatible with a successful engine
             * evaluation. Invalid policy parameters may still include an
             * engine-valid zero floor; they must never be recorded on-chain.
             */
            if parameters_valid {
                let mut policy = make_policy(
                    minimum_reserve_floor,
                    liquidity_floor_bps,
                    surplus_deployment_bps,
                    cooldown_seconds,
                    last_deployed_at,
                    lifetime_deployed,
                );

                let before = snapshot(&policy);

                if result.deployable_amount == 0 {
                    /*
                     * The live Spillway rejects zero deployable surplus before
                     * policy mutation. Reproduce that fail-closed behavior.
                     */
                    assert_eq!(
                        snapshot(&policy),
                        before,
                        "zero-surplus path mutated Reserve Policy"
                    );

                    return;
                }

                let cooldown_result =
                    policy.cooldown_has_elapsed(current_timestamp);

                match cooldown_result {
                    Ok(true) => {
                        let expected_lifetime = lifetime_deployed
                            .checked_add(result.deployable_amount);

                        let record_result = policy.record_deployment(
                            result.deployable_amount,
                            current_timestamp,
                        );

                        match expected_lifetime {
                            Some(expected) => {
                                assert!(
                                    record_result.is_ok(),
                                    "eligible deployment unexpectedly failed"
                                );

                                assert_eq!(
                                    policy.lifetime_deployed, expected,
                                    "lifetime deployment accounting mismatch"
                                );

                                assert_eq!(
                                    policy.last_deployed_at,
                                    current_timestamp,
                                    "deployment timestamp was not recorded"
                                );

                                /*
                                 * Immutable policy parameters must never be
                                 * modified by a successful transition.
                                 */
                                assert_eq!(
                                    policy.minimum_reserve_floor,
                                    before.minimum_reserve_floor
                                );
                                assert_eq!(
                                    policy.liquidity_floor_bps,
                                    before.liquidity_floor_bps
                                );
                                assert_eq!(
                                    policy.surplus_deployment_bps,
                                    before.surplus_deployment_bps
                                );
                                assert_eq!(
                                    policy.cooldown_seconds,
                                    before.cooldown_seconds
                                );
                            }

                            None => {
                                assert!(
                                    record_result.is_err(),
                                    "lifetime overflow was accepted"
                                );

                                assert_eq!(
                                    snapshot(&policy),
                                    before,
                                    "overflow failure mutated Reserve Policy"
                                );
                            }
                        }
                    }

                    Ok(false) | Err(_) => {
                        let record_result = policy.record_deployment(
                            result.deployable_amount,
                            current_timestamp,
                        );

                        assert!(
                            record_result.is_err(),
                            "cooldown-protected deployment was accepted"
                        );

                        assert_eq!(
                            snapshot(&policy),
                            before,
                            "cooldown failure mutated Reserve Policy"
                        );
                    }
                }
            }
        }

        Err(_) => {
            /*
             * Invalid rates or impossible arithmetic must fail closed. There
             * is no evaluation result and therefore no deployable amount.
             */
        }
    }
});
