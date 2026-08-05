use anchor_lang::prelude::*;

use crate::{errors::TreasuryRouterError, state::FounderUsdCapState};

/// USD accounting precision.
pub const USD_E6_SCALE: u128 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FounderUsdCapAllocation {
    /// Founder token amount permitted by the remaining annual USD cap.
    pub founder_token_amount: u64,

    /// Founder token amount redirected to Liquidity Growth.
    pub liquidity_overflow_token_amount: u64,

    /// USD micro-units charged to the Founder annual cap.
    pub founder_usd_e6: u64,

    /// Oracle price used in the calculation.
    pub oracle_price: i64,

    /// Oracle exponent used in the calculation.
    pub oracle_exponent: i32,
}

fn checked_pow10(exponent: u32) -> Result<u128> {
    10u128
        .checked_pow(exponent)
        .ok_or(TreasuryRouterError::ArithmeticOverflow.into())
}

/// Converts raw settlement-token units into USD micro-units.
///
/// Pyth represents price as:
///
/// real_price = price * 10^exponent
///
/// Token raw units represent:
///
/// tokens = raw_amount / 10^token_decimals
///
/// This function floors the USD result, which ensures the protocol never
/// records more USD than the token amount actually represents.
pub fn token_amount_to_usd_e6(
    raw_token_amount: u64,
    token_decimals: u8,
    price: i64,
    exponent: i32,
) -> Result<u64> {
    require!(price > 0, TreasuryRouterError::InvalidFounderOraclePrice);

    let numerator = u128::from(raw_token_amount)
        .checked_mul(
            u128::try_from(price).map_err(|_| TreasuryRouterError::InvalidFounderOraclePrice)?,
        )
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?
        .checked_mul(USD_E6_SCALE)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let token_scale = checked_pow10(u32::from(token_decimals))?;

    let usd_e6 = if exponent >= 0 {
        numerator
            .checked_mul(checked_pow10(exponent as u32)?)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?
            .checked_div(token_scale)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?
    } else {
        numerator
            .checked_div(
                token_scale
                    .checked_mul(checked_pow10(exponent.unsigned_abs())?)
                    .ok_or(TreasuryRouterError::ArithmeticOverflow)?,
            )
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?
    };

    u64::try_from(usd_e6).map_err(|_| TreasuryRouterError::ArithmeticOverflow.into())
}

/// Converts a remaining USD allowance into the maximum raw settlement-token
/// units that may be allocated to Founder compensation.
///
/// This floors the token result so the allocation cannot exceed the cap.
pub fn usd_e6_to_token_amount(
    usd_e6: u64,
    token_decimals: u8,
    price: i64,
    exponent: i32,
) -> Result<u64> {
    require!(price > 0, TreasuryRouterError::InvalidFounderOraclePrice);

    let token_scale = checked_pow10(u32::from(token_decimals))?;

    let positive_price =
        u128::try_from(price).map_err(|_| TreasuryRouterError::InvalidFounderOraclePrice)?;

    let numerator = u128::from(usd_e6)
        .checked_mul(token_scale)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let raw_tokens = if exponent >= 0 {
        numerator
            .checked_div(
                positive_price
                    .checked_mul(USD_E6_SCALE)
                    .ok_or(TreasuryRouterError::ArithmeticOverflow)?
                    .checked_mul(checked_pow10(exponent as u32)?)
                    .ok_or(TreasuryRouterError::ArithmeticOverflow)?,
            )
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?
    } else {
        numerator
            .checked_mul(checked_pow10(exponent.unsigned_abs())?)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?
            .checked_div(
                positive_price
                    .checked_mul(USD_E6_SCALE)
                    .ok_or(TreasuryRouterError::ArithmeticOverflow)?,
            )
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?
    };

    u64::try_from(raw_tokens).map_err(|_| TreasuryRouterError::ArithmeticOverflow.into())
}

