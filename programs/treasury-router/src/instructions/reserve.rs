use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{
    constants::{PROTOCOL_SEED, TREASURY_SEED},
    engines::execution_guard::{authorize_release, ReleaseBucket},
    errors::TreasuryRouterError,
    events::ReserveExecutionAuthorized,
    state::{ProtocolState, TreasuryState},
};

#[derive(Accounts)]
pub struct AuthorizeReserveExecution<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        has_one = authority,
        constraint = protocol_state.treasury_state != Pubkey::default()
            @ TreasuryRouterError::TreasuryNotInitialized,
        constraint = protocol_state.treasury_state == treasury.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol
    )]
    pub protocol_state: Box<Account<'info, ProtocolState>>,

    #[account(
        mut,
        seeds = [
            TREASURY_SEED,
            protocol_state.key().as_ref()
        ],
        bump = treasury.bump,
        constraint = treasury.protocol == protocol_state.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol,
        constraint = treasury.settlement_mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint,
        constraint = treasury.settlement_vault == settlement_vault.key()
            @ TreasuryRouterError::InvalidSettlementVault
    )]
    pub treasury: Box<Account<'info, TreasuryState>>,

    #[account(
        constraint = settlement_mint.key() == treasury.settlement_mint
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub settlement_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = settlement_vault.key() == treasury.settlement_vault
            @ TreasuryRouterError::InvalidSettlementVault,
        constraint = settlement_vault.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint,
        constraint = settlement_vault.owner == treasury.key()
            @ TreasuryRouterError::InvalidSettlementVault
    )]
    pub settlement_vault: Box<Account<'info, TokenAccount>>,

    /// Reserve-controlled settlement-token account receiving the release.
    #[account(
        mut,
        constraint = reserve_destination.key() != settlement_vault.key()
            @ TreasuryRouterError::InvalidExecutionDestination,
        constraint = reserve_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub reserve_destination: Box<Account<'info, TokenAccount>>,

    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

/// Executes a reserve release previously allocated by fee processing.
///
/// This instruction:
/// - applies the Execution Guard and Dam release limit;
/// - transfers settlement tokens from the treasury vault;
/// - decreases pending reserve accounting;
/// - increases released reserve accounting.
///
/// It does not allocate fees or modify lifetime reserve allocation.
pub fn handler(ctx: Context<AuthorizeReserveExecution>, amount: u64) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    require!(
        ctx.accounts.treasury.execution_accounting_is_valid(),
        TreasuryRouterError::AccountingInvariantViolation
    );

    let authorization = authorize_release(
        &ctx.accounts.protocol_state,
        &ctx.accounts.treasury,
        ReleaseBucket::Reserve,
        amount,
    )?;

    let protocol_key = ctx.accounts.protocol_state.key();
    let treasury_bump = [ctx.accounts.treasury.bump];

    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];

    let signer_seeds = &[treasury_signer_seeds];

    let transfer_accounts = TransferChecked {
        from: ctx.accounts.settlement_vault.to_account_info(),
        mint: ctx.accounts.settlement_mint.to_account_info(),
        to: ctx.accounts.reserve_destination.to_account_info(),
        authority: ctx.accounts.treasury.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        transfer_accounts,
        signer_seeds,
    );

    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;

    let clock = Clock::get()?;

    let previous_pending_balance = ctx.accounts.treasury.pending_reserve;
    let previous_released_balance = ctx.accounts.treasury.released_reserve;

    ctx.accounts.treasury.pending_reserve = previous_pending_balance
        .checked_sub(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    ctx.accounts.treasury.released_reserve = previous_released_balance
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        ctx.accounts.treasury.reserve_accounting_is_valid(),
        TreasuryRouterError::AccountingInvariantViolation
    );

    require!(
        ctx.accounts.treasury.execution_accounting_is_valid(),
        TreasuryRouterError::AccountingInvariantViolation
    );

    let remaining_pending_balance = ctx.accounts.treasury.pending_reserve;
    let total_released_balance = ctx.accounts.treasury.released_reserve;

    emit!(ReserveExecutionAuthorized {
        protocol: ctx.accounts.protocol_state.key(),
        treasury: ctx.accounts.treasury.key(),
        authority: ctx.accounts.authority.key(),
        authorized_amount: amount,
        previous_pending_balance,
        remaining_pending_balance,
        maximum_release: authorization.maximum_release,
        waterfall_stage: authorization.waterfall_stage.as_u8(),
        dam_level: authorization.dam_level.as_u8(),
        release_bps: authorization.release_bps,
        reserve_ratio_bps: authorization.reserve_ratio_bps,
        beaver_score: ctx.accounts.protocol_state.beaver_score,
        processing_epoch: ctx.accounts.treasury.processing_epoch,
        authorized_at: clock.unix_timestamp,
    });

    msg!("Reserve settlement-token execution completed");
    msg!("Transferred amount in base units: {}", amount);
    msg!(
        "Treasury settlement vault: {}",
        ctx.accounts.settlement_vault.key()
    );
    msg!(
        "Reserve destination: {}",
        ctx.accounts.reserve_destination.key()
    );
    msg!(
        "Previous pending reserve balance: {}",
        previous_pending_balance
    );
    msg!(
        "Remaining pending reserve balance: {}",
        remaining_pending_balance
    );
    msg!(
        "Previous released reserve balance: {}",
        previous_released_balance
    );
    msg!("Total released reserve balance: {}", total_released_balance);
    msg!(
        "Maximum release permitted by Dam: {}",
        authorization.maximum_release
    );
    msg!(
        "Waterfall stage: {} ({})",
        authorization.waterfall_stage.as_u8(),
        authorization.waterfall_stage.label()
    );
    msg!(
        "Dam level: {} ({})",
        authorization.dam_level.as_u8(),
        authorization.dam_level.label()
    );
    msg!(
        "Dam release rate: {} basis points",
        authorization.release_bps
    );
    msg!(
        "Reserve ratio: {} basis points",
        authorization.reserve_ratio_bps
    );

    Ok(())
}
