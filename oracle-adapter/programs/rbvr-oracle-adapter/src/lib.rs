use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use treasury_router::program::TreasuryRouter;

declare_id!("E5T6hAp6pTE9iZvwMMRaUD7kDC3UkJWiiqi24fJf3sqx");

/// Canonical signer used when submitting verified prices to Treasury Router.
///
/// PDA:
/// ["rbvr-oracle-authority"]
pub const ORACLE_AUTHORITY_SEED: &[u8] = b"rbvr-oracle-authority";

#[program]
pub mod rbvr_oracle_adapter {
    use super::*;

    /// Verifies a Pyth price and immediately submits it to the RBVR
    /// Treasury Router.
    ///
    /// The Treasury Router accepts the update only when
    ///  matches the authority permanently stored
    /// in FounderPriceState.
    pub fn verify_and_submit_founder_price(
        ctx: Context<VerifyAndSubmitFounderPrice>,
        expected_feed_id: [u8; 32],
        maximum_age_seconds: u64,
        maximum_confidence_bps: u16,
    ) -> Result<()> {
        require!(
            expected_feed_id != [0u8; 32],
            OracleAdapterError::InvalidFeedId
        );

        require!(
            maximum_age_seconds > 0,
            OracleAdapterError::InvalidMaximumAge
        );

        require!(
            maximum_confidence_bps > 0 && maximum_confidence_bps <= 10_000,
            OracleAdapterError::InvalidConfidenceLimit
        );

        let clock = Clock::get()?;

        // This validates:
        // - Pyth Receiver ownership through Account<PriceUpdateV2>;
        // - the expected feed identifier;
        // - the maximum accepted publication age;
        // - sufficient Pyth verification level.
        let price = ctx.accounts.price_update.get_price_no_older_than(
            &clock,
            maximum_age_seconds,
            &expected_feed_id,
        )?;

        require!(price.price > 0, OracleAdapterError::InvalidPrice);

        require!(
            price.publish_time > 0 && price.publish_time <= clock.unix_timestamp,
            OracleAdapterError::InvalidPublishTime
        );

        let absolute_price =
            u128::try_from(price.price).map_err(|_| OracleAdapterError::InvalidPrice)?;

        let confidence_bps = u128::from(price.conf)
            .checked_mul(10_000)
            .ok_or(OracleAdapterError::ArithmeticOverflow)?
            .checked_div(absolute_price)
            .ok_or(OracleAdapterError::ArithmeticOverflow)?;

        require!(
            confidence_bps <= u128::from(maximum_confidence_bps),
            OracleAdapterError::ConfidenceTooWide
        );

        let authority_bump = ctx.bumps.oracle_adapter_authority;

        let authority_seeds: &[&[u8]] = &[ORACLE_AUTHORITY_SEED, &[authority_bump]];

        let signer_seeds: &[&[&[u8]]] = &[authority_seeds];

        let cpi_accounts = treasury_router::cpi::accounts::SubmitFounderPrice {
            protocol_state: ctx.accounts.protocol_state.to_account_info(),

            founder_price: ctx.accounts.founder_price.to_account_info(),

            oracle_adapter_authority: ctx.accounts.oracle_adapter_authority.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.treasury_router_program.key(),
            cpi_accounts,
            signer_seeds,
        );

        treasury_router::cpi::submit_founder_price(
            cpi_context,
            expected_feed_id,
            price.price,
            price.exponent,
            price.conf,
            price.publish_time,
        )?;

        msg!("RBVR verified Founder price submitted.");
        msg!("Price: {}", price.price);
        msg!("Exponent: {}", price.exponent);
        msg!("Confidence: {}", price.conf);
        msg!("Publication time: {}", price.publish_time);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct VerifyAndSubmitFounderPrice<'info> {
    /// Pyth Receiver price-update account.
    ///
    /// Anchor and the Pyth SDK validate ownership and deserialize the
    /// verification data.
    pub price_update: Account<'info, PriceUpdateV2>,

    /// Canonical adapter CPI signer.
    ///
    /// CHECK: This account is a program-derived address and carries no data.
    #[account(
        seeds = [ORACLE_AUTHORITY_SEED],
        bump
    )]
    pub oracle_adapter_authority: UncheckedAccount<'info>,

    /// Treasury Router ProtocolState.
    ///
    /// CHECK: Treasury Router validates its PDA, bump, and state.
    pub protocol_state: UncheckedAccount<'info>,

    /// Treasury Router FounderPriceState.
    ///
    /// CHECK: Treasury Router validates its PDA, protocol relationship,
    /// configured adapter authority, feed ID, freshness, and confidence.
    #[account(mut)]
    pub founder_price: UncheckedAccount<'info>,

    /// Treasury Router executable program.
    pub treasury_router_program: Program<'info, TreasuryRouter>,
}

#[error_code]
pub enum OracleAdapterError {
    #[msg("The expected Pyth feed identifier is invalid.")]
    InvalidFeedId,

    #[msg("Maximum oracle age must be greater than zero.")]
    InvalidMaximumAge,

    #[msg("Oracle confidence limit is invalid.")]
    InvalidConfidenceLimit,

    #[msg("The verified oracle price is not positive.")]
    InvalidPrice,

    #[msg("The oracle publication time is invalid.")]
    InvalidPublishTime,

    #[msg("The oracle confidence interval is too wide.")]
    ConfidenceTooWide,

    #[msg("Arithmetic overflow while validating the oracle price.")]
    ArithmeticOverflow,
}
