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
        self.lifetime_reserve == self.pending_reserve.saturating_add(self.released_reserve)
    }

    pub fn buyback_accounting_is_valid(&self) -> bool {
        self.lifetime_buyback_burn
            == self
                .pending_buyback_burn
                .saturating_add(self.released_buyback_burn)
    }

    pub fn liquidity_accounting_is_valid(&self) -> bool {
        self.lifetime_liquidity
            == self
                .pending_liquidity
                .saturating_add(self.released_liquidity)
    }

    pub fn company_accounting_is_valid(&self) -> bool {
        self.lifetime_company == self.pending_company.saturating_add(self.released_company)
    }

    pub fn founder_accounting_is_valid(&self) -> bool {
        self.lifetime_founder == self.pending_founder.saturating_add(self.released_founder)
    }

    pub fn execution_accounting_is_valid(&self) -> bool {
        self.reserve_accounting_is_valid()
            && self.buyback_accounting_is_valid()
            && self.liquidity_accounting_is_valid()
            && self.company_accounting_is_valid()
            && self.founder_accounting_is_valid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_treasury() -> TreasuryState {
        TreasuryState {
            version: 1,
            protocol: Pubkey::new_unique(),
            settlement_mint: Pubkey::new_unique(),
            settlement_vault: Pubkey::new_unique(),

            total_fees_received: 1_500,
            total_fees_allocated: 1_500,

            pending_reserve: 250,
            pending_buyback_burn: 200,
            pending_liquidity: 300,
            pending_company: 400,
            pending_founder: 100,

            lifetime_reserve: 300,
            lifetime_buyback_burn: 250,
            lifetime_liquidity: 350,
            lifetime_company: 500,
            lifetime_founder: 100,

            released_reserve: 50,
            released_buyback_burn: 50,
            released_liquidity: 50,
            released_company: 100,
            released_founder: 0,

            last_processed_at: 1,
            processing_epoch: 1,
            waterfall_stage: 0,
            buybacks_paused: false,
            bump: 255,
            reserved: [0; 24],
        }
    }

    #[test]
    fn reserve_accounting_is_valid_when_lifetime_equals_pending_plus_released() {
        let treasury = valid_treasury();

        assert!(treasury.reserve_accounting_is_valid());
    }

    #[test]
    fn reserve_accounting_is_invalid_when_values_do_not_balance() {
        let mut treasury = valid_treasury();

        treasury.lifetime_reserve = 299;

        assert!(!treasury.reserve_accounting_is_valid());
    }

    #[test]
    fn reserve_accounting_fails_closed_on_overflow() {
        let mut treasury = valid_treasury();

        treasury.pending_reserve = u64::MAX;
        treasury.released_reserve = 1;
        treasury.lifetime_reserve = 0;

        assert!(!treasury.reserve_accounting_is_valid());
    }

    #[test]
    fn buyback_accounting_is_valid_when_lifetime_equals_pending_plus_released() {
        let treasury = valid_treasury();

        assert!(treasury.buyback_accounting_is_valid());
    }

    #[test]
    fn buyback_accounting_is_invalid_when_values_do_not_balance() {
        let mut treasury = valid_treasury();

        treasury.lifetime_buyback_burn = 249;

        assert!(!treasury.buyback_accounting_is_valid());
    }

    #[test]
    fn buyback_accounting_fails_closed_on_overflow() {
        let mut treasury = valid_treasury();

        treasury.pending_buyback_burn = u64::MAX;
        treasury.released_buyback_burn = 1;
        treasury.lifetime_buyback_burn = 0;

        assert!(!treasury.buyback_accounting_is_valid());
    }

    #[test]
    fn liquidity_accounting_is_valid_when_lifetime_equals_pending_plus_released() {
        let treasury = valid_treasury();

        assert!(treasury.liquidity_accounting_is_valid());
    }

    #[test]
    fn liquidity_accounting_is_invalid_when_values_do_not_balance() {
        let mut treasury = valid_treasury();

        treasury.lifetime_liquidity = 349;

        assert!(!treasury.liquidity_accounting_is_valid());
    }

    #[test]
    fn liquidity_accounting_fails_closed_on_overflow() {
        let mut treasury = valid_treasury();

        treasury.pending_liquidity = u64::MAX;
        treasury.released_liquidity = 1;
        treasury.lifetime_liquidity = 0;

        assert!(!treasury.liquidity_accounting_is_valid());
    }

    #[test]
    fn company_accounting_is_valid_when_lifetime_equals_pending_plus_released() {
        let treasury = valid_treasury();

        assert!(treasury.company_accounting_is_valid());
    }

    #[test]
    fn company_accounting_is_invalid_when_values_do_not_balance() {
        let mut treasury = valid_treasury();

        treasury.lifetime_company = 499;

        assert!(!treasury.company_accounting_is_valid());
    }

    #[test]
    fn company_accounting_fails_closed_on_overflow() {
        let mut treasury = valid_treasury();

        treasury.pending_company = u64::MAX;
        treasury.released_company = 1;
        treasury.lifetime_company = 0;

        assert!(!treasury.company_accounting_is_valid());
    }

    #[test]
    fn founder_accounting_is_valid_when_lifetime_equals_pending_plus_released() {
        let treasury = valid_treasury();

        assert!(treasury.founder_accounting_is_valid());
    }

    #[test]
    fn founder_accounting_is_invalid_when_values_do_not_balance() {
        let mut treasury = valid_treasury();

        treasury.lifetime_founder = 99;

        assert!(!treasury.founder_accounting_is_valid());
    }

    #[test]
    fn founder_accounting_fails_closed_on_overflow() {
        let mut treasury = valid_treasury();

        treasury.pending_founder = u64::MAX;
        treasury.released_founder = 1;
        treasury.lifetime_founder = 0;

        assert!(!treasury.founder_accounting_is_valid());
    }

    #[test]
    fn execution_accounting_is_valid_when_every_bucket_balances() {
        let treasury = valid_treasury();

        assert!(treasury.execution_accounting_is_valid());
    }

    #[test]
    fn execution_accounting_is_invalid_when_reserve_is_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_reserve += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }

    #[test]
    fn execution_accounting_is_invalid_when_buyback_is_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_buyback_burn += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }

    #[test]
    fn execution_accounting_is_invalid_when_liquidity_is_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_liquidity += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }

    #[test]
    fn execution_accounting_is_invalid_when_company_is_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_company += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }

    #[test]
    fn execution_accounting_is_invalid_when_founder_is_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_founder += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }

    #[test]
    fn execution_accounting_is_invalid_when_multiple_buckets_are_corrupted() {
        let mut treasury = valid_treasury();

        treasury.lifetime_reserve += 1;
        treasury.lifetime_liquidity += 1;
        treasury.lifetime_founder += 1;

        assert!(!treasury.execution_accounting_is_valid());
    }

    #[test]
    fn zero_balances_are_valid_accounting() {
        let mut treasury = valid_treasury();

        treasury.pending_reserve = 0;
        treasury.pending_buyback_burn = 0;
        treasury.pending_liquidity = 0;
        treasury.pending_company = 0;
        treasury.pending_founder = 0;

        treasury.lifetime_reserve = 0;
        treasury.lifetime_buyback_burn = 0;
        treasury.lifetime_liquidity = 0;
        treasury.lifetime_company = 0;
        treasury.lifetime_founder = 0;

        treasury.released_reserve = 0;
        treasury.released_buyback_burn = 0;
        treasury.released_liquidity = 0;
        treasury.released_company = 0;
        treasury.released_founder = 0;

        assert!(treasury.execution_accounting_is_valid());
    }
}
