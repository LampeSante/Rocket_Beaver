use anchor_lang::prelude::*;

use crate::{
    constants::{PROTOCOL_SEED, RESERVE_POLICY_SEED, RESERVE_POLICY_VERSION, TREASURY_SEED},
    errors::TreasuryRouterError,
    state::{ProtocolState, ReservePolicy, TreasuryState},
};

#[derive(Accounts)]
pub struct InitializeReservePolicy<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        has_one = authority,
        constraint = !protocol_state.paused
            @ TreasuryRouterError::ProtocolPaused,
        constraint = protocol_state.treasury_state == treasury.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol
    )]
    pub protocol_state: Box<Account<'info, ProtocolState>>,

    #[account(
        seeds = [
            TREASURY_SEED,
            protocol_state.key().as_ref()
        ],
        bump = treasury.bump,
        constraint = treasury.protocol == protocol_state.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol
    )]
    pub treasury: Box<Account<'info, TreasuryState>>,

    #[account(
        init,
        payer = authority,
        space = ReservePolicy::SPACE,
        seeds = [
            RESERVE_POLICY_SEED,
            treasury.key().as_ref()
        ],
        bump
    )]
    pub reserve_policy: Box<Account<'info, ReservePolicy>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeReservePolicy>,
    minimum_reserve_floor: u64,
    liquidity_floor_bps: u16,
    surplus_deployment_bps: u16,
    cooldown_seconds: i64,
) -> Result<()> {
    ReservePolicy::validate_parameters(
        minimum_reserve_floor,
        liquidity_floor_bps,
        surplus_deployment_bps,
        cooldown_seconds,
    )?;

    let reserve_policy = &mut ctx.accounts.reserve_policy;

    reserve_policy.version = RESERVE_POLICY_VERSION;
    reserve_policy.protocol = ctx.accounts.protocol_state.key();
    reserve_policy.treasury = ctx.accounts.treasury.key();

    reserve_policy.minimum_reserve_floor = minimum_reserve_floor;
    reserve_policy.liquidity_floor_bps = liquidity_floor_bps;
    reserve_policy.surplus_deployment_bps = surplus_deployment_bps;
    reserve_policy.cooldown_seconds = cooldown_seconds;

    reserve_policy.last_deployed_at = 0;
    reserve_policy.lifetime_deployed = 0;
    reserve_policy.bump = ctx.bumps.reserve_policy;
    reserve_policy.reserved = [0; 31];

    msg!("Immutable Reserve Policy initialized");
    msg!("Reserve Policy: {}", reserve_policy.key());
    msg!("Protocol: {}", reserve_policy.protocol);
    msg!("Treasury: {}", reserve_policy.treasury);
    msg!(
        "Minimum reserve floor: {}",
        reserve_policy.minimum_reserve_floor
    );
    msg!(
        "Liquidity floor: {} basis points",
        reserve_policy.liquidity_floor_bps
    );
    msg!(
        "Surplus deployment rate: {} basis points",
        reserve_policy.surplus_deployment_bps
    );
    msg!(
        "Deployment cooldown: {} seconds",
        reserve_policy.cooldown_seconds
    );

    Ok(())
}
