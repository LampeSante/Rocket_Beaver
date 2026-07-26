use anchor_lang::prelude::*;

#[account]
pub struct TreasuryState {
    /// State layout version for future migrations.
    pub version: u16,

    /// Root ProtocolState controlling this treasury.
    pub protocol: Pubkey,

    /// Mint in which fees are accounted.
    pub settlement_mint: Pubkey,

    /// SPL Token vault holding the treasury's settlement tokens.
    pub settlement_vault: Pubkey,

    /// Total fee units received by the treasury engine.
    pub total_fees_received: u64,

    /// Total fee units assigned to protocol buckets.
    pub total_fees_allocated: u64,

    /// Amounts assigned but not yet transferred or spent.
    pub pending_reserve: u64,
    pub pending_buyback_burn: u64,
    pub pending_liquidity: u64,
    pub pending_company: u64,
    pub pending_founder: u64,

    /// Lifetime allocations for transparent protocol accounting.
    pub lifetime_reserve: u64,
    pub lifetime_buyback_burn: u64,
    pub lifetime_liquidity: u64,
    pub lifetime_company: u64,
    pub lifetime_founder: u64,

    /// Lifetime settlement-token amounts actually released from the treasury.
    pub released_reserve: u64,
    pub released_buyback_burn: u64,
    pub released_liquidity: u64,
    pub released_company: u64,
    pub released_founder: u64,

    /// Last successful fee-processing timestamp.
    pub last_processed_at: i64,

    /// Increments after every successful accounting cycle.
    pub processing_epoch: u64,

    /// Current Survival Waterfall operating stage.
    pub waterfall_stage: u8,

    /// Buyback execution can be paused independently.
    pub buybacks_paused: bool,

    /// Canonical treasury PDA bump.
    pub bump: u8,

    /// Reserved account space for compatible future fields.
    pub reserved: [u8; 24],
}

impl TreasuryState {
    pub const SPACE: usize = 8 +        // Anchor discriminator
        2 +        // version
        32 +       // protocol
        32 +       // settlement_mint
        32 +       // settlement_vault
        (19 * 8) + // u64/i64 accounting fields
        1 +        // waterfall_stage
        1 +        // buybacks_paused
        1 +        // bump
        24; // reserved

    pub fn reserve_accounting_is_valid(&self) -> bool {
        self.lifetime_reserve
            == self
                .pending_reserve
                .checked_add(self.released_reserve)
                .unwrap_or(u64::MAX)
    }

    pub fn buyback_accounting_is_valid(&self) -> bool {
        self.lifetime_buyback_burn
            == self
                .pending_buyback_burn
                .checked_add(self.released_buyback_burn)
                .unwrap_or(u64::MAX)
    }

    pub fn liquidity_accounting_is_valid(&self) -> bool {
        self.lifetime_liquidity
            == self
                .pending_liquidity
                .checked_add(self.released_liquidity)
                .unwrap_or(u64::MAX)
    }

    pub fn company_accounting_is_valid(&self) -> bool {
        self.lifetime_company
            == self
                .pending_company
                .checked_add(self.released_company)
                .unwrap_or(u64::MAX)
    }

    pub fn founder_accounting_is_valid(&self) -> bool {
        self.lifetime_founder
            == self
                .pending_founder
                .checked_add(self.released_founder)
                .unwrap_or(u64::MAX)
    }

    pub fn execution_accounting_is_valid(&self) -> bool {
        self.reserve_accounting_is_valid()
            && self.buyback_accounting_is_valid()
            && self.liquidity_accounting_is_valid()
            && self.company_accounting_is_valid()
            && self.founder_accounting_is_valid()
    }
}
