use anchor_lang::prelude::*;

use crate::{
    constants::{INITIAL_DAM_LEVEL, PROTOCOL_SEED, PROTOCOL_VERSION},
    state::ProtocolState,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = ProtocolState::SPACE,
        seeds = [PROTOCOL_SEED],
        bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let protocol_state = &mut ctx.accounts.protocol_state;
    let clock = Clock::get()?;

    protocol_state.version = PROTOCOL_VERSION;
    protocol_state.authority = ctx.accounts.authority.key();

    protocol_state.protocol_config = Pubkey::default();
    protocol_state.treasury_state = Pubkey::default();
    protocol_state.reserve_state = Pubkey::default();
    protocol_state.liquidity_state = Pubkey::default();
    protocol_state.founder_state = Pubkey::default();
    protocol_state.company_state = Pubkey::default();
    protocol_state.buyback_state = Pubkey::default();

    protocol_state.beaver_score = 0;
    protocol_state.dam_level = INITIAL_DAM_LEVEL;
    protocol_state.paused = false;
    protocol_state.bump = ctx.bumps.protocol_state;
    protocol_state.initialized_at = clock.unix_timestamp;
    protocol_state.reserved = [0; 64];

    msg!(
        "Rocket Beaver ProtocolState initialized: {}",
        protocol_state.key()
    );

    Ok(())
}
