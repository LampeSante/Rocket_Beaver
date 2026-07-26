use anchor_lang::prelude::*;

use crate::{errors::TreasuryRouterError, state::TreasuryState};

/// Result returned by the Buyback Engine.
pub struct BuybackDeposit {
    /// Amount credited to the pending buyback-and-burn bucket.
    pub amount: u64,

    /// Updated pending buyback-and-burn balance.
    pub pending_balance: u64,

    /// Updated lifetime buyback-and-burn allocation.
    pub lifetime_total: u64,
}

/// Credits a buyback-and-burn allocation to TreasuryState.
///
/// This function performs protocol accounting only. Actual token purchase,
/// burn execution, slippage controls, and DEX interaction will be handled
/// by later execution instructions.
pub fn deposit(treasury: &mut TreasuryState, amount: u64) -> Result<BuybackDeposit> {
    treasury.pending_buyback_burn = treasury
        .pending_buyback_burn
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.lifetime_buyback_burn = treasury
        .lifetime_buyback_burn
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    Ok(BuybackDeposit {
        amount,
        pending_balance: treasury.pending_buyback_burn,
        lifetime_total: treasury.lifetime_buyback_burn,
    })
}
