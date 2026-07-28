use anchor_lang::prelude::*;

#[account]
pub struct FounderState {
    /// State layout version for future migrations.
    pub version: u16,

    /// Root ProtocolState controlling this founder account.
    pub protocol: Pubkey,

    /// Wallet authorized to receive founder allocations.
    pub recipient: Pubkey,

    /// Maximum founder allocation during the current accounting period.
    pub period_cap: u64,

    /// Founder allocation credited during the current period.
    pub earned_current_period: u64,

    /// Lifetime founder allocation credited by the protocol.
    pub lifetime_earned: u64,

    /// Unix timestamp marking the start of the current period.
    pub period_started_at: i64,

    /// Length of each accounting period in seconds.
    pub period_duration: i64,

    /// Progressive compensation tier. Initially zero.
    pub current_tier: u8,

    /// Allows founder allocations to be disabled independently.
    pub enabled: bool,

    /// Canonical FounderState PDA bump.
    pub bump: u8,

    /// Reserved account space for compatible future fields.
    pub reserved: [u8; 64],
}

impl FounderState {
    pub const SPACE: usize = 8 +  // Anchor discriminator
        2 +                       // version
        32 +                      // protocol
        32 +                      // recipient
        (5 * 8) +                 // three u64 values and two i64 values
        1 +                       // current_tier
        1 +                       // enabled
        1 +                       // bump
        64; // reserved
}
