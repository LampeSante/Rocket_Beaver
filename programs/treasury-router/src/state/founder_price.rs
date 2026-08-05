use anchor_lang::prelude::*;

use crate::errors::TreasuryRouterError;

pub const FOUNDER_PRICE_SEED: &[u8] = b"founder-price";
pub const FOUNDER_PRICE_VERSION: u8 = 1;

#[account]
#[derive(Debug)]
pub struct FounderPriceState {
    /// Layout version.
    pub version: u8,

    /// Canonical ProtocolState.
    pub protocol: Pubkey,

    /// Settlement token being priced in USD.
    pub settlement_mint: Pubkey,

    /// Pyth feed identifier verified by the adapter.
    pub price_feed_id: [u8; 32],

    /// Only this CPI signer may post verified prices.
    pub oracle_adapter_authority: Pubkey,

    /// Pyth integer price.
    pub price: i64,

    /// Pyth exponent.
    pub exponent: i32,

    /// Pyth confidence interval.
    pub confidence: u64,

    /// Pyth publication timestamp.
    pub publish_time: i64,

    /// Solana timestamp when RBVR accepted the update.
    pub received_at: i64,

    /// Monotonic update sequence.
    pub sequence: u64,

    /// Maximum accepted price age during fee processing.
    pub max_price_age_seconds: u64,

    /// Maximum confidence width relative to absolute price.
    pub max_confidence_bps: u16,

    pub enabled: bool,
    pub bump: u8,
    pub reserved: [u8; 64],
}

impl FounderPriceState {
    pub const LEN: usize = 1 +     // version
        32 +    // protocol
        32 +    // settlement_mint
        32 +    // price_feed_id
        32 +    // oracle_adapter_authority
        8 +     // price
        4 +     // exponent
        8 +     // confidence
        8 +     // publish_time
        8 +     // received_at
        8 +     // sequence
        8 +     // max_price_age_seconds
        2 +     // max_confidence_bps
        1 +     // enabled
        1 +     // bump
        64; // reserved

    pub fn validate_configuration(&self) -> Result<()> {
        require!(
            self.version == FOUNDER_PRICE_VERSION,
            TreasuryRouterError::InvalidFounderPriceVersion
        );

        require!(
            self.protocol != Pubkey::default(),
            TreasuryRouterError::InvalidFounderPriceProtocol
        );

        require!(
            self.settlement_mint != Pubkey::default(),
            TreasuryRouterError::InvalidFounderPriceSettlementMint
        );

        require!(
            self.price_feed_id != [0u8; 32],
            TreasuryRouterError::InvalidFounderUsdPriceFeed
        );

        require!(
            self.oracle_adapter_authority != Pubkey::default(),
            TreasuryRouterError::InvalidFounderOracleAdapter
        );

        require!(
            self.max_price_age_seconds > 0,
            TreasuryRouterError::InvalidFounderOracleMaximumAge
        );

        require!(
            self.max_confidence_bps > 0 && self.max_confidence_bps <= 10_000,
            TreasuryRouterError::InvalidFounderOracleConfidenceLimit
        );

        require!(self.enabled, TreasuryRouterError::FounderPriceDisabled);

        Ok(())
    }

    pub fn validate_price(&self, now: i64) -> Result<()> {
        self.validate_configuration()?;

        require!(
            self.price > 0,
            TreasuryRouterError::InvalidFounderOraclePrice
        );

        require!(
            self.publish_time > 0 && self.publish_time <= now,
            TreasuryRouterError::InvalidFounderOraclePublishTime
        );

        let age = now
            .checked_sub(self.publish_time)
            .ok_or(TreasuryRouterError::ArithmeticUnderflow)?;

        let maximum_age = i64::try_from(self.max_price_age_seconds)
            .map_err(|_| TreasuryRouterError::ArithmeticOverflow)?;

        require!(
            age <= maximum_age,
            TreasuryRouterError::FounderOraclePriceStale
        );

        let absolute_price = u128::try_from(self.price)
            .map_err(|_| TreasuryRouterError::InvalidFounderOraclePrice)?;

        let confidence_bps = u128::from(self.confidence)
            .checked_mul(10_000)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?
            .checked_div(absolute_price)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        require!(
            confidence_bps <= u128::from(self.max_confidence_bps),
            TreasuryRouterError::FounderOracleConfidenceTooWide
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_price_state() -> FounderPriceState {
        FounderPriceState {
            version: FOUNDER_PRICE_VERSION,
            protocol: Pubkey::new_unique(),
            settlement_mint: Pubkey::new_unique(),
            price_feed_id: [9u8; 32],
            oracle_adapter_authority: Pubkey::new_unique(),
            price: 100_000_000,
            exponent: -8,
            confidence: 100_000,
            publish_time: 1_000,
            received_at: 1_001,
            sequence: 1,
            max_price_age_seconds: 60,
            max_confidence_bps: 100,
            enabled: true,
            bump: 255,
            reserved: [0u8; 64],
        }
    }

    #[test]
    fn valid_price_passes() {
        let state = valid_price_state();
        assert!(state.validate_price(1_030).is_ok());
    }

    #[test]
    fn stale_price_fails() {
        let state = valid_price_state();
        assert!(state.validate_price(1_061).is_err());
    }

    #[test]
    fn future_price_fails() {
        let state = valid_price_state();
        assert!(state.validate_price(999).is_err());
    }

    #[test]
    fn non_positive_price_fails() {
        let mut state = valid_price_state();
        state.price = 0;

        assert!(state.validate_price(1_030).is_err());
    }

    #[test]
    fn excessive_confidence_fails() {
        let mut state = valid_price_state();

        // 2% confidence width against a 1% configured maximum.
        state.confidence = 2_000_000;

        assert!(state.validate_price(1_030).is_err());
    }

    #[test]
    fn disabled_price_state_fails() {
        let mut state = valid_price_state();
        state.enabled = false;

        assert!(state.validate_price(1_030).is_err());
    }
}
