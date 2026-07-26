use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    constants::{
        INITIAL_WATERFALL_STAGE, PROTOCOL_SEED, TREASURY_SEED, TREASURY_VAULT_SEED,
        TREASURY_VERSION,
    },
    errors::TreasuryRouterError,
    state::{ProtocolState, TreasuryState},
};

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
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
        space = TreasuryState::SPACE,
        seeds = [
            TREASURY_SEED,
            protocol_state.key().as_ref()
        ],
        bump
    )]
    pub treasury_state: Account<'info, TreasuryState>,

    /// Existing SPL mint used to account for and hold protocol fees.
    pub settlement_mint: Account<'info, Mint>,

    /// Treasury-owned SPL token vault for the settlement mint.
    #[account(
        init,
        payer = authority,
        seeds = [
            TREASURY_VAULT_SEED,
            treasury_state.key().as_ref()
        ],
        bump,
        token::mint = settlement_mint,
        token::authority = treasury_state
    )]
    pub settlement_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeTreasury>) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    require!(
        ctx.accounts.protocol_state.treasury_state == Pubkey::default(),
        TreasuryRouterError::TreasuryAlreadyInitialized
    );

    let treasury_state = &mut ctx.accounts.treasury_state;

    treasury_state.version = TREASURY_VERSION;
    treasury_state.protocol = ctx.accounts.protocol_state.key();
    treasury_state.settlement_mint = ctx.accounts.settlement_mint.key();
    treasury_state.settlement_vault = ctx.accounts.settlement_vault.key();

    treasury_state.total_fees_received = 0;
    treasury_state.total_fees_allocated = 0;

    treasury_state.pending_reserve = 0;
    treasury_state.pending_buyback_burn = 0;
    treasury_state.pending_liquidity = 0;
    treasury_state.pending_company = 0;
    treasury_state.pending_founder = 0;

    treasury_state.lifetime_reserve = 0;
    treasury_state.lifetime_buyback_burn = 0;
    treasury_state.lifetime_liquidity = 0;
    treasury_state.lifetime_company = 0;
    treasury_state.lifetime_founder = 0;

    treasury_state.released_reserve = 0;
    treasury_state.released_buyback_burn = 0;
    treasury_state.released_liquidity = 0;
    treasury_state.released_company = 0;
    treasury_state.released_founder = 0;

    treasury_state.last_processed_at = 0;
    treasury_state.processing_epoch = 0;
    treasury_state.waterfall_stage = INITIAL_WATERFALL_STAGE;
    treasury_state.buybacks_paused = false;
    treasury_state.bump = ctx.bumps.treasury_state;
    treasury_state.reserved = [0; 24];

    require!(
        treasury_state.execution_accounting_is_valid(),
        TreasuryRouterError::AccountingInvariantViolation
    );

    ctx.accounts.protocol_state.treasury_state = treasury_state.key();

    msg!("TreasuryState initialized: {}", treasury_state.key());
    msg!("Settlement mint: {}", ctx.accounts.settlement_mint.key());
    msg!("Settlement vault: {}", ctx.accounts.settlement_vault.key());
    msg!("Settlement vault authority: {}", treasury_state.key());

    Ok(())
}
