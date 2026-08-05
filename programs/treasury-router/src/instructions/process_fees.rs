use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::{
    constants::{
        BPS_DENOMINATOR, COMPANY_STATE_SEED, FOUNDER_STATE_SEED, PROTOCOL_CONFIG_SEED,
        PROTOCOL_SEED, TREASURY_SEED,
    },
    engines::{beaver_score, buyback, company, dam, liquidity, reserve, sentinel, waterfall},
    errors::TreasuryRouterError,
    state::{
        CompanyState, FounderPriceState, FounderState, FounderUsdCapState, ProtocolConfig,
        ProtocolState, TreasuryState, FOUNDER_PRICE_SEED, FOUNDER_USD_CAP_SEED,
    },
};

#[derive(Accounts)]
pub struct ProcessFees<'info> {
    #[account(
        constraint = settlement_mint.key()
            == treasury.settlement_mint
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub settlement_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
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

    /// Migration-compatible Founder recipient and linkage account.
    ///
    /// The token-denominated period cap in this account is no longer the
    /// authoritative production cap. FounderUsdCapState is authoritative.
    #[account(
        mut,
        seeds = [
            FOUNDER_USD_CAP_SEED,
            protocol_state.key().as_ref(),
        ],
        bump = founder_usd_cap.bump,
        constraint = founder_usd_cap.protocol
            == protocol_state.key()
            @ TreasuryRouterError::InvalidFounderUsdCapProtocol,
        constraint = founder_usd_cap.settlement_mint
            == treasury.settlement_mint
            @ TreasuryRouterError::InvalidFounderUsdCapSettlementMint
    )]
    pub founder_usd_cap: Box<Account<'info, FounderUsdCapState>>,

    #[account(
        seeds = [
            FOUNDER_PRICE_SEED,
            protocol_state.key().as_ref(),
        ],
        bump = founder_price.bump,
        constraint = founder_price.protocol
            == protocol_state.key()
            @ TreasuryRouterError::InvalidFounderPriceProtocol,
        constraint = founder_price.settlement_mint
            == treasury.settlement_mint
            @ TreasuryRouterError::InvalidFounderPriceSettlementMint,
        constraint = founder_price.price_feed_id
            == founder_usd_cap.price_feed_id
            @ TreasuryRouterError::InvalidFounderUsdPriceFeed
    )]
    pub founder_price: Box<Account<'info, FounderPriceState>>,

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

    #[account(
        constraint = settlement_vault.key() == treasury.settlement_vault
            @ TreasuryRouterError::InvalidSettlementVault,
        constraint = settlement_vault.mint == treasury.settlement_mint
            @ TreasuryRouterError::InvalidSettlementMint,
        constraint = settlement_vault.owner == treasury.key()
            @ TreasuryRouterError::InvalidSettlementVault
    )]
    pub settlement_vault: Box<Account<'info, TokenAccount>>,
}

