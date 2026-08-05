use anchor_lang::prelude::*;

use crate::errors::TreasuryRouterError;

/// One year using the protocol's fixed 365-day policy.
pub const FOUNDER_ANNUAL_PERIOD_SECONDS: i64 = 31_536_000;

/// Founder compensation is capped at US$3,000,000 per annual period.
///
/// USD values use six decimal places:
/// 1 USD = 1_000_000 USD micro-units.
pub const FOUNDER_ANNUAL_CAP_USD_E6: u64 = 3_000_000_000_000;

/// Version for the migration-safe Founder USD-cap account.
pub const FOUNDER_USD_CAP_VERSION: u8 = 1;

/// Canonical PDA seed.
///
/// PDA:
/// ["founder-usd-cap", protocol_state]
pub const FOUNDER_USD_CAP_SEED: &[u8] = b"founder-usd-cap";

/// Maximum denominator for confidence-width validation.
pub const CONFIDENCE_BPS_DENOMINATOR: u128 = 10_000;

#[account]
#[derive(Debug)]
pub struct FounderUsdCapState {
    /// Account-layout version.
    pub version: u8,

    /// Canonical ProtocolState account.
    pub protocol: Pubkey,

    /// Settlement token whose USD price must be supplied.
    pub settlement_mint: Pubkey,

    /// Pyth price feed identifier for settlement-token/USD.
    pub price_feed_id: [u8; 32],

    /// Fixed annual cap in USD micro-units.
    pub annual_cap_usd_e6: u64,

    /// USD micro-units earned during the current annual period.
    pub earned_current_period_usd_e6: u64,

    /// Lifetime USD micro-units attributed to Founder compensation.
    pub lifetime_earned_usd_e6: u128,

    /// Beginning of the current fixed 365-day period.
    pub period_started_at: i64,

    /// Must equal 31,536,000.
    pub period_duration: i64,

    /// Maximum accepted oracle age in seconds.
    pub max_price_age_seconds: u64,

    /// Maximum confidence interval as basis points of absolute price.
    pub max_confidence_bps: u16,

    /// Feature gate. Final production configuration should remain enabled.
    pub enabled: bool,

    /// Canonical PDA bump.
    pub bump: u8,

    /// Reserved for forward-compatible non-breaking additions.
    pub reserved: [u8; 64],
}

impl FounderUsdCapState {
    pub const LEN: usize = 1 +     // version
        32 +    // protocol
        32 +    // settlement_mint
        32 +    // price_feed_id
        8 +     // annual_cap_usd_e6
        8 +     // earned_current_period_usd_e6
        16 +    // lifetime_earned_usd_e6
        8 +     // period_started_at
        8 +     // period_duration
        8 +     // max_price_age_seconds
        2 +     // max_confidence_bps
        1 +     // enabled
        1 +     // bump
        64; // reserved

    pub fn validate_configuration(&self) -> Result<()> {
        require!(
            self.version == FOUNDER_USD_CAP_VERSION,
            TreasuryRouterError::InvalidFounderUsdCapVersion
        );

        require!(
            self.protocol != Pubkey::default(),
            TreasuryRouterError::InvalidFounderUsdCapProtocol
        );

        require!(
            self.settlement_mint != Pubkey::default(),
            TreasuryRouterError::InvalidFounderUsdCapSettlementMint
        );

        require!(
            self.price_feed_id != [0u8; 32],
            TreasuryRouterError::InvalidFounderUsdPriceFeed
        );

        require!(
            self.annual_cap_usd_e6 == FOUNDER_ANNUAL_CAP_USD_E6,
            TreasuryRouterError::InvalidFounderAnnualUsdCap
        );

        require!(
            self.period_duration == FOUNDER_ANNUAL_PERIOD_SECONDS,
            TreasuryRouterError::InvalidFounderAnnualPeriod
        );

        require!(
            self.max_price_age_seconds > 0,
            TreasuryRouterError::InvalidFounderOracleMaximumAge
        );

        require!(
            self.max_confidence_bps > 0 && self.max_confidence_bps <= 10_000,
            TreasuryRouterError::InvalidFounderOracleConfidenceLimit
        );

        require!(self.enabled, TreasuryRouterError::FounderUsdCapDisabled);

        Ok(())
    }

    /// Rolls the annual period forward deterministically.
    ///
    /// Multiple elapsed periods are skipped without granting overlapping
    /// periods. The new period begins at the nearest period boundary.
    pub fn roll_period_if_elapsed(&mut self, now: i64) -> Result<()> {
        require!(
            now >= self.period_started_at,
            TreasuryRouterError::InvalidFounderAnnualPeriodTimestamp
        );

        let elapsed = now
            .checked_sub(self.period_started_at)
            .ok_or(TreasuryRouterError::ArithmeticUnderflow)?;

        if elapsed < self.period_duration {
            return Ok(());
        }

        let periods_elapsed = elapsed
            .checked_div(self.period_duration)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        let period_advance = periods_elapsed
            .checked_mul(self.period_duration)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        self.period_started_at = self
            .period_started_at
            .checked_add(period_advance)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        self.earned_current_period_usd_e6 = 0;

        Ok(())
    }

    pub fn remaining_usd_e6(&self) -> Result<u64> {
        self.annual_cap_usd_e6
            .checked_sub(self.earned_current_period_usd_e6)
            .ok_or(TreasuryRouterError::FounderAnnualUsdAccountingViolation.into())
    }
}
