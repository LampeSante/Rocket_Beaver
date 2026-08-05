use anchor_lang::prelude::*;

use crate::{
    constants::PROTOCOL_SEED,
    errors::TreasuryRouterError,
    state::{FounderPriceState, ProtocolState, FOUNDER_PRICE_SEED},
};

#[derive(Accounts)]
pub struct SubmitFounderPrice<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        mut,
        seeds = [
            FOUNDER_PRICE_SEED,
            protocol_state.key().as_ref(),
        ],
        bump = founder_price.bump,
        constraint = founder_price.protocol == protocol_state.key()
            @ TreasuryRouterError::InvalidFounderPriceProtocol,
        constraint = founder_price.oracle_adapter_authority
            == oracle_adapter_authority.key()
            @ TreasuryRouterError::InvalidFounderOracleAdapter
    )]
    pub founder_price: Account<'info, FounderPriceState>,

    /// The locked oracle adapter CPI authority.
    pub oracle_adapter_authority: Signer<'info>,
}

pub fn handler(
    ctx: Context<SubmitFounderPrice>,
    price_feed_id: [u8; 32],
    price: i64,
    exponent: i32,
    confidence: u64,
    publish_time: i64,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    let clock = Clock::get()?;
    let founder_price = &mut ctx.accounts.founder_price;

    founder_price.validate_configuration()?;

    require!(
        price_feed_id == founder_price.price_feed_id,
        TreasuryRouterError::InvalidFounderUsdPriceFeed
    );

    require!(price > 0, TreasuryRouterError::InvalidFounderOraclePrice);

    require!(
        publish_time > 0 && publish_time <= clock.unix_timestamp,
        TreasuryRouterError::InvalidFounderOraclePublishTime
    );

    // Price updates must move forward. This blocks replaying the same
    // publication and prevents replacing current state with an older value.
    require!(
        publish_time > founder_price.publish_time,
        TreasuryRouterError::FounderOraclePriceNotNewer
    );

    let age = clock
        .unix_timestamp
        .checked_sub(publish_time)
        .ok_or(TreasuryRouterError::ArithmeticUnderflow)?;

    let maximum_age = i64::try_from(founder_price.max_price_age_seconds)
        .map_err(|_| TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        age <= maximum_age,
        TreasuryRouterError::FounderOraclePriceStale
    );

    let absolute_price =
        u128::try_from(price).map_err(|_| TreasuryRouterError::InvalidFounderOraclePrice)?;

    let confidence_bps = u128::from(confidence)
        .checked_mul(10_000)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?
        .checked_div(absolute_price)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        confidence_bps <= u128::from(founder_price.max_confidence_bps),
        TreasuryRouterError::FounderOracleConfidenceTooWide
    );

    founder_price.price = price;
    founder_price.exponent = exponent;
    founder_price.confidence = confidence;
    founder_price.publish_time = publish_time;
    founder_price.received_at = clock.unix_timestamp;
    founder_price.sequence = founder_price
        .sequence
        .checked_add(1)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    // Re-run the state validation after mutation.
    founder_price.validate_price(clock.unix_timestamp)?;

    msg!("Verified Founder settlement-token price accepted.");
    msg!("Price: {}", founder_price.price);
    msg!("Exponent: {}", founder_price.exponent);
    msg!("Confidence: {}", founder_price.confidence);
    msg!("Publish time: {}", founder_price.publish_time);
    msg!("Sequence: {}", founder_price.sequence);

    Ok(())
}
