use anchor_lang::prelude::*;

#[account]
pub struct ProtocolState {
    /// State layout version for future migrations.
    pub version: u16,

    /// Current protocol administration authority.
    pub authority: Pubkey,

    /// Module account references. These remain Pubkey::default()
    /// until each module is initialized.
    pub protocol_config: Pubkey,
    pub treasury_state: Pubkey,
    pub reserve_state: Pubkey,
    pub liquidity_state: Pubkey,
    pub founder_state: Pubkey,
    pub company_state: Pubkey,
    pub buyback_state: Pubkey,

    /// Protocol health indicators.
    pub beaver_score: u16,
    pub dam_level: u8,

    /// Emergency protocol pause.
    pub paused: bool,

    /// Canonical PDA bump.
    pub bump: u8,

    /// Unix timestamp when the protocol was initialized.
    pub initialized_at: i64,

    /// Reserved account space for compatible future fields.
    pub reserved: [u8; 64],
}

impl ProtocolState {
    pub const SPACE: usize = 8 +       // Anchor account discriminator
        2 +       // version
        (32 * 8) + // authority, config, and six module Pubkeys
        2 +       // beaver_score
        1 +       // dam_level
        1 +       // paused
        1 +       // bump
        8 +       // initialized_at
        64; // reserved
}
