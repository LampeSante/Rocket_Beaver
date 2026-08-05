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

    pub fn initialize_execution_config(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
        instructions::initialize_execution_config::handler(ctx)
    }

    pub fn initialize_reserve_policy(
        ctx: Context<InitializeReservePolicy>,
        minimum_reserve_floor: u64,
        liquidity_floor_bps: u16,
        surplus_deployment_bps: u16,
        cooldown_seconds: i64,
    ) -> Result<()> {
        instructions::initialize_reserve_policy::handler(
            ctx,
            minimum_reserve_floor,
            liquidity_floor_bps,
            surplus_deployment_bps,
            cooldown_seconds,
        )
    }

    pub fn deposit_settlement(ctx: Context<DepositSettlement>, amount: u64) -> Result<()> {
        instructions::deposit_settlement::handler(ctx, amount)
    }

    pub fn process_fees(ctx: Context<ProcessFees>) -> Result<()> {
        instructions::process_fees::handler(ctx)
    }

    pub fn authorize_reserve_execution(ctx: Context<AuthorizeReserveExecution>) -> Result<()> {
        instructions::reserve::handler(ctx)
    }

    pub fn spillway_release(ctx: Context<SpillwayRelease>) -> Result<()> {
        instructions::spillway_release::handler(ctx)
    }

    pub fn authorize_buyback_execution(ctx: Context<AuthorizeBuybackExecution>) -> Result<()> {
        instructions::buyback::handler(ctx)
    }

    pub fn authorize_liquidity_execution(ctx: Context<AuthorizeLiquidityExecution>) -> Result<()> {
        instructions::liquidity::handler(ctx)
    }

    pub fn authorize_founder_execution(ctx: Context<AuthorizeFounderExecution>) -> Result<()> {
        instructions::founder::handler(ctx)
    }

    pub fn authorize_company_execution(ctx: Context<AuthorizeCompanyExecution>) -> Result<()> {
        instructions::company::handler(ctx)
    }

    pub fn initialize_founder_usd_cap(
        ctx: Context<InitializeFounderUsdCap>,
        price_feed_id: [u8; 32],
        max_price_age_seconds: u64,
        max_confidence_bps: u16,
    ) -> Result<()> {
        instructions::initialize_founder_usd_cap::handler(
            ctx,
            price_feed_id,
            max_price_age_seconds,
            max_confidence_bps,
        )
    }

    pub fn initialize_founder_price(
        ctx: Context<InitializeFounderPrice>,
        price_feed_id: [u8; 32],
        oracle_adapter_authority: Pubkey,
        max_price_age_seconds: u64,
        max_confidence_bps: u16,
    ) -> Result<()> {
        instructions::initialize_founder_price::handler(
            ctx,
            price_feed_id,
            oracle_adapter_authority,
            max_price_age_seconds,
            max_confidence_bps,
        )
    }

    pub fn submit_founder_price(
        ctx: Context<SubmitFounderPrice>,
        price_feed_id: [u8; 32],
        price: i64,
        exponent: i32,
        confidence: u64,
        publish_time: i64,
    ) -> Result<()> {
        instructions::submit_founder_price::handler(
            ctx,
            price_feed_id,
            price,
            exponent,
            confidence,
            publish_time,
        )
    }
}
