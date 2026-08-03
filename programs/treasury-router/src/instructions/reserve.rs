use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{
    constants::{
        COMPANY_STATE_SEED, EXECUTION_CONFIG_SEED, FOUNDER_STATE_SEED, PROTOCOL_CONFIG_SEED,
        PROTOCOL_SEED, RESERVE_VAULT_SEED, TREASURY_SEED,
    },
    engines::{
        execution_guard::{authorize_autonomous_release, ReleaseBucket},
        integrity_firewall::{
            evaluate_integrity, IntegrityAccountKeys, IntegrityDestinations, TokenAccountFacts,
        },
        release::process_release,
    },
    errors::TreasuryRouterError,
    events::ReserveExecutionAuthorized,
    state::{
        CompanyState, ExecutionConfig, FounderState, ProtocolConfig, ProtocolState, TreasuryState,
    },
};

#[derive(Accounts)]
pub struct AuthorizeReserveExecution<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        constraint = protocol_state.treasury_state != Pubkey::default()
            @ TreasuryRouterError::TreasuryNotInitialized,
        constraint = protocol_state.treasury_state == treasury.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol
    )]
    pub protocol_state: Box<Account<'info, ProtocolState>>,

    #[account(
        seeds = [
            PROTOCOL_CONFIG_SEED,
            protocol_state.key().as_ref()
        ],
        bump = protocol_config.bump,
        constraint = protocol_state.protocol_config == protocol_config.key()
            @ TreasuryRouterError::IntegrityFirewallViolation,
        constraint = protocol_config.protocol == protocol_state.key()
            @ TreasuryRouterError::IntegrityFirewallViolation
    )]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,

    #[account(
        seeds = [
            FOUNDER_STATE_SEED,
            protocol_state.key().as_ref()
        ],
        bump = founder_state.bump,
        constraint = protocol_state.founder_state == founder_state.key()
            @ TreasuryRouterError::IntegrityFirewallViolation,
        constraint = founder_state.protocol == protocol_state.key()
            @ TreasuryRouterError::IntegrityFirewallViolation
    )]
    pub founder_state: Box<Account<'info, FounderState>>,

    #[account(
        seeds = [
            COMPANY_STATE_SEED,
            protocol_state.key().as_ref()
        ],
        bump = company_state.bump,
        constraint = protocol_state.company_state == company_state.key()
            @ TreasuryRouterError::IntegrityFirewallViolation,
        constraint = company_state.protocol == protocol_state.key()
            @ TreasuryRouterError::IntegrityFirewallViolation
    )]
    pub company_state: Box<Account<'info, CompanyState>>,

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

    /// Canonical protocol-controlled Reserve Vault receiving reserve funding.
    #[account(
        mut,
        seeds = [
            RESERVE_VAULT_SEED,
            treasury.key().as_ref()
        ],
        bump,
        constraint = reserve_destination.key()
            == execution_config.reserve_destination
                @ TreasuryRouterError::InvalidExecutionDestination,
        constraint = reserve_destination.key() != settlement_vault.key()
            @ TreasuryRouterError::InvalidExecutionDestination,
        constraint = reserve_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint,
        constraint = reserve_destination.owner == treasury.key()
            @ TreasuryRouterError::InvalidExecutionDestination
    )]
    pub reserve_destination: Box<Account<'info, TokenAccount>>,

    /// Permanent buyback destination supplied read-only to the firewall.
    #[account(
        constraint = buyback_destination.key()
            == execution_config.buyback_destination
                @ TreasuryRouterError::IntegrityFirewallViolation,
        constraint = buyback_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::IntegrityFirewallViolation
    )]
    pub buyback_destination: Box<Account<'info, TokenAccount>>,

    /// Permanent liquidity destination supplied read-only to the firewall.
    #[account(
        constraint = liquidity_destination.key()
            == execution_config.liquidity_destination
                @ TreasuryRouterError::IntegrityFirewallViolation,
        constraint = liquidity_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::IntegrityFirewallViolation
    )]
    pub liquidity_destination: Box<Account<'info, TokenAccount>>,

    /// Permanent company destination supplied read-only to the firewall.
    #[account(
        constraint = company_destination.key()
            == execution_config.company_destination
                @ TreasuryRouterError::IntegrityFirewallViolation,
        constraint = company_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::IntegrityFirewallViolation,
        constraint = company_destination.owner == company_state.recipient
            @ TreasuryRouterError::IntegrityFirewallViolation
    )]
    pub company_destination: Box<Account<'info, TokenAccount>>,

    /// Permanent founder destination supplied read-only to the firewall.
    #[account(
        constraint = founder_destination.key()
            == execution_config.founder_destination
                @ TreasuryRouterError::IntegrityFirewallViolation,
        constraint = founder_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::IntegrityFirewallViolation,
        constraint = founder_destination.owner == founder_state.recipient
            @ TreasuryRouterError::IntegrityFirewallViolation
    )]
    pub founder_destination: Box<Account<'info, TokenAccount>>,

    /// Permissionless transaction caller and fee payer.
    /// This signer does not need to match ProtocolState.authority.
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
pub fn handler(ctx: Context<AuthorizeReserveExecution>) -> Result<()> {
    let integrity_report = evaluate_integrity(
        crate::ID,
        IntegrityAccountKeys {
            protocol: ctx.accounts.protocol_state.key(),
            protocol_config: ctx.accounts.protocol_config.key(),
            treasury: ctx.accounts.treasury.key(),
            founder: ctx.accounts.founder_state.key(),
            company: ctx.accounts.company_state.key(),
            execution_config: ctx.accounts.execution_config.key(),
        },
        &ctx.accounts.protocol_state,
        &ctx.accounts.protocol_config,
        &ctx.accounts.treasury,
        TokenAccountFacts {
            key: ctx.accounts.settlement_vault.key(),
            mint: ctx.accounts.settlement_vault.mint,
            owner: ctx.accounts.settlement_vault.owner,
        },
        &ctx.accounts.founder_state,
        &ctx.accounts.company_state,
        &ctx.accounts.execution_config,
        IntegrityDestinations {
            reserve: TokenAccountFacts {
                key: ctx.accounts.reserve_destination.key(),
                mint: ctx.accounts.reserve_destination.mint,
                owner: ctx.accounts.reserve_destination.owner,
            },
            buyback: TokenAccountFacts {
                key: ctx.accounts.buyback_destination.key(),
                mint: ctx.accounts.buyback_destination.mint,
                owner: ctx.accounts.buyback_destination.owner,
            },
            liquidity: TokenAccountFacts {
                key: ctx.accounts.liquidity_destination.key(),
                mint: ctx.accounts.liquidity_destination.mint,
                owner: ctx.accounts.liquidity_destination.owner,
            },
            company: TokenAccountFacts {
                key: ctx.accounts.company_destination.key(),
                mint: ctx.accounts.company_destination.mint,
                owner: ctx.accounts.company_destination.owner,
            },
            founder: TokenAccountFacts {
                key: ctx.accounts.founder_destination.key(),
                mint: ctx.accounts.founder_destination.mint,
                owner: ctx.accounts.founder_destination.owner,
            },
        },
    );

    require!(
        integrity_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    msg!(
        "Integrity Firewall passed with failure mask: {}",
        integrity_report.failure_mask
    );

    let authorization = authorize_autonomous_release(
        &ctx.accounts.protocol_state,
        &ctx.accounts.treasury,
        ReleaseBucket::Reserve,
    )?;

    let amount = authorization.maximum_release;

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

    let transition = process_release(&mut ctx.accounts.treasury, ReleaseBucket::Reserve, amount)?;

    let previous_pending_balance = transition.previous_pending_balance;
    let remaining_pending_balance = transition.remaining_pending_balance;
    let previous_released_balance = transition.previous_released_balance;
    let total_released_balance = transition.total_released_balance;

    emit!(ReserveExecutionAuthorized {
        protocol: ctx.accounts.protocol_state.key(),
        treasury: ctx.accounts.treasury.key(),
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

    msg!("Reserve Vault funding completed");
    msg!("Transferred amount in base units: {}", amount);
    msg!(
        "Treasury settlement vault: {}",
        ctx.accounts.settlement_vault.key()
    );
    msg!(
        "Protocol Reserve Vault: {}",
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
