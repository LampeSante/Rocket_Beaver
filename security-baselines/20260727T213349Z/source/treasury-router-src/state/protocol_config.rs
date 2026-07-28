use anchor_lang::prelude::*;

#[account]
pub struct ProtocolConfig {
    /// State layout version.
    pub version: u16,

    /// Root ProtocolState that owns this configuration.
    pub protocol: Pubkey,

    /// Allocation of collected fee proceeds, in basis points.
    pub reserve_bps: u16,
    pub buyback_burn_bps: u16,
    pub liquidity_bps: u16,
    pub company_bps: u16,
    pub founder_bps: u16,

    /// Whether configuration changes are currently permitted.
    pub updates_enabled: bool,

    /// Canonical PDA bump.
    pub bump: u8,

    /// Timestamp of the most recent configuration update.
    pub updated_at: i64,

    /// Reserved space for future compatible fields.
    pub reserved: [u8; 64],
}

impl ProtocolConfig {
    pub const SPACE: usize = 8 +       // Anchor discriminator
        2 +       // version
        32 +      // protocol
        (5 * 2) + // five u16 allocation values
        1 +       // updates_enabled
        1 +       // bump
        8 +       // updated_at
        64; // reserved

    pub fn total_allocation_bps(&self) -> u16 {
        self.reserve_bps
            .saturating_add(self.buyback_burn_bps)
            .saturating_add(self.liquidity_bps)
            .saturating_add(self.company_bps)
            .saturating_add(self.founder_bps)
    }
}
