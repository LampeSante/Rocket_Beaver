use anchor_lang::prelude::*;

use crate::{
    constants::PROTOCOL_SEED,
    errors::TreasuryRouterError,
    state::{
        FounderPriceState, FounderUsdCapState, ProtocolState, FOUNDER_PRICE_SEED,
        FOUNDER_USD_CAP_SEED,
    },
};

/// Temporary feed ID used when the Founder USD oracle accounts were first
/// initialized on Devnet.
///
/// The migration instruction becomes permanently unusable after both accounts
/// are changed away from this value.
pub const FOUNDER_PLACEHOLDER_FEED_ID: [u8; 32] = [7u8; 32];

#[derive(Accounts)]
pub struct MigrateFounderFeedId<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        has_one = authority @ TreasuryRouterError::Unauthorized
    )]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(
        mut,
        seeds = [
            FOUNDER_USD_CAP_SEED,
            protocol_state.key().as_ref()
        ],
        bump = founder_usd_cap.bump,
        constraint =
            founder_usd_cap.protocol == protocol_state.key()
            @ TreasuryRouterError::InvalidFounderUsdCapProtocol
    )]
    pub founder_usd_cap: Account<'info, FounderUsdCapState>,

    #[account(
        mut,
        seeds = [
            FOUNDER_PRICE_SEED,
            protocol_state.key().as_ref()
        ],
        bump = founder_price.bump,
        constraint =
            founder_price.protocol == protocol_state.key()
            @ TreasuryRouterError::InvalidFounderPriceProtocol,
        constraint =
            founder_price.settlement_mint
                == founder_usd_cap.settlement_mint
            @ TreasuryRouterError::InvalidFounderPriceSettlementMint
    )]
    pub founder_price: Account<'info, FounderPriceState>,

    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<MigrateFounderFeedId>, new_feed_id: [u8; 32]) -> Result<()> {
    let protocol_state = &ctx.accounts.protocol_state;
    let founder_usd_cap = &mut ctx.accounts.founder_usd_cap;
    let founder_price = &mut ctx.accounts.founder_price;

    require!(!protocol_state.paused, TreasuryRouterError::ProtocolPaused);

    require!(
        new_feed_id != [0u8; 32],
        TreasuryRouterError::InvalidFounderUsdPriceFeed
    );

    require!(
        new_feed_id != FOUNDER_PLACEHOLDER_FEED_ID,
        TreasuryRouterError::InvalidFounderFeedMigrationTarget
    );

    // These checks make the migration self-locking. Once either stored feed
    // changes, this instruction can never succeed again.
    require!(
        founder_usd_cap.price_feed_id == FOUNDER_PLACEHOLDER_FEED_ID,
        TreasuryRouterError::FounderFeedMigrationUnavailable
    );

    require!(
        founder_price.price_feed_id == FOUNDER_PLACEHOLDER_FEED_ID,
        TreasuryRouterError::FounderFeedMigrationUnavailable
    );

    // The accounts must still be entirely unused.
    require!(
        founder_price.sequence == 0,
        TreasuryRouterError::FounderFeedMigrationStateUsed
    );

    require!(
        founder_price.price == 0,
        TreasuryRouterError::FounderFeedMigrationStateUsed
    );

    require!(
        founder_price.confidence == 0,
        TreasuryRouterError::FounderFeedMigrationStateUsed
    );

    require!(
        founder_price.publish_time == 0,
        TreasuryRouterError::FounderFeedMigrationStateUsed
    );

    require!(
        founder_price.received_at == 0,
        TreasuryRouterError::FounderFeedMigrationStateUsed
    );

    require!(
        founder_usd_cap.earned_current_period_usd_e6 == 0,
        TreasuryRouterError::FounderFeedMigrationStateUsed
    );

    require!(
        founder_usd_cap.lifetime_earned_usd_e6 == 0,
        TreasuryRouterError::FounderFeedMigrationStateUsed
    );

    let previous_feed_id = founder_price.price_feed_id;

    founder_usd_cap.price_feed_id = new_feed_id;
    founder_price.price_feed_id = new_feed_id;

    emit!(FounderFeedIdMigrated {
        protocol: protocol_state.key(),
        previous_feed_id,
        new_feed_id,
        authority: ctx.accounts.authority.key(),
    });

    msg!("Founder oracle feed ID migrated.");
    msg!("Founder USD cap account: {}", founder_usd_cap.key());
    msg!("Founder price account: {}", founder_price.key());

    Ok(())
}

#[event]
pub struct FounderFeedIdMigrated {
    pub protocol: Pubkey,
    pub previous_feed_id: [u8; 32],
    pub new_feed_id: [u8; 32],
    pub authority: Pubkey,
}
