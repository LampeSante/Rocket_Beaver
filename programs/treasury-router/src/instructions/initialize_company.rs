use anchor_lang::prelude::*;

use crate::{
    constants::{COMPANY_STATE_SEED, COMPANY_STATE_VERSION, PROTOCOL_SEED},
    errors::TreasuryRouterError,
    state::{CompanyState, ProtocolState},
};

#[derive(Accounts)]
pub struct InitializeCompany<'info> {
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
        space = CompanyState::SPACE,
        seeds = [
            COMPANY_STATE_SEED,
            protocol_state.key().as_ref()
        ],
        bump
    )]
    pub company_state: Account<'info, CompanyState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeCompany>,
    recipient: Pubkey,
    period_cap: u64,
    period_duration: i64,
) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    require!(
        ctx.accounts.protocol_state.company_state == Pubkey::default(),
        TreasuryRouterError::CompanyAlreadyInitialized
    );

    require!(
        recipient != Pubkey::default(),
        TreasuryRouterError::InvalidCompanyRecipient
    );

    require!(period_cap > 0, TreasuryRouterError::InvalidCompanyPeriodCap);

    require!(
        period_duration > 0,
        TreasuryRouterError::InvalidCompanyPeriodDuration
    );

    let clock = Clock::get()?;
    let protocol_key = ctx.accounts.protocol_state.key();
    let company_state = &mut ctx.accounts.company_state;

    company_state.version = COMPANY_STATE_VERSION;
    company_state.protocol = protocol_key;
    company_state.recipient = recipient;
    company_state.period_cap = period_cap;
    company_state.spent_current_period = 0;
    company_state.lifetime_spent = 0;
    company_state.period_started_at = clock.unix_timestamp;
    company_state.period_duration = period_duration;
    company_state.enabled = true;
    company_state.bump = ctx.bumps.company_state;
    company_state.reserved = [0; 64];

    ctx.accounts.protocol_state.company_state = company_state.key();

    msg!("CompanyState initialized: {}", company_state.key());
    msg!("Company recipient: {}", company_state.recipient);
    msg!("Company period cap: {}", company_state.period_cap);
    msg!(
        "Company period duration: {} seconds",
        company_state.period_duration
    );

    Ok(())
}
