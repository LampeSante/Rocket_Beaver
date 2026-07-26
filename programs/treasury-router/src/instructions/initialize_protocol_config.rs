use anchor_lang::prelude::*;

use crate::{
    constants::{
        BPS_DENOMINATOR, INITIAL_BUYBACK_BURN_BPS, INITIAL_COMPANY_BPS, INITIAL_FOUNDER_BPS,
        INITIAL_LIQUIDITY_BPS, INITIAL_RESERVE_BPS, PROTOCOL_CONFIG_SEED, PROTOCOL_CONFIG_VERSION,
        PROTOCOL_SEED,
    },
    errors::TreasuryRouterError,
    state::{ProtocolConfig, ProtocolState},
};

#[derive(Accounts)]
pub struct InitializeProtocolConfig<'info> {
    #[account(
        mut,
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        has_one = authority
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        init,
        payer = authority,
        space = ProtocolConfig::SPACE,
        seeds = [
            PROTOCOL_CONFIG_SEED,
            protocol_state.key().as_ref()
        ],
        bump
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeProtocolConfig>) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    require!(
        ctx.accounts.protocol_state.protocol_config == Pubkey::default(),
        TreasuryRouterError::ProtocolConfigAlreadyInitialized
    );

    let total_bps = INITIAL_RESERVE_BPS
        .checked_add(INITIAL_BUYBACK_BURN_BPS)
        .and_then(|value| value.checked_add(INITIAL_LIQUIDITY_BPS))
        .and_then(|value| value.checked_add(INITIAL_COMPANY_BPS))
        .and_then(|value| value.checked_add(INITIAL_FOUNDER_BPS))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        total_bps == BPS_DENOMINATOR,
        TreasuryRouterError::InvalidAllocationConfiguration
    );

    let clock = Clock::get()?;
    let protocol_key = ctx.accounts.protocol_state.key();
    let protocol_config = &mut ctx.accounts.protocol_config;

    protocol_config.version = PROTOCOL_CONFIG_VERSION;
    protocol_config.protocol = protocol_key;

    protocol_config.reserve_bps = INITIAL_RESERVE_BPS;
    protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
    protocol_config.liquidity_bps = INITIAL_LIQUIDITY_BPS;
    protocol_config.company_bps = INITIAL_COMPANY_BPS;
    protocol_config.founder_bps = INITIAL_FOUNDER_BPS;

    protocol_config.updates_enabled = false;
    protocol_config.bump = ctx.bumps.protocol_config;
    protocol_config.updated_at = clock.unix_timestamp;
    protocol_config.reserved = [0; 64];

    ctx.accounts.protocol_state.protocol_config = protocol_config.key();

    msg!("ProtocolConfig initialized: {}", protocol_config.key());
    msg!(
        "Allocations: reserve={} buyback={} liquidity={} company={} founder={}",
        protocol_config.reserve_bps,
        protocol_config.buyback_burn_bps,
        protocol_config.liquidity_bps,
        protocol_config.company_bps,
        protocol_config.founder_bps
    );

    Ok(())
}
