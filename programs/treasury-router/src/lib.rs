use anchor_lang::prelude::*;

pub mod constants;
pub mod engines;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("5mEQEoksSyTMJifw88UWGna81bY6ghWPJqHNFfDWcYD3");

#[program]
pub mod treasury_router {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn initialize_protocol_config(ctx: Context<InitializeProtocolConfig>) -> Result<()> {
        instructions::initialize_protocol_config::handler(ctx)
    }

    pub fn initialize_founder(
        ctx: Context<InitializeFounder>,
        recipient: Pubkey,
        period_cap: u64,
        period_duration: i64,
    ) -> Result<()> {
        instructions::initialize_founder::handler(ctx, recipient, period_cap, period_duration)
    }

    pub fn initialize_company(
        ctx: Context<InitializeCompany>,
        recipient: Pubkey,
        period_cap: u64,
        period_duration: i64,
    ) -> Result<()> {
        instructions::initialize_company::handler(ctx, recipient, period_cap, period_duration)
    }

    pub fn initialize_treasury(ctx: Context<InitializeTreasury>) -> Result<()> {
        instructions::initialize_treasury::handler(ctx)
    }

    pub fn deposit_settlement(ctx: Context<DepositSettlement>, amount: u64) -> Result<()> {
        instructions::deposit_settlement::handler(ctx, amount)
    }

    pub fn process_fees(ctx: Context<ProcessFees>, amount: u64) -> Result<()> {
        instructions::process_fees::handler(ctx, amount)
    }

    pub fn authorize_reserve_execution(
        ctx: Context<AuthorizeReserveExecution>,
        amount: u64,
    ) -> Result<()> {
        instructions::reserve::handler(ctx, amount)
    }

    pub fn authorize_buyback_execution(
        ctx: Context<AuthorizeBuybackExecution>,
        amount: u64,
    ) -> Result<()> {
        instructions::buyback::handler(ctx, amount)
    }

    pub fn authorize_liquidity_execution(
        ctx: Context<AuthorizeLiquidityExecution>,
        amount: u64,
    ) -> Result<()> {
        instructions::liquidity::handler(ctx, amount)
    }

    pub fn authorize_founder_execution(
        ctx: Context<AuthorizeFounderExecution>,
        amount: u64,
    ) -> Result<()> {
        instructions::founder::handler(ctx, amount)
    }

    pub fn authorize_company_execution(
        ctx: Context<AuthorizeCompanyExecution>,
        amount: u64,
    ) -> Result<()> {
        instructions::company::handler(ctx, amount)
    }
}