/// Applies the true US$3,000,000 annual Founder cap.
///
/// Every token unit that cannot be allocated to Founder compensation is
/// deterministically redirected to Liquidity Growth.
pub fn apply_founder_usd_cap(
    state: &mut FounderUsdCapState,
    requested_founder_tokens: u64,
    settlement_token_decimals: u8,
    oracle_price: i64,
    oracle_exponent: i32,
    now: i64,
) -> Result<FounderUsdCapAllocation> {
    state.validate_configuration()?;
    state.roll_period_if_elapsed(now)?;

    if requested_founder_tokens == 0 {
        return Ok(FounderUsdCapAllocation {
            founder_token_amount: 0,
            liquidity_overflow_token_amount: 0,
            founder_usd_e6: 0,
            oracle_price,
            oracle_exponent,
        });
    }

    let requested_usd_e6 = token_amount_to_usd_e6(
        requested_founder_tokens,
        settlement_token_decimals,
        oracle_price,
        oracle_exponent,
    )?;

    let remaining_usd_e6 = state.remaining_usd_e6()?;

    let founder_token_amount = if requested_usd_e6 <= remaining_usd_e6 {
        requested_founder_tokens
    } else {
        usd_e6_to_token_amount(
            remaining_usd_e6,
            settlement_token_decimals,
            oracle_price,
            oracle_exponent,
        )?
        .min(requested_founder_tokens)
    };

    let founder_usd_e6 = token_amount_to_usd_e6(
        founder_token_amount,
        settlement_token_decimals,
        oracle_price,
        oracle_exponent,
    )?;

    require!(
        founder_usd_e6 <= remaining_usd_e6,
        TreasuryRouterError::FounderAnnualUsdCapExceeded
    );

    let liquidity_overflow_token_amount = requested_founder_tokens
        .checked_sub(founder_token_amount)
        .ok_or(TreasuryRouterError::ArithmeticUnderflow)?;

    state.earned_current_period_usd_e6 = state
        .earned_current_period_usd_e6
        .checked_add(founder_usd_e6)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    state.lifetime_earned_usd_e6 = state
        .lifetime_earned_usd_e6
        .checked_add(u128::from(founder_usd_e6))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        state.earned_current_period_usd_e6 <= state.annual_cap_usd_e6,
        TreasuryRouterError::FounderAnnualUsdCapExceeded
    );

    Ok(FounderUsdCapAllocation {
        founder_token_amount,
        liquidity_overflow_token_amount,
        founder_usd_e6,
        oracle_price,
        oracle_exponent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{
        FOUNDER_ANNUAL_CAP_USD_E6, FOUNDER_ANNUAL_PERIOD_SECONDS, FOUNDER_USD_CAP_VERSION,
    };

    fn state() -> FounderUsdCapState {
        FounderUsdCapState {
            version: FOUNDER_USD_CAP_VERSION,
            protocol: Pubkey::new_unique(),
            settlement_mint: Pubkey::new_unique(),
            price_feed_id: [7u8; 32],
            annual_cap_usd_e6: FOUNDER_ANNUAL_CAP_USD_E6,
            earned_current_period_usd_e6: 0,
            lifetime_earned_usd_e6: 0,
            period_started_at: 1_000,
            period_duration: FOUNDER_ANNUAL_PERIOD_SECONDS,
            max_price_age_seconds: 60,
            max_confidence_bps: 100,
            enabled: true,
            bump: 255,
            reserved: [0u8; 64],
        }
    }

    #[test]
    fn one_token_at_one_dollar_equals_one_dollar() {
        assert_eq!(
            token_amount_to_usd_e6(1_000_000, 6, 100_000_000, -8,).unwrap(),
            1_000_000
        );
    }

    #[test]
    fn fixed_founder_amount_below_cap_is_fully_allocated() {
        let mut cap = state();

        let result =
            apply_founder_usd_cap(&mut cap, 100_000_000, 6, 100_000_000, -8, 1_100).unwrap();

        assert_eq!(result.founder_token_amount, 100_000_000);
        assert_eq!(result.liquidity_overflow_token_amount, 0);
        assert_eq!(result.founder_usd_e6, 100_000_000);
    }

    #[test]
    fn exact_cap_is_allowed() {
        let mut cap = state();

        let result =
            apply_founder_usd_cap(&mut cap, 3_000_000_000_000, 6, 100_000_000, -8, 1_100).unwrap();

        assert_eq!(result.founder_usd_e6, FOUNDER_ANNUAL_CAP_USD_E6);
        assert_eq!(result.liquidity_overflow_token_amount, 0);
    }

    #[test]
    fn excess_is_redirected_to_liquidity() {
        let mut cap = state();
        cap.earned_current_period_usd_e6 = FOUNDER_ANNUAL_CAP_USD_E6 - 10_000_000;

        let result =
            apply_founder_usd_cap(&mut cap, 25_000_000, 6, 100_000_000, -8, 1_100).unwrap();

        assert_eq!(result.founder_token_amount, 10_000_000);
        assert_eq!(result.liquidity_overflow_token_amount, 15_000_000);
        assert_eq!(cap.earned_current_period_usd_e6, FOUNDER_ANNUAL_CAP_USD_E6);
    }

    #[test]
    fn fully_capped_founder_receives_zero() {
        let mut cap = state();
        cap.earned_current_period_usd_e6 = FOUNDER_ANNUAL_CAP_USD_E6;

        let result =
            apply_founder_usd_cap(&mut cap, 25_000_000, 6, 100_000_000, -8, 1_100).unwrap();

        assert_eq!(result.founder_token_amount, 0);
        assert_eq!(result.liquidity_overflow_token_amount, 25_000_000);
    }

    #[test]
    fn annual_period_resets_after_365_days() {
        let mut cap = state();
        cap.earned_current_period_usd_e6 = FOUNDER_ANNUAL_CAP_USD_E6;

        let now = cap.period_started_at + FOUNDER_ANNUAL_PERIOD_SECONDS;

        let result = apply_founder_usd_cap(&mut cap, 1_000_000, 6, 100_000_000, -8, now).unwrap();

        assert_eq!(result.founder_token_amount, 1_000_000);
        assert_eq!(cap.earned_current_period_usd_e6, 1_000_000);
        assert_eq!(cap.period_started_at, now);
    }

    #[test]
    fn non_positive_oracle_price_fails_closed() {
        let mut cap = state();

        assert!(apply_founder_usd_cap(&mut cap, 1_000_000, 6, 0, -8, 1_100,).is_err());
    }

    #[test]
    fn conversion_handles_non_one_dollar_price() {
        // One token at $2.50.
        assert_eq!(
            token_amount_to_usd_e6(1_000_000_000, 9, 250_000_000, -8,).unwrap(),
            2_500_000
        );
    }
}
