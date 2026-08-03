use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{
    constants::{
        EXECUTION_CONFIG_SEED, PROTOCOL_SEED, RESERVE_POLICY_SEED, RESERVE_VAULT_SEED,
        TREASURY_SEED,
    },
    engines::reserve_deployment::{evaluate_reserve_deployment, ReserveDeploymentPolicy},
    errors::TreasuryRouterError,
    events::SpillwayReleaseExecuted,
    state::{ExecutionConfig, ProtocolState, ReservePolicy, TreasuryState},
};

#[derive(Accounts)]
pub struct SpillwayRelease<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
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
            @ TreasuryRouterError::InvalidTreasuryProtocol,
        constraint = treasury.settlement_mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub treasury: Box<Account<'info, TreasuryState>>,

    #[account(
        seeds = [
            EXECUTION_CONFIG_SEED,
            protocol_state.key().as_ref()
        ],
        bump = execution_config.bump,
        constraint = execution_config.protocol_state
            == protocol_state.key()
                @ TreasuryRouterError::InvalidTreasuryProtocol,
        constraint = execution_config.settlement_mint
            == settlement_mint.key()
                @ TreasuryRouterError::InvalidSettlementMint,
        constraint = execution_config.reserve_destination
            == reserve_vault.key()
                @ TreasuryRouterError::InvalidSettlementVault,
        constraint = execution_config.liquidity_destination
            == liquidity_destination.key()
                @ TreasuryRouterError::InvalidSpillwayDestination
    )]
    pub execution_config: Box<Account<'info, ExecutionConfig>>,

    #[account(
        mut,
        seeds = [
            RESERVE_POLICY_SEED,
            treasury.key().as_ref()
        ],
        bump = reserve_policy.bump,
        constraint = reserve_policy.protocol
            == protocol_state.key()
                @ TreasuryRouterError::InvalidReservePolicyLinkage,
        constraint = reserve_policy.treasury
            == treasury.key()
                @ TreasuryRouterError::InvalidReservePolicyLinkage
    )]
    pub reserve_policy: Box<Account<'info, ReservePolicy>>,

    #[account(
        constraint = settlement_mint.key()
            == treasury.settlement_mint
                @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub settlement_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            RESERVE_VAULT_SEED,
            treasury.key().as_ref()
        ],
        bump,
        constraint = reserve_vault.key()
            == execution_config.reserve_destination
                @ TreasuryRouterError::InvalidSettlementVault,
        constraint = reserve_vault.mint
            == settlement_mint.key()
                @ TreasuryRouterError::InvalidSettlementMint,
        constraint = reserve_vault.owner
            == treasury.key()
                @ TreasuryRouterError::InvalidSettlementVault
    )]
    pub reserve_vault: Box<Account<'info, TokenAccount>>,

    /// Immutable liquidity destination configured during protocol setup.
    ///
    /// Its current settlement-token balance is used as the liquidity
    /// reference for calculating the dynamic Reserve floor.
    #[account(
        mut,
        constraint = liquidity_destination.key()
            == execution_config.liquidity_destination
                @ TreasuryRouterError::InvalidSpillwayDestination,
        constraint = liquidity_destination.key()
            != reserve_vault.key()
                @ TreasuryRouterError::InvalidSpillwayDestination,
        constraint = liquidity_destination.mint
            == settlement_mint.key()
                @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub liquidity_destination: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

/// Deploys only the Reserve Engine-approved surplus to the immutable
/// Liquidity Growth destination.
///
/// This instruction is permissionless. No caller signer controls the amount,
/// source, destination, Reserve floor, deployment rate or cooldown.
pub fn handler(ctx: Context<SpillwayRelease>) -> Result<()> {
    let clock = Clock::get()?;

    require!(
        ctx.accounts
            .reserve_policy
            .cooldown_has_elapsed(clock.unix_timestamp)?,
        TreasuryRouterError::ReserveDeploymentCooldownActive
    );

    let reserve_balance_before = ctx.accounts.reserve_vault.amount;
    let liquidity_reference = ctx.accounts.liquidity_destination.amount;

    let evaluation = evaluate_reserve_deployment(
        reserve_balance_before,
        liquidity_reference,
        ReserveDeploymentPolicy {
            minimum_reserve_floor: ctx.accounts.reserve_policy.minimum_reserve_floor,
            liquidity_floor_bps: ctx.accounts.reserve_policy.liquidity_floor_bps,
            surplus_deployment_bps: ctx.accounts.reserve_policy.surplus_deployment_bps,
        },
    )
    .map_err(|_| error!(TreasuryRouterError::ReserveDeploymentEvaluationFailed))?;

    require!(
        evaluation.deployable_amount > 0,
        TreasuryRouterError::NoDeployableReserveSurplus
    );

    require!(
        evaluation.remaining_reserve >= evaluation.reserve_floor,
        TreasuryRouterError::ReserveFloorViolation
    );

    let protocol_key = ctx.accounts.protocol_state.key();
    let treasury_bump = [ctx.accounts.treasury.bump];

    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];

    let signer_seeds = &[treasury_signer_seeds];

    let transfer_accounts = TransferChecked {
        from: ctx.accounts.reserve_vault.to_account_info(),
        mint: ctx.accounts.settlement_mint.to_account_info(),
        to: ctx.accounts.liquidity_destination.to_account_info(),
        authority: ctx.accounts.treasury.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        transfer_accounts,
        signer_seeds,
    );

    token::transfer_checked(
        cpi_context,
        evaluation.deployable_amount,
        ctx.accounts.settlement_mint.decimals,
    )?;

    ctx.accounts
        .reserve_policy
        .record_deployment(evaluation.deployable_amount, clock.unix_timestamp)?;

    emit!(SpillwayReleaseExecuted {
        protocol: ctx.accounts.protocol_state.key(),
        treasury: ctx.accounts.treasury.key(),
        reserve_policy: ctx.accounts.reserve_policy.key(),
        reserve_vault: ctx.accounts.reserve_vault.key(),
        destination: ctx.accounts.liquidity_destination.key(),

        reserve_balance_before,
        protected_floor: evaluation.reserve_floor,
        gross_surplus: evaluation.gross_surplus,
        deployed_amount: evaluation.deployable_amount,
        reserve_balance_after: evaluation.remaining_reserve,

        liquidity_reference,
        deployment_bps: ctx.accounts.reserve_policy.surplus_deployment_bps,

        lifetime_deployed: ctx.accounts.reserve_policy.lifetime_deployed,
        executed_at: clock.unix_timestamp,
    });

    msg!("Permissionless Reserve Spillway release completed");
    msg!("Reserve balance before: {}", reserve_balance_before);
    msg!("Protected Reserve floor: {}", evaluation.reserve_floor);
    msg!("Gross Reserve surplus: {}", evaluation.gross_surplus);
    msg!("Spillway deployed amount: {}", evaluation.deployable_amount);
    msg!("Reserve balance after: {}", evaluation.remaining_reserve);
    msg!(
        "Liquidity destination: {}",
        ctx.accounts.liquidity_destination.key()
    );

    Ok(())
}
