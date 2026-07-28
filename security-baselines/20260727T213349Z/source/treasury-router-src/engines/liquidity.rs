use anchor_lang::prelude::*;

use crate::{errors::TreasuryRouterError, state::TreasuryState};

/// Result returned by the Liquidity Engine.
pub struct LiquidityDeposit {
    /// Amount credited to pending liquidity.
    pub amount: u64,

    /// Updated pending liquidity balance.
    pub pending_balance: u64,

    /// Updated lifetime liquidity allocation.
    pub lifetime_total: u64,
}

/// Credits a liquidity allocation to TreasuryState.
///
/// This function only performs protocol accounting. Actual deployment
/// into a liquidity pool will be handled by a later execution instruction.
pub fn deposit(treasury: &mut TreasuryState, amount: u64) -> Result<LiquidityDeposit> {
    treasury.pending_liquidity = treasury
        .pending_liquidity
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    treasury.lifetime_liquidity = treasury
        .lifetime_liquidity
        .checked_add(amount)
        .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

    Ok(LiquidityDeposit {
        amount,
        pending_balance: treasury.pending_liquidity,
        lifetime_total: treasury.lifetime_liquidity,
    })
}
