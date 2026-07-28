use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::{
    constants::{
        COMPANY_STATE_SEED, EXECUTION_CONFIG_SEED, EXECUTION_CONFIG_VERSION, FOUNDER_STATE_SEED,
        PROTOCOL_SEED, TREASURY_SEED,
    },
    errors::TreasuryRouterError,
    state::{CompanyState, ExecutionConfig, FounderState, ProtocolState, TreasuryState},
};

#[derive(Accounts)]
pub struct InitializeExecutionConfig<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        has_one = authority,
        constraint = !protocol_state.paused
            @ TreasuryRouterError::ProtocolPaused,
        constraint = protocol_state.treasury_state == treasury.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol,
        constraint = protocol_state.founder_state == founder_state.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol,
        constraint = protocol_state.company_state == company_state.key()
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
            COMPANY_STATE_SEED,
            protocol_state.key().as_ref()
        ],
        bump = company_state.bump,
        constraint = company_state.protocol == protocol_state.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol
    )]
    pub company_state: Box<Account<'info, CompanyState>>,

    #[account(
        constraint = settlement_mint.key() == treasury.settlement_mint
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub settlement_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = settlement_vault.key() == treasury.settlement_vault
            @ TreasuryRouterError::InvalidSettlementVault,
        constraint = settlement_vault.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint,
        constraint = settlement_vault.owner == treasury.key()
            @ TreasuryRouterError::InvalidSettlementVault
    )]
    pub settlement_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = reserve_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub reserve_destination: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = buyback_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub buyback_destination: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = liquidity_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub liquidity_destination: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = company_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint,
        constraint = company_destination.owner == company_state.recipient
            @ TreasuryRouterError::InvalidExecutionDestination
    )]
    pub company_destination: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = founder_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint,
        constraint = founder_destination.owner == founder_state.recipient
            @ TreasuryRouterError::InvalidExecutionDestination
    )]
    pub founder_destination: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = authority,
        space = 8 + ExecutionConfig::LEN,
        seeds = [
            EXECUTION_CONFIG_SEED,
            protocol_state.key().as_ref()
        ],
        bump
    )]
    pub execution_config: Box<Account<'info, ExecutionConfig>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
    let settlement_vault = ctx.accounts.settlement_vault.key();

    let reserve_destination = ctx.accounts.reserve_destination.key();
    let buyback_destination = ctx.accounts.buyback_destination.key();
    let liquidity_destination = ctx.accounts.liquidity_destination.key();
    let company_destination = ctx.accounts.company_destination.key();
    let founder_destination = ctx.accounts.founder_destination.key();

    // No allocation destination may point back to the treasury vault.
    require_keys_neq!(
        reserve_destination,
        settlement_vault,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        buyback_destination,
        settlement_vault,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        liquidity_destination,
        settlement_vault,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        company_destination,
        settlement_vault,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        founder_destination,
        settlement_vault,
        TreasuryRouterError::InvalidExecutionDestination
    );

    // Every allocation bucket must have its own permanently isolated account.
    require_keys_neq!(
        reserve_destination,
        buyback_destination,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        reserve_destination,
        liquidity_destination,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        reserve_destination,
        company_destination,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        reserve_destination,
        founder_destination,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        buyback_destination,
        liquidity_destination,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        buyback_destination,
        company_destination,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        buyback_destination,
        founder_destination,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        liquidity_destination,
        company_destination,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        liquidity_destination,
        founder_destination,
        TreasuryRouterError::InvalidExecutionDestination
    );
    require_keys_neq!(
        company_destination,
        founder_destination,
        TreasuryRouterError::InvalidExecutionDestination
    );

    let execution_config = &mut ctx.accounts.execution_config;

    execution_config.protocol_state = ctx.accounts.protocol_state.key();
    execution_config.settlement_mint = ctx.accounts.settlement_mint.key();

    execution_config.reserve_destination = reserve_destination;
    execution_config.buyback_destination = buyback_destination;
    execution_config.liquidity_destination = liquidity_destination;
    execution_config.company_destination = company_destination;
    execution_config.founder_destination = founder_destination;

    execution_config.version = EXECUTION_CONFIG_VERSION;
    execution_config.bump = ctx.bumps.execution_config;

    msg!(
        "Immutable ExecutionConfig initialized: {}",
        execution_config.key()
    );
    msg!("ProtocolState: {}", execution_config.protocol_state);
    msg!("Settlement mint: {}", execution_config.settlement_mint);
    msg!("Reserve destination: {}", reserve_destination);
    msg!("Buyback destination: {}", buyback_destination);
    msg!("Liquidity destination: {}", liquidity_destination);
    msg!("Company destination: {}", company_destination);
    msg!("Founder destination: {}", founder_destination);

    Ok(())
}
