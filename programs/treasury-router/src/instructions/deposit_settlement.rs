use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{
    constants::{PROTOCOL_SEED, TREASURY_SEED},
    errors::TreasuryRouterError,
    state::{ProtocolState, TreasuryState},
};

#[derive(Accounts)]
pub struct DepositSettlement<'info> {
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
        constraint = source_token_account.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint,
        constraint = source_token_account.owner == authority.key()
            @ TreasuryRouterError::Unauthorized
    )]
    pub source_token_account: Box<Account<'info, TokenAccount>>,

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

    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<DepositSettlement>, amount: u64) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    require!(amount > 0, TreasuryRouterError::InvalidDepositAmount);

    let transfer_accounts = TransferChecked {
        from: ctx.accounts.source_token_account.to_account_info(),
        mint: ctx.accounts.settlement_mint.to_account_info(),
        to: ctx.accounts.settlement_vault.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.token_program.key(), transfer_accounts);

    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;

    msg!("Settlement-token deposit completed");
    msg!("Deposited amount in base units: {}", amount);
    msg!("Settlement mint: {}", ctx.accounts.settlement_mint.key());
    msg!(
        "Source token account: {}",
        ctx.accounts.source_token_account.key()
    );
    msg!(
        "Treasury settlement vault: {}",
        ctx.accounts.settlement_vault.key()
    );

    Ok(())
}
