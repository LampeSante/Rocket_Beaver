use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{
    constants::{PROTOCOL_SEED, TREASURY_SEED},
    errors::TreasuryRouterError,
    state::{
        FounderPriceState, ProtocolState, TreasuryState, FOUNDER_PRICE_SEED, FOUNDER_PRICE_VERSION,
    },
};

#[derive(Accounts)]
pub struct InitializeFounderPrice<'info> {
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
            @ TreasuryRouterError::InvalidFounderPriceSettlementMint
    )]
    pub settlement_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        space = 8 + FounderPriceState::LEN,
        seeds = [
            FOUNDER_PRICE_SEED,
            protocol_state.key().as_ref(),
        ],
        bump
    )]
    pub founder_price: Account<'info, FounderPriceState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeFounderPrice>,
    price_feed_id: [u8; 32],
    oracle_adapter_authority: Pubkey,
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
        oracle_adapter_authority != Pubkey::default(),
        TreasuryRouterError::InvalidFounderOracleAdapter
    );

    require!(
        max_price_age_seconds > 0,
        TreasuryRouterError::InvalidFounderOracleMaximumAge
    );

    require!(
        max_confidence_bps > 0 && max_confidence_bps <= 10_000,
        TreasuryRouterError::InvalidFounderOracleConfidenceLimit
    );

    let founder_price = &mut ctx.accounts.founder_price;

    founder_price.version = FOUNDER_PRICE_VERSION;
    founder_price.protocol = ctx.accounts.protocol_state.key();
    founder_price.settlement_mint = ctx.accounts.settlement_mint.key();
    founder_price.price_feed_id = price_feed_id;
    founder_price.oracle_adapter_authority = oracle_adapter_authority;

    founder_price.price = 0;
    founder_price.exponent = 0;
    founder_price.confidence = 0;
    founder_price.publish_time = 0;
    founder_price.received_at = 0;
    founder_price.sequence = 0;

    founder_price.max_price_age_seconds = max_price_age_seconds;
    founder_price.max_confidence_bps = max_confidence_bps;
    founder_price.enabled = true;
    founder_price.bump = ctx.bumps.founder_price;
    founder_price.reserved = [0u8; 64];

    founder_price.validate_configuration()?;

    msg!("Founder price receipt initialized.");
    msg!(
        "Oracle adapter authority: {}",
        founder_price.oracle_adapter_authority
    );
    msg!(
        "Maximum price age: {} seconds",
        founder_price.max_price_age_seconds
    );
    msg!(
        "Maximum confidence width: {} basis points",
        founder_price.max_confidence_bps
    );

    Ok(())
}
