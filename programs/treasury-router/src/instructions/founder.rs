use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{
    constants::{EXECUTION_CONFIG_SEED, FOUNDER_STATE_SEED, PROTOCOL_SEED, TREASURY_SEED},
    engines::execution_guard::{authorize_autonomous_release, ReleaseBucket},
    engines::release::process_release,
    errors::TreasuryRouterError,
    events::FounderExecutionAuthorized,
    state::{ExecutionConfig, FounderState, ProtocolState, TreasuryState},
};

#[derive(Accounts)]
pub struct AuthorizeFounderExecution<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        constraint = protocol_state.treasury_state != Pubkey::default()
            @ TreasuryRouterError::TreasuryNotInitialized,
        constraint = protocol_state.treasury_state == treasury.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol,
        constraint = protocol_state.founder_state == founder_state.key()
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
        seeds = [
            FOUNDER_STATE_SEED,
            protocol_state.key().as_ref()
        ],
        bump = founder_state.bump,
        constraint = founder_state.protocol == protocol_state.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol
    )]
    pub founder_state: Box<Account<'info, FounderState>>,

    #[account(
        seeds = [
            EXECUTION_CONFIG_SEED,
            protocol_state.key().as_ref()
        ],
        bump = execution_config.bump,
        constraint = execution_config.protocol_state == protocol_state.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol,
        constraint = execution_config.settlement_mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub execution_config: Box<Account<'info, ExecutionConfig>>,

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

    /// Settlement-token account controlled by the configured founder recipient.
    #[account(
        mut,
        constraint = founder_destination.key()
            == execution_config.founder_destination
                @ TreasuryRouterError::InvalidExecutionDestination,
        constraint = founder_destination.key() != settlement_vault.key()
            @ TreasuryRouterError::InvalidExecutionDestination,
        constraint = founder_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint,
        constraint = founder_destination.owner == founder_state.recipient
            @ TreasuryRouterError::InvalidExecutionDestination
    )]
    pub founder_destination: Box<Account<'info, TokenAccount>>,

    /// Permissionless transaction caller and fee payer.
    /// This signer does not need to match ProtocolState.authority.
    pub token_program: Program<'info, Token>,
}

/// Releases founder funds already allocated during fee processing.
///
/// Founder period-cap enforcement occurs while fees are processed and credited
/// to `pending_founder`. This instruction does not allocate new founder
/// earnings or modify founder compensation counters.
///
/// The settlement-token transfer and treasury accounting update are atomic.
pub fn handler(ctx: Context<AuthorizeFounderExecution>) -> Result<()> {
    let authorization = authorize_autonomous_release(
        &ctx.accounts.protocol_state,
        &ctx.accounts.treasury,
        ReleaseBucket::Founder,
    )?;

    let amount = authorization.maximum_release;

    let protocol_key = ctx.accounts.protocol_state.key();
    let treasury_bump = [ctx.accounts.treasury.bump];

    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];

    let signer_seeds = &[treasury_signer_seeds];

    let transfer_accounts = TransferChecked {
        from: ctx.accounts.settlement_vault.to_account_info(),
        mint: ctx.accounts.settlement_mint.to_account_info(),
        to: ctx.accounts.founder_destination.to_account_info(),
        authority: ctx.accounts.treasury.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        transfer_accounts,
        signer_seeds,
    );

    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;

    let clock = Clock::get()?;

    let transition = process_release(&mut ctx.accounts.treasury, ReleaseBucket::Founder, amount)?;

    let previous_pending_balance = transition.previous_pending_balance;
    let remaining_pending_balance = transition.remaining_pending_balance;
    let previous_released_balance = transition.previous_released_balance;
    let total_released_balance = transition.total_released_balance;

    emit!(FounderExecutionAuthorized {
        protocol: ctx.accounts.protocol_state.key(),
        treasury: ctx.accounts.treasury.key(),
        founder_state: ctx.accounts.founder_state.key(),
        recipient: ctx.accounts.founder_state.recipient,
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
        founder_period_cap: ctx.accounts.founder_state.period_cap,
        founder_earned_current_period: ctx.accounts.founder_state.earned_current_period,
        founder_lifetime_earned: ctx.accounts.founder_state.lifetime_earned,
        authorized_at: clock.unix_timestamp,
    });

    msg!("Founder settlement-token execution completed");
    msg!(
        "Founder recipient: {}",
        ctx.accounts.founder_state.recipient
    );
    msg!(
        "Founder destination: {}",
        ctx.accounts.founder_destination.key()
    );
    msg!("Transferred amount in base units: {}", amount);
    msg!(
        "Previous pending founder balance: {}",
        previous_pending_balance
    );
    msg!(
        "Remaining pending founder balance: {}",
        remaining_pending_balance
    );
    msg!(
        "Previous released founder balance: {}",
        previous_released_balance
    );
    msg!("Total released founder balance: {}", total_released_balance);
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
    msg!(
        "Founder earned during current period: {}",
        ctx.accounts.founder_state.earned_current_period
    );
    msg!(
        "Founder period cap: {}",
        ctx.accounts.founder_state.period_cap
    );

    Ok(())
}
