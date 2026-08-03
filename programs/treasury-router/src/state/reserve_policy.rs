use anchor_lang::prelude::*;

use crate::{constants::RESERVE_POLICY_BPS_DENOMINATOR, errors::TreasuryRouterError};

/// Immutable parameters governing future Reserve Vault surplus deployment.
///
/// No instruction is provided to update this account after initialization.
#[account]
pub struct ReservePolicy {
    /// State layout version.
    pub version: u16,

    /// Root protocol linked to this policy.
    pub protocol: Pubkey,

    /// Treasury whose Reserve Vault is governed by this policy.
    pub treasury: Pubkey,

    /// Absolute settlement-token floor that can never be deployed.
    pub minimum_reserve_floor: u64,

    /// Additional floor calculated from a future liquidity reference.
    pub liquidity_floor_bps: u16,

    /// Fraction of current surplus deployable per Spillway execution.
    pub surplus_deployment_bps: u16,

    /// Minimum seconds between successful Spillway executions.
    pub cooldown_seconds: i64,

    /// Timestamp of the latest successful deployment.
    ///
    /// Zero means that no deployment has occurred.
    pub last_deployed_at: i64,

    /// Lifetime value deployed through the Spillway.
    pub lifetime_deployed: u64,

    /// Canonical Reserve Policy PDA bump.
    pub bump: u8,

    /// Reserved space for compatible extensions.
    pub reserved: [u8; 31],
}

impl ReservePolicy {
    pub const SPACE: usize = 8 +  // Anchor discriminator
        2 +                       // version
        32 +                      // protocol
        32 +                      // treasury
        8 +                       // minimum_reserve_floor
        2 +                       // liquidity_floor_bps
        2 +                       // surplus_deployment_bps
        8 +                       // cooldown_seconds
        8 +                       // last_deployed_at
        8 +                       // lifetime_deployed
        1 +                       // bump
        31; // reserved

    pub fn validate_parameters(
        minimum_reserve_floor: u64,
        liquidity_floor_bps: u16,
        surplus_deployment_bps: u16,
        cooldown_seconds: i64,
    ) -> Result<()> {
        require!(
            minimum_reserve_floor > 0,
            TreasuryRouterError::InvalidReserveMinimumFloor
        );

        require!(
            liquidity_floor_bps <= RESERVE_POLICY_BPS_DENOMINATOR,
            TreasuryRouterError::InvalidReserveLiquidityFloorRate
        );

        require!(
            surplus_deployment_bps > 0 && surplus_deployment_bps <= RESERVE_POLICY_BPS_DENOMINATOR,
            TreasuryRouterError::InvalidReserveDeploymentRate
        );

        require!(
            cooldown_seconds > 0,
            TreasuryRouterError::InvalidReserveDeploymentCooldown
        );

        Ok(())
    }

    pub fn cooldown_has_elapsed(&self, current_timestamp: i64) -> Result<bool> {
        if self.last_deployed_at == 0 {
            return Ok(true);
        }

        let next_allowed = self
            .last_deployed_at
            .checked_add(self.cooldown_seconds)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        Ok(current_timestamp >= next_allowed)
    }

    pub fn record_deployment(&mut self, amount: u64, current_timestamp: i64) -> Result<()> {
        require!(
            amount > 0,
            TreasuryRouterError::InvalidReserveDeploymentAmount
        );

        require!(
            self.cooldown_has_elapsed(current_timestamp)?,
            TreasuryRouterError::ReserveDeploymentCooldownActive
        );

        let updated_lifetime = self
            .lifetime_deployed
            .checked_add(amount)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        self.lifetime_deployed = updated_lifetime;
        self.last_deployed_at = current_timestamp;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> ReservePolicy {
        ReservePolicy {
            version: 1,
            protocol: Pubkey::new_unique(),
            treasury: Pubkey::new_unique(),
            minimum_reserve_floor: 1_000,
            liquidity_floor_bps: 2_000,
            surplus_deployment_bps: 2_500,
            cooldown_seconds: 86_400,
            last_deployed_at: 0,
            lifetime_deployed: 0,
            bump: 255,
            reserved: [0; 31],
        }
    }

    #[test]
    fn valid_parameters_are_accepted() {
        assert!(ReservePolicy::validate_parameters(1, 2_000, 2_500, 86_400,).is_ok());
    }

    #[test]
    fn zero_minimum_floor_is_rejected() {
        assert!(ReservePolicy::validate_parameters(0, 2_000, 2_500, 86_400,).is_err());
    }

    #[test]
    fn liquidity_floor_above_one_hundred_percent_is_rejected() {
        assert!(ReservePolicy::validate_parameters(1, 10_001, 2_500, 86_400,).is_err());
    }

    #[test]
    fn zero_deployment_rate_is_rejected() {
        assert!(ReservePolicy::validate_parameters(1, 2_000, 0, 86_400,).is_err());
    }

    #[test]
    fn deployment_rate_above_one_hundred_percent_is_rejected() {
        assert!(ReservePolicy::validate_parameters(1, 2_000, 10_001, 86_400,).is_err());
    }

    #[test]
    fn non_positive_cooldown_is_rejected() {
        assert!(ReservePolicy::validate_parameters(1, 2_000, 2_500, 0,).is_err());

        assert!(ReservePolicy::validate_parameters(1, 2_000, 2_500, -1,).is_err());
    }

    #[test]
    fn first_deployment_is_immediately_eligible() {
        let reserve_policy = policy();

        assert!(reserve_policy.cooldown_has_elapsed(1).unwrap());
    }

    #[test]
    fn cooldown_blocks_early_redeployment() {
        let mut reserve_policy = policy();
        reserve_policy.last_deployed_at = 100;

        assert!(!reserve_policy.cooldown_has_elapsed(86_499).unwrap());
        assert!(reserve_policy.cooldown_has_elapsed(86_500).unwrap());
    }

    #[test]
    fn successful_deployment_updates_timestamp_and_lifetime() {
        let mut reserve_policy = policy();

        reserve_policy.record_deployment(250, 1_000).unwrap();

        assert_eq!(reserve_policy.last_deployed_at, 1_000);
        assert_eq!(reserve_policy.lifetime_deployed, 250);
    }

    #[test]
    fn failed_cooldown_check_does_not_mutate_policy() {
        let mut reserve_policy = policy();
        reserve_policy.last_deployed_at = 1_000;
        reserve_policy.lifetime_deployed = 500;

        let before_timestamp = reserve_policy.last_deployed_at;
        let before_lifetime = reserve_policy.lifetime_deployed;

        assert!(reserve_policy.record_deployment(100, 2_000).is_err());

        assert_eq!(reserve_policy.last_deployed_at, before_timestamp);
        assert_eq!(reserve_policy.lifetime_deployed, before_lifetime);
    }

    #[test]
    fn lifetime_overflow_fails_without_mutation() {
        let mut reserve_policy = policy();
        reserve_policy.lifetime_deployed = u64::MAX;

        assert!(reserve_policy.record_deployment(1, 1_000).is_err());

        assert_eq!(reserve_policy.lifetime_deployed, u64::MAX);
        assert_eq!(reserve_policy.last_deployed_at, 0);
    }
}
