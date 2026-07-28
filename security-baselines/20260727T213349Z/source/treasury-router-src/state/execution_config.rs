use anchor_lang::prelude::*;

/// Immutable routing configuration for all protocol-controlled executions.
///
/// This account is created once as a PDA. The program will deliberately expose
/// no instruction capable of modifying its destination addresses after
/// initialization.
#[account]
pub struct ExecutionConfig {
    /// Protocol instance to which this configuration belongs.
    pub protocol_state: Pubkey,

    /// Settlement-token mint accepted by every configured destination.
    pub settlement_mint: Pubkey,

    /// Permanent destination for Protocol Reserve releases.
    pub reserve_destination: Pubkey,

    /// Permanent destination for Automatic Buyback & Burn funding.
    pub buyback_destination: Pubkey,

    /// Permanent destination for Liquidity Growth funding.
    pub liquidity_destination: Pubkey,

    /// Permanent destination for Company/Operations funding.
    pub company_destination: Pubkey,

    /// Permanent destination for Founder Compensation funding.
    pub founder_destination: Pubkey,

    /// Account layout version.
    pub version: u16,

    /// PDA bump.
    pub bump: u8,
}

impl ExecutionConfig {
    pub const VERSION: u16 = 1;

    /// Serialized account data length, excluding Anchor's discriminator.
    pub const LEN: usize = 32 +      // protocol_state
        32 +      // settlement_mint
        32 +      // reserve_destination
        32 +      // buyback_destination
        32 +      // liquidity_destination
        32 +      // company_destination
        32 +      // founder_destination
        2 +       // version
        1; // bump
}
