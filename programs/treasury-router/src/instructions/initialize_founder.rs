use anchor_lang::prelude::*;

use crate::{
    constants::{FOUNDER_STATE_SEED, FOUNDER_STATE_VERSION, PROTOCOL_SEED},
    errors::TreasuryRouterError,
    state::{FounderState, ProtocolState},
};

#[derive(Accounts)]
pub struct InitializeFounder<'info> {
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
        space = FounderState::SPACE,
        seeds = [
            FOUNDER_STATE_SEED,
            protocol_state.key().as_ref()
        ],
        bump
    )]
    pub founder_state: Account<'info, FounderState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeFounder>,
    recipient: Pubkey,
    period_cap: u64,
    period_duration: i64,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    require!(
        ctx.accounts.protocol_state.founder_state == Pubkey::default(),
        TreasuryRouterError::FounderAlreadyInitialized
    );

    require!(
        recipient != Pubkey::default(),
        TreasuryRouterError::InvalidFounderRecipient
    );

    require!(period_cap > 0, TreasuryRouterError::InvalidFounderPeriodCap);

    require!(
        period_duration > 0,
        TreasuryRouterError::InvalidFounderPeriodDuration
    );

    let clock = Clock::get()?;
    let protocol_key = ctx.accounts.protocol_state.key();
    let founder_state = &mut ctx.accounts.founder_state;

    founder_state.version = FOUNDER_STATE_VERSION;
    founder_state.protocol = protocol_key;
    founder_state.recipient = recipient;
    founder_state.period_cap = period_cap;
    founder_state.earned_current_period = 0;
    founder_state.lifetime_earned = 0;
    founder_state.period_started_at = clock.unix_timestamp;
    founder_state.period_duration = period_duration;
    founder_state.current_tier = 0;
    founder_state.enabled = true;
    founder_state.bump = ctx.bumps.founder_state;
    founder_state.reserved = [0; 64];

    ctx.accounts.protocol_state.founder_state = founder_state.key();

    msg!("FounderState initialized: {}", founder_state.key());
    msg!("Founder recipient: {}", founder_state.recipient);
    msg!("Founder period cap: {}", founder_state.period_cap);
    msg!(
        "Founder period duration: {} seconds",
        founder_state.period_duration
    );

    Ok(())
}