pub fn handler(ctx: Context<ProcessFees>) -> Result<()> {
    require!(
        !ctx.accounts.protocol_state.paused,
        TreasuryRouterError::ProtocolPaused
    );

    // Fail closed before any accounting mutation. Sentinel V2 verifies the
    // exact locked 30/20/20/20/10 configuration, immutable configuration flag,
    // canonical protocol linkage, and lifetime-allocation conservation.
    let pre_linkage_report = sentinel::evaluate_linkage(
        ctx.accounts.protocol_state.key(),
        &ctx.accounts.protocol_config,
        &ctx.accounts.treasury,
    );

    require!(
        pre_linkage_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    let pre_sentinel_report =
        sentinel::evaluate(&ctx.accounts.protocol_config, &ctx.accounts.treasury)?;

    require!(
        pre_sentinel_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    let total_released = ctx
        .accounts
        .treasury
        .released_reserve
        .checked_add(ctx.accounts.treasury.released_buyback_burn)
        .and_then(|value| value.checked_add(ctx.accounts.treasury.released_liquidity))
        .and_then(|value| value.checked_add(ctx.accounts.treasury.released_company))
        .and_then(|value| value.checked_add(ctx.accounts.treasury.released_founder))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let total_received = ctx
        .accounts
        .settlement_vault
        .amount
        .checked_add(total_released)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let amount = total_received
        .checked_sub(ctx.accounts.treasury.total_fees_allocated)
        .ok_or(TreasuryRouterError::AccountingInvariantViolation)?;

    require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);

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

    let FeeCycleOutcome {
        pre_allocation_waterfall,
        adaptive_allocation,
        requested_company_amount,
        requested_founder_amount,
        company_amount,
        founder_amount,
        reserve_deposit,
        liquidity_deposit,
        buyback_deposit,
        previous_waterfall_stage,
        waterfall_evaluation,
        previous_dam_level,
        pre_dam_health_score,
        dam_evaluation,
        previous_beaver_score,
        beaver_score_evaluation,
    } = process_fee_cycle_usd_cap(
        &mut ctx.accounts.protocol_state,
        &mut ctx.accounts.treasury,
        &mut ctx.accounts.founder_state,
        &mut ctx.accounts.founder_usd_cap,
        &ctx.accounts.founder_price,
        ctx.accounts.settlement_mint.decimals,
        &mut ctx.accounts.company_state,
        amount,
        clock.unix_timestamp,
    )?;

    // Re-run both Sentinel layers after every state mutation. A failed
    // invariant aborts the transaction atomically and rolls all changes back.
    let post_linkage_report = sentinel::evaluate_linkage(
        ctx.accounts.protocol_state.key(),
        &ctx.accounts.protocol_config,
        &ctx.accounts.treasury,
    );

    require!(
        post_linkage_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    let post_sentinel_report =
        sentinel::evaluate(&ctx.accounts.protocol_config, &ctx.accounts.treasury)?;

    require!(
        post_sentinel_report.healthy,
        TreasuryRouterError::IntegrityFirewallViolation
    );

    msg!("Beavernomics fee accounting completed");
    msg!("Gross fee amount: {}", amount);

    msg!(
        "Pre-allocation Waterfall stage: {} ({})",
        pre_allocation_waterfall.stage.as_u8(),
        pre_allocation_waterfall.stage.label()
    );

    msg!(
        "Adaptive allocation BPS R/B/L/C/F: {}/{}/{}/{}/{}",
        adaptive_allocation.reserve_bps,
        adaptive_allocation.buyback_burn_bps,
        adaptive_allocation.liquidity_bps,
        adaptive_allocation.company_bps,
        adaptive_allocation.founder_bps
    );

    msg!(
        "Reserve allocation including rounding: {}",
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
        "Founder USD earned during current annual period: {}",
        ctx.accounts.founder_usd_cap.earned_current_period_usd_e6
    );

    msg!(
        "Founder annual USD cap: {}",
        ctx.accounts.founder_usd_cap.annual_cap_usd_e6
    );

    msg!(
        "Founder annual period duration: {} seconds",
        ctx.accounts.founder_usd_cap.period_duration
    );

    msg!(
        "Founder oracle price/exponent: {}/{}",
        ctx.accounts.founder_price.price,
        ctx.accounts.founder_price.exponent
    );

    msg!(
        "Founder oracle publication time: {}",
        ctx.accounts.founder_price.publish_time
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
        "Pre-Dam Health Score: {} / {}",
        pre_dam_health_score,
        beaver_score::PRE_DAM_MAX_POINTS
    );

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

    msg!(
        "Processing epoch: {}",
        ctx.accounts.treasury.processing_epoch
    );

    Ok(())
}

/// Deterministic result of one successful Beavernomics fee-processing cycle.
///
/// Account validation, vault-balance discovery, protocol pause enforcement,
/// and Sentinel linkage checks remain in the Anchor instruction handler.
/// This function contains the production economic state transition and is
/// shared directly with the fuzz harness.
pub struct FeeCycleOutcome {
    pub pre_allocation_waterfall: waterfall::WaterfallEvaluation,
    pub adaptive_allocation: waterfall::AdaptiveAllocation,
    pub requested_company_amount: u64,
    pub requested_founder_amount: u64,
    pub company_amount: u64,
    pub founder_amount: u64,
    pub reserve_deposit: reserve::ReserveDeposit,
    pub liquidity_deposit: liquidity::LiquidityDeposit,
    pub buyback_deposit: buyback::BuybackDeposit,
    pub previous_waterfall_stage: u8,
    pub waterfall_evaluation: waterfall::WaterfallEvaluation,
    pub previous_dam_level: u8,
    pub pre_dam_health_score: u16,
    pub dam_evaluation: dam::DamEvaluation,
    pub previous_beaver_score: u16,
    pub beaver_score_evaluation: beaver_score::BeaverScoreEvaluation,
}

/// Applies one complete production fee-allocation state transition.
///
/// All arithmetic is checked. Company and Founder overflow is redirected to
/// Liquidity Growth. Rounding remainder is assigned to Reserve. Any error
/// aborts the surrounding Solana transaction atomically when called through
/// the instruction handler.
pub fn process_fee_cycle_usd_cap(
    protocol_state: &mut ProtocolState,
    treasury: &mut TreasuryState,
    founder_state: &mut FounderState,
    founder_usd_cap: &mut FounderUsdCapState,
    founder_price: &FounderPriceState,
    settlement_token_decimals: u8,
    company_state: &mut CompanyState,
    amount: u64,
    now: i64,
) -> Result<FeeCycleOutcome> {
    require!(amount > 0, TreasuryRouterError::NoUnprocessedFees);

    // Evaluate health before applying this cycle so the transaction cannot
    // improve the stage used to allocate itself.
    let pre_allocation_waterfall = waterfall::evaluate(treasury)?;

    let adaptive_allocation = waterfall::allocation_for_stage(pre_allocation_waterfall.stage);

    require!(
        adaptive_allocation.total_bps() == u32::from(BPS_DENOMINATOR),
        TreasuryRouterError::InvalidAllocationConfiguration
    );

    let base_reserve_amount = calculate_share(amount, adaptive_allocation.reserve_bps)?;

    let buyback_amount = calculate_share(amount, adaptive_allocation.buyback_burn_bps)?;

    let liquidity_amount = calculate_share(amount, adaptive_allocation.liquidity_bps)?;

    let requested_company_amount = calculate_share(amount, adaptive_allocation.company_bps)?;

    let requested_founder_amount =
        crate::engines::founder::calculate_progressive_request(
            founder_state,
            amount,
            adaptive_allocation.founder_bps,
        )?;

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

    founder_price.validate_price(now)?;
    founder_usd_cap.validate_configuration()?;

    let founder_allocation = crate::engines::founder_usd_cap::apply_founder_usd_cap(
        founder_usd_cap,
        requested_founder_amount,
        settlement_token_decimals,
        founder_price.price,
        founder_price.exponent,
        now,
    )?;

    let company_allocation = company::allocate(company_state, requested_company_amount, now)?;

    let founder_amount = founder_allocation.founder_token_amount;
    let company_amount = company_allocation.company_amount;

    // Locked Bevernomics rule: all Company and Founder cap overflow is
    // redirected to Liquidity Growth.
    let final_liquidity_amount = liquidity_amount
        .checked_add(founder_allocation.liquidity_overflow_token_amount)
        .and_then(|value| value.checked_add(company_allocation.liquidity_overflow_amount))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let final_allocated_amount = normal_reserve_amount
        .checked_add(buyback_amount)
        .and_then(|value| value.checked_add(final_liquidity_amount))
        .and_then(|value| value.checked_add(company_amount))
        .and_then(|value| value.checked_add(founder_amount))
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    require!(
        final_allocated_amount == amount,
        TreasuryRouterError::InvalidAllocationConfiguration
    );

    treasury.total_fees_received = treasury
        .total_fees_received
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.total_fees_allocated = treasury
        .total_fees_allocated
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    let reserve_deposit = reserve::deposit_fee_allocation(treasury, normal_reserve_amount)?;

    let liquidity_deposit = liquidity::deposit(treasury, final_liquidity_amount)?;

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

    treasury.last_processed_at = now;

    let previous_waterfall_stage = treasury.waterfall_stage;
    let waterfall_evaluation = waterfall::evaluate(treasury)?;

    treasury.waterfall_stage = waterfall_evaluation.stage.as_u8();

    let previous_dam_level = protocol_state.dam_level;

    let pre_dam_health_score = beaver_score::pre_dam_health_score(
        treasury,
        waterfall_evaluation.reserve_ratio_bps,
        waterfall_evaluation.stage,
    )?;

    let dam_evaluation = dam::evaluate_adaptive(waterfall_evaluation.stage, pre_dam_health_score);

    protocol_state.dam_level = dam_evaluation.level.as_u8();

    let previous_beaver_score = protocol_state.beaver_score;

    let beaver_score_evaluation = beaver_score::evaluate(
        treasury,
        waterfall_evaluation.reserve_ratio_bps,
        waterfall_evaluation.stage,
        dam_evaluation.level,
    )?;

    protocol_state.beaver_score = beaver_score_evaluation.total_score;

    Ok(FeeCycleOutcome {
        pre_allocation_waterfall,
        adaptive_allocation,
        requested_company_amount,
        requested_founder_amount,
        company_amount,
        founder_amount,
        reserve_deposit,
        liquidity_deposit,
        buyback_deposit,
        previous_waterfall_stage,
        waterfall_evaluation,
        previous_dam_level,
        pre_dam_health_score,
        dam_evaluation,
        previous_beaver_score,
        beaver_score_evaluation,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn calculate_total(
        reserve: u64,
        buyback: u64,
        liquidity: u64,
        company: u64,
        founder: u64,
    ) -> Result<u64> {
        reserve
            .checked_add(buyback)
            .and_then(|value| value.checked_add(liquidity))
            .and_then(|value| value.checked_add(company))
            .and_then(|value| value.checked_add(founder))
            .ok_or(TreasuryRouterError::ArithmeticOverflow.into())
    }

    #[test]
    fn calculate_share_returns_exact_whole_number_share() {
        let result = calculate_share(1_000_000, 3_000).unwrap();

        assert_eq!(result, 300_000);
    }

    #[test]
    fn calculate_share_rounds_down() {
        let result = calculate_share(101, 2_000).unwrap();

        assert_eq!(result, 20);
    }

    #[test]
    fn calculate_share_returns_zero_for_zero_amount() {
        let result = calculate_share(0, 3_000).unwrap();

        assert_eq!(result, 0);
    }

    #[test]
    fn calculate_share_returns_zero_for_zero_basis_points() {
        let result = calculate_share(1_000_000, 0).unwrap();

        assert_eq!(result, 0);
    }

    #[test]
    fn calculate_share_returns_full_amount_for_full_basis_points() {
        let result = calculate_share(1_000_000, BPS_DENOMINATOR).unwrap();

        assert_eq!(result, 1_000_000);
    }

    #[test]
    fn calculate_share_handles_u64_max_without_multiplication_overflow() {
        let result = calculate_share(u64::MAX, BPS_DENOMINATOR).unwrap();

        assert_eq!(result, u64::MAX);
    }

    #[test]
    fn locked_normal_allocation_is_conserved() {
        let amount = 1_000_000;

        let reserve = calculate_share(amount, 3_000).unwrap();
        let buyback = calculate_share(amount, 2_000).unwrap();
        let liquidity = calculate_share(amount, 2_000).unwrap();
        let company = calculate_share(amount, 2_000).unwrap();
        let founder = calculate_share(amount, 1_000).unwrap();

        let total = calculate_total(reserve, buyback, liquidity, company, founder).unwrap();

        assert_eq!(reserve, 300_000);
        assert_eq!(buyback, 200_000);
        assert_eq!(liquidity, 200_000);
        assert_eq!(company, 200_000);
        assert_eq!(founder, 100_000);
        assert_eq!(total, amount);
    }

    #[test]
    fn rounding_remainder_is_assigned_to_reserve() {
        let amount = 101;

        let base_reserve = calculate_share(amount, 3_000).unwrap();
        let buyback = calculate_share(amount, 2_000).unwrap();
        let liquidity = calculate_share(amount, 2_000).unwrap();
        let company = calculate_share(amount, 2_000).unwrap();
        let founder = calculate_share(amount, 1_000).unwrap();

        let allocated_before_remainder =
            calculate_total(base_reserve, buyback, liquidity, company, founder).unwrap();

        let rounding_remainder = amount.checked_sub(allocated_before_remainder).unwrap();

        let final_reserve = base_reserve.checked_add(rounding_remainder).unwrap();

        let total = calculate_total(final_reserve, buyback, liquidity, company, founder).unwrap();

        assert_eq!(base_reserve, 30);
        assert_eq!(rounding_remainder, 1);
        assert_eq!(final_reserve, 31);
        assert_eq!(total, amount);
    }

    #[test]
    fn company_cap_overflow_redirected_to_liquidity_is_conserved() {
        let normal_reserve: u64 = 300;
        let buyback: u64 = 200;
        let base_liquidity: u64 = 200;
        let requested_company: u64 = 200;
        let founder = 100;

        let actual_company: u64 = 50;
        let company_overflow = requested_company - actual_company;

        let final_liquidity = base_liquidity.checked_add(company_overflow).unwrap();

        let total = calculate_total(
            normal_reserve,
            buyback,
            final_liquidity,
            actual_company,
            founder,
        )
        .unwrap();

        assert_eq!(company_overflow, 150);
        assert_eq!(final_liquidity, 350);
        assert_eq!(total, 1_000);
    }

    #[test]
    fn founder_cap_overflow_redirected_to_liquidity_is_conserved() {
        let normal_reserve: u64 = 300;
        let buyback: u64 = 200;
        let base_liquidity: u64 = 200;
        let company = 200;
        let requested_founder: u64 = 100;

        let actual_founder: u64 = 25;
        let founder_overflow = requested_founder - actual_founder;

        let final_liquidity = base_liquidity.checked_add(founder_overflow).unwrap();

        let total = calculate_total(
            normal_reserve,
            buyback,
            final_liquidity,
            company,
            actual_founder,
        )
        .unwrap();

        assert_eq!(founder_overflow, 75);
        assert_eq!(final_liquidity, 275);
        assert_eq!(total, 1_000);
    }

    #[test]
    fn simultaneous_company_and_founder_overflow_is_conserved() {
        let normal_reserve: u64 = 300;
        let buyback: u64 = 200;
        let base_liquidity: u64 = 200;

        let requested_company: u64 = 200;
        let actual_company: u64 = 50;
        let company_overflow = requested_company - actual_company;

        let requested_founder: u64 = 100;
        let actual_founder: u64 = 25;
        let founder_overflow = requested_founder - actual_founder;

        let final_liquidity = base_liquidity
            .checked_add(company_overflow)
            .and_then(|value| value.checked_add(founder_overflow))
            .unwrap();

        let total = calculate_total(
            normal_reserve,
            buyback,
            final_liquidity,
            actual_company,
            actual_founder,
        )
        .unwrap();

        assert_eq!(company_overflow, 150);
        assert_eq!(founder_overflow, 75);
        assert_eq!(final_liquidity, 425);
        assert_eq!(total, 1_000);
    }

    #[test]
    fn fully_capped_company_and_founder_allocations_go_to_liquidity() {
        let normal_reserve: u64 = 300;
        let buyback: u64 = 200;
        let base_liquidity: u64 = 200;

        let actual_company: u64 = 0;
        let company_overflow: u64 = 200;

        let actual_founder: u64 = 0;
        let founder_overflow: u64 = 100;

        let final_liquidity = base_liquidity
            .checked_add(company_overflow)
            .and_then(|value| value.checked_add(founder_overflow))
            .unwrap();

        let total = calculate_total(
            normal_reserve,
            buyback,
            final_liquidity,
            actual_company,
            actual_founder,
        )
        .unwrap();

        assert_eq!(final_liquidity, 500);
        assert_eq!(total, 1_000);
    }

    #[test]
    fn allocation_total_fails_on_overflow() {
        let result = calculate_total(u64::MAX, 1, 0, 0, 0);

        assert!(result.is_err());
    }
}
