use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{
    constants::{PROTOCOL_SEED, TREASURY_SEED},
    errors::TreasuryRouterError,
    state::{
        FounderUsdCapState, ProtocolState, TreasuryState, FOUNDER_ANNUAL_CAP_USD_E6,
        FOUNDER_ANNUAL_PERIOD_SECONDS, FOUNDER_USD_CAP_SEED, FOUNDER_USD_CAP_VERSION,
    },
};

#[derive(Accounts)]
pub struct InitializeFounderUsdCap<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        has_one = authority @ TreasuryRouterError::Unauthorized
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        seeds = [
            TREASURY_SEED,
            protocol_state.key().as_ref(),
        ],
        bump = treasury_state.bump,
        constraint = treasury_state.protocol == protocol_state.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol
    )]
    pub treasury_state: Account<'info, TreasuryState>,

    #[account(
        constraint = settlement_mint.key() == treasury_state.settlement_mint
            @ TreasuryRouterError::InvalidFounderUsdCapSettlementMint
    )]
    pub settlement_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        space = 8 + FounderUsdCapState::LEN,
        seeds = [
            FOUNDER_USD_CAP_SEED,
            protocol_state.key().as_ref(),
        ],
        bump
    )]
    pub founder_usd_cap: Account<'info, FounderUsdCapState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeFounderUsdCap>,
    price_feed_id: [u8; 32],
    max_price_age_seconds: u64,
    max_confidence_bps: u16,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    require!(
        price_feed_id != [0u8; 32],
        TreasuryRouterError::InvalidFounderUsdPriceFeed
    );

    require!(
        max_price_age_seconds > 0,
        TreasuryRouterError::InvalidFounderOracleMaximumAge
    );

    require!(
        max_confidence_bps > 0 && max_confidence_bps <= 10_000,
        TreasuryRouterError::InvalidFounderOracleConfidenceLimit
    );

    let clock = Clock::get()?;
    let founder_usd_cap = &mut ctx.accounts.founder_usd_cap;

    founder_usd_cap.version = FOUNDER_USD_CAP_VERSION;
    founder_usd_cap.protocol = ctx.accounts.protocol_state.key();
    founder_usd_cap.settlement_mint = ctx.accounts.settlement_mint.key();
    founder_usd_cap.price_feed_id = price_feed_id;

    // These policy values are compiled constants, not caller inputs.
    founder_usd_cap.annual_cap_usd_e6 = FOUNDER_ANNUAL_CAP_USD_E6;
    founder_usd_cap.earned_current_period_usd_e6 = 0;
    founder_usd_cap.lifetime_earned_usd_e6 = 0;
    founder_usd_cap.period_started_at = clock.unix_timestamp;
    founder_usd_cap.period_duration = FOUNDER_ANNUAL_PERIOD_SECONDS;

    founder_usd_cap.max_price_age_seconds = max_price_age_seconds;
    founder_usd_cap.max_confidence_bps = max_confidence_bps;
    founder_usd_cap.enabled = true;
    founder_usd_cap.bump = ctx.bumps.founder_usd_cap;
    founder_usd_cap.reserved = [0u8; 64];

    founder_usd_cap.validate_configuration()?;

    msg!("Founder USD-cap account initialized.");
    msg!(
        "Annual Founder cap: {} USD micro-units",
        founder_usd_cap.annual_cap_usd_e6
    );
    msg!("Annual period: {} seconds", founder_usd_cap.period_duration);
    msg!(
        "Oracle maximum age: {} seconds",
        founder_usd_cap.max_price_age_seconds
    );
    msg!(
        "Oracle confidence limit: {} basis points",
        founder_usd_cap.max_confidence_bps
    );

    Ok(())
}
