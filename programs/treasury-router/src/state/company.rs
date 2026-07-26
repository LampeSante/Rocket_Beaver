use anchor_lang::prelude::*;

#[account]
pub struct CompanyState {
    /// State layout version for future migrations.
    pub version: u16,

    /// Root ProtocolState controlling this company account.
    pub protocol: Pubkey,

    /// Wallet authorized to receive company allocations.
    pub recipient: Pubkey,

    /// Maximum company allocation during the current accounting period.
    pub period_cap: u64,

    /// Company allocation credited during the current period.
    pub spent_current_period: u64,

    /// Lifetime company allocation credited by the protocol.
    pub lifetime_spent: u64,

    /// Unix timestamp marking the start of the current period.
    pub period_started_at: i64,

    /// Length of each accounting period in seconds.
    pub period_duration: i64,

    /// Allows company allocations to be disabled independently.
    pub enabled: bool,

    /// Canonical CompanyState PDA bump.
    pub bump: u8,

    /// Reserved account space for compatible future fields.
    pub reserved: [u8; 64],
}

impl CompanyState {
    pub const SPACE: usize = 8 +  // Anchor discriminator
        2 +                       // version
        32 +                      // protocol
        32 +                      // recipient
        (5 * 8) +                 // three u64 values and two i64 values
        1 +                       // enabled
        1 +                       // bump
        64; // reserved
}
