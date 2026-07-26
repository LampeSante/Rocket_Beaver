use anchor_lang::prelude::*;

use crate::{
    constants::{
        BPS_DENOMINATOR, COMPANY_STATE_SEED, FOUNDER_STATE_SEED, PROTOCOL_CONFIG_SEED,
        PROTOCOL_SEED, TREASURY_SEED,
    },
    engines::{beaver_score, buyback, company, dam, founder, liquidity, reserve, waterfall},
    errors::TreasuryRouterError,
    state::{CompanyState, FounderState, ProtocolConfig, ProtocolState, TreasuryState},
};

#[derive(Accounts)]
pub struct ProcessFees<'info> {
    #[account(
        mut,
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        has_one = authority,
        has_one = protocol_config,
        constraint = protocol_state.treasury_state == treasury.key(),
        constraint = protocol_state.founder_state == founder_state.key(),
        constraint = protocol_state.company_state == company_state.key()
    )]
    pub protocol_state: Box<Account<'info, ProtocolState>>,

    #[account(
        seeds = [
            PROTOCOL_CONFIG_SEED,
            protocol_state.key().as_ref()
        ],
        bump = protocol_config.bump,
        constraint = protocol_config.protocol == protocol_state.key()
    )]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,

    #[account(
        mut,
        seeds = [
            TREASURY_SEED,
            protocol_state.key().as_ref()
        ],
        bump = treasury.bump,
        constraint = treasury.protocol == protocol_state.key()
    )]
    pub treasury: Box<Account<'info, TreasuryState>>,

    #[account(
        mut,
        seeds = [
            FOUNDER_STATE_SEED,
            protocol_state.key().as_ref()
        ],
        bump = founder_state.bump,
        constraint = founder_state.protocol == protocol_state.key()
    )]
    pub founder_state: Box<Account<'info, FounderState>>,

    #[account(
        mut,
        seeds = [
            COMPANY_STATE_SEED,
            protocol_state.key().as_ref()
        ],
        bump = company_state.bump,
        constraint = company_state.protocol == protocol_state.key()
    )]
    pub company_state: Box<Account<'info, CompanyState>>,

    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<ProcessFees>, amount: u64) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    let config = &ctx.accounts.protocol_config;

    let total_bps = u32::from(config.reserve_bps)
        .checked_add(u32::from(config.buyback_burn_bps))
        .and_then(|value| value.checked_add(u32::from(config.liquidity_bps)))
        .and_then(|value| value.checked_add(u32::from(config.company_bps)))
        .and_then(|value| value.checked_add(u32::from(config.founder_bps)))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        total_bps == u32::from(BPS_DENOMINATOR),
        TreasuryRouterError::InvalidAllocationConfiguration
    );

    let clock = Clock::get()?;

    let base_reserve_amount = calculate_share(amount, config.reserve_bps)?;

    let buyback_amount = calculate_share(amount, config.buyback_burn_bps)?;

    let liquidity_amount = calculate_share(amount, config.liquidity_bps)?;

    let requested_company_amount = calculate_share(amount, config.company_bps)?;

    let requested_founder_amount = calculate_share(amount, config.founder_bps)?;

    let allocated_before_remainder = base_reserve_amount
        .checked_add(buyback_amount)
        .and_then(|value| value.checked_add(liquidity_amount))
        .and_then(|value| value.checked_add(requested_company_amount))
        .and_then(|value| value.checked_add(requested_founder_amount))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let rounding_remainder = amount
        .checked_sub(allocated_before_remainder)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let normal_reserve_amount = base_reserve_amount
        .checked_add(rounding_remainder)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let founder_allocation = founder::allocate(
        &mut ctx.accounts.founder_state,
        requested_founder_amount,
        clock.unix_timestamp,
    )?;

    let company_allocation = company::allocate(
        &mut ctx.accounts.company_state,
        requested_company_amount,
        clock.unix_timestamp,
    )?;

    let founder_amount = founder_allocation.founder_amount;
    let company_amount = company_allocation.company_amount;

    let expected_reserve_amount = normal_reserve_amount;

    let final_allocated_amount = expected_reserve_amount
        .checked_add(buyback_amount)
        .and_then(|value| value.checked_add(liquidity_amount))
        .and_then(|value| value.checked_add(company_amount))
        .and_then(|value| value.checked_add(founder_amount))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        final_allocated_amount == amount,
        TreasuryRouterError::InvalidAllocationConfiguration
    );

    let treasury = &mut ctx.accounts.treasury;

    treasury.total_fees_received = treasury
        .total_fees_received
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.total_fees_allocated = treasury
        .total_fees_allocated
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let reserve_deposit = reserve::deposit_fee_allocation(treasury, normal_reserve_amount)?;

    let liquidity_deposit = liquidity::deposit(
        treasury,
        liquidity_amount
            .checked_add(founder_allocation.liquidity_overflow_amount)
            .and_then(|v| v.checked_add(company_allocation.liquidity_overflow_amount))
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?,
    )?;

    let buyback_deposit = buyback::deposit(treasury, buyback_amount)?;

    treasury.pending_company = treasury
        .pending_company
        .checked_add(company_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.pending_founder = treasury
        .pending_founder
        .checked_add(founder_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.lifetime_company = treasury
        .lifetime_company
        .checked_add(company_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.lifetime_founder = treasury
        .lifetime_founder
        .checked_add(founder_amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.processing_epoch = treasury
        .processing_epoch
        .checked_add(1)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.last_processed_at = clock.unix_timestamp;

    let previous_waterfall_stage = treasury.waterfall_stage;
    let waterfall_evaluation = waterfall::evaluate(treasury)?;

    treasury.waterfall_stage = waterfall_evaluation.stage.as_u8();

    let previous_dam_level = ctx.accounts.protocol_state.dam_level;

    let dam_evaluation = dam::evaluate(waterfall_evaluation.stage);

    ctx.accounts.protocol_state.dam_level = dam_evaluation.level.as_u8();

    let previous_beaver_score = ctx.accounts.protocol_state.beaver_score;

    let beaver_score_evaluation = beaver_score::evaluate(
        treasury,
        waterfall_evaluation.reserve_ratio_bps,
        waterfall_evaluation.stage,
        dam_evaluation.level,
    )?;

    ctx.accounts.protocol_state.beaver_score = beaver_score_evaluation.total_score;

    msg!("Beavernomics fee accounting completed");
    msg!("Gross fee amount: {}", amount);

    msg!(
        "Normal reserve allocation: {}",
        reserve_deposit.normal_amount
    );

    msg!("Total reserve allocation: {}", reserve_deposit.total_amount);

    msg!(
        "Liquidity allocation deposited: {}",
        liquidity_deposit.amount
    );

    msg!(
        "Pending liquidity balance: {}",
        liquidity_deposit.pending_balance
    );

    msg!(
        "Lifetime liquidity allocation: {}",
        liquidity_deposit.lifetime_total
    );

    msg!(
        "Buyback-and-burn allocation deposited: {}",
        buyback_deposit.amount
    );

    msg!(
        "Pending buyback-and-burn balance: {}",
        buyback_deposit.pending_balance
    );

    msg!(
        "Lifetime buyback-and-burn allocation: {}",
        buyback_deposit.lifetime_total
    );

    msg!("Company requested allocation: {}", requested_company_amount);

    msg!("Company actual allocation: {}", company_amount);

    msg!(
        "Company spent during current period: {}",
        ctx.accounts.company_state.spent_current_period
    );

    msg!(
        "Company period cap: {}",
        ctx.accounts.company_state.period_cap
    );

    msg!("Founder requested allocation: {}", requested_founder_amount);

    msg!("Founder actual allocation: {}", founder_amount);

    msg!(
        "Founder earned during current period: {}",
        ctx.accounts.founder_state.earned_current_period
    );

    msg!(
        "Founder period cap: {}",
        ctx.accounts.founder_state.period_cap
    );

    msg!("Waterfall previous stage: {}", previous_waterfall_stage);

    msg!(
        "Waterfall current stage: {} ({})",
        waterfall_evaluation.stage.as_u8(),
        waterfall_evaluation.stage.label()
    );

    msg!(
        "Waterfall reserve ratio: {} basis points",
        waterfall_evaluation.reserve_ratio_bps
    );

    msg!(
        "Waterfall pending reserve: {}",
        waterfall_evaluation.pending_reserve
    );

    msg!(
        "Waterfall total pending allocations: {}",
        waterfall_evaluation.total_pending
    );

    msg!("Dam previous level: {}", previous_dam_level);

    msg!(
        "Dam current level: {} ({})",
        dam_evaluation.level.as_u8(),
        dam_evaluation.level.label()
    );

    msg!(
        "Dam maximum release rate: {} basis points",
        dam_evaluation.release_bps
    );

    msg!("Beaver Score previous value: {}", previous_beaver_score);

    msg!(
        "Beaver Score current value: {} / {}",
        beaver_score_evaluation.total_score,
        beaver_score::MAX_BEAVER_SCORE
    );

    msg!(
        "Beaver Score Waterfall component: {} / {}",
        beaver_score_evaluation.waterfall_points,
        beaver_score::WATERFALL_MAX_POINTS
    );

    msg!(
        "Beaver Score Dam component: {} / {}",
        beaver_score_evaluation.dam_points,
        beaver_score::DAM_MAX_POINTS
    );

    msg!(
        "Beaver Score reserve component: {} / {}",
        beaver_score_evaluation.reserve_points,
        beaver_score::RESERVE_MAX_POINTS
    );

    msg!(
        "Beaver Score accounting component: {} / {}",
        beaver_score_evaluation.accounting_points,
        beaver_score::ACCOUNTING_MAX_POINTS
    );

    msg!("Processing epoch: {}", treasury.processing_epoch);

    Ok(())
}

fn calculate_share(amount: u64, basis_points: u16) -> Result<u64> {
    let numerator = u128::from(amount)
        .checked_mul(u128::from(basis_points))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let share = numerator
        .checked_div(u128::from(BPS_DENOMINATOR))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    u64::try_from(share).map_err(|_| TreasuryRouterError::ArithmeticOverflow.into())
}
