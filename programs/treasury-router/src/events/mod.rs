use anchor_lang::prelude::*;

/// Emitted when the protocol authorizes reserve funds for execution.
///
/// This event records treasury authorization only. It does not claim that
/// settlement tokens have already been transferred.
#[event]
pub struct ReserveExecutionAuthorized {
    pub protocol: Pubkey,
    pub treasury: Pubkey,
    pub authorized_amount: u64,
    pub previous_pending_balance: u64,
    pub remaining_pending_balance: u64,
    pub maximum_release: u64,
    pub waterfall_stage: u8,
    pub dam_level: u8,
    pub release_bps: u16,
    pub reserve_ratio_bps: u16,
    pub beaver_score: u16,
    pub processing_epoch: u64,
    pub authorized_at: i64,
}

/// Emitted when the protocol authorizes buyback funds for execution.
///
/// This event records treasury authorization only. It does not claim that a
/// swap or RBVR burn has already occurred.
#[event]
pub struct BuybackExecutionAuthorized {
    pub protocol: Pubkey,
    pub treasury: Pubkey,
    pub authorized_amount: u64,
    pub previous_pending_balance: u64,
    pub remaining_pending_balance: u64,
    pub maximum_release: u64,
    pub waterfall_stage: u8,
    pub dam_level: u8,
    pub release_bps: u16,
    pub reserve_ratio_bps: u16,
    pub beaver_score: u16,
    pub processing_epoch: u64,
    pub authorized_at: i64,
}

/// Emitted after liquidity funds have been authorized and transferred.
///
/// The settlement-token transfer and accounting update occur atomically.
/// This event does not claim that an LP position has been created.
#[event]
pub struct LiquidityExecutionAuthorized {
    pub protocol: Pubkey,
    pub treasury: Pubkey,
    pub authorized_amount: u64,
    pub previous_pending_balance: u64,
    pub remaining_pending_balance: u64,
    pub maximum_release: u64,
    pub waterfall_stage: u8,
    pub dam_level: u8,
    pub release_bps: u16,
    pub reserve_ratio_bps: u16,
    pub beaver_score: u16,
    pub processing_epoch: u64,
    pub authorized_at: i64,
}

/// Emitted after previously allocated founder funds are transferred.
///
/// Founder caps are enforced during fee processing. The settlement-token
/// transfer and treasury execution-accounting update occur atomically.
#[event]
pub struct FounderExecutionAuthorized {
    pub protocol: Pubkey,
    pub treasury: Pubkey,
    pub founder_state: Pubkey,
    pub recipient: Pubkey,
    pub authorized_amount: u64,
    pub previous_pending_balance: u64,
    pub remaining_pending_balance: u64,
    pub maximum_release: u64,
    pub waterfall_stage: u8,
    pub dam_level: u8,
    pub release_bps: u16,
    pub reserve_ratio_bps: u16,
    pub beaver_score: u16,
    pub processing_epoch: u64,
    pub founder_period_cap: u64,
    pub founder_earned_current_period: u64,
    pub founder_lifetime_earned: u64,
    pub authorized_at: i64,
}

/// Emitted after previously allocated company funds are transferred.
///
/// Company caps are enforced during fee processing. The settlement-token
/// transfer and treasury execution-accounting update occur atomically.
#[event]
pub struct CompanyExecutionAuthorized {
    pub protocol: Pubkey,
    pub treasury: Pubkey,
    pub company_state: Pubkey,
    pub recipient: Pubkey,
    pub authorized_amount: u64,
    pub previous_pending_balance: u64,
    pub remaining_pending_balance: u64,
    pub maximum_release: u64,
    pub waterfall_stage: u8,
    pub dam_level: u8,
    pub release_bps: u16,
    pub reserve_ratio_bps: u16,
    pub beaver_score: u16,
    pub processing_epoch: u64,
    pub company_period_cap: u64,
    pub company_spent_current_period: u64,
    pub company_lifetime_spent: u64,
    pub authorized_at: i64,
}

/// Emitted after a successful permissionless Reserve Spillway release.
#[event]
pub struct SpillwayReleaseExecuted {
    pub protocol: Pubkey,
    pub treasury: Pubkey,
    pub reserve_policy: Pubkey,
    pub reserve_vault: Pubkey,
    pub destination: Pubkey,

    pub reserve_balance_before: u64,
    pub protected_floor: u64,
    pub gross_surplus: u64,
    pub deployed_amount: u64,
    pub reserve_balance_after: u64,

    pub liquidity_reference: u64,
    pub deployment_bps: u16,

    pub lifetime_deployed: u64,
    pub executed_at: i64,
}
