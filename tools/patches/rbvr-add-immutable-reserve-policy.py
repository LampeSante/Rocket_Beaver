from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import re
import shutil

ROOT = Path.cwd()

CONSTANTS = ROOT / "programs/treasury-router/src/constants.rs"
ERRORS = ROOT / "programs/treasury-router/src/errors/mod.rs"
LIB = ROOT / "programs/treasury-router/src/lib.rs"
STATE_MOD = ROOT / "programs/treasury-router/src/state/mod.rs"
INSTRUCTIONS_MOD = ROOT / "programs/treasury-router/src/instructions/mod.rs"

STATE_FILE = ROOT / "programs/treasury-router/src/state/reserve_policy.rs"
INSTRUCTION_FILE = (
    ROOT
    / "programs/treasury-router/src/instructions/initialize_reserve_policy.rs"
)

FILES_TO_BACKUP = [
    CONSTANTS,
    ERRORS,
    LIB,
    STATE_MOD,
    INSTRUCTIONS_MOD,
]

for path in FILES_TO_BACKUP:
    if not path.exists():
        raise RuntimeError(f"Missing required file: {path}")

if STATE_FILE.exists() or INSTRUCTION_FILE.exists():
    raise RuntimeError(
        "Reserve Policy files already exist. Refusing to overwrite them."
    )

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup = ROOT / f".reserve-policy-backup-{timestamp}"
backup.mkdir(parents=True, exist_ok=False)

for source in FILES_TO_BACKUP:
    destination = backup / source.relative_to(ROOT)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)

print(f"Backup created: {backup}")


def append_once(text: str, addition: str, marker: str) -> str:
    if marker in text:
        raise RuntimeError(f"Marker already exists: {marker}")

    return text.rstrip() + "\n\n" + addition.rstrip() + "\n"


def insert_before_last_brace(text: str, addition: str) -> str:
    position = text.rfind("}")

    if position == -1:
        raise RuntimeError("Could not locate final closing brace")

    return text[:position].rstrip() + "\n\n" + addition.rstrip() + "\n" + text[position:]


try:
    # -------------------------------------------------------------
    # Constants
    # -------------------------------------------------------------
    constants = CONSTANTS.read_text()

    constants = append_once(
        constants,
        '''/// Immutable Reserve Policy PDA seed.
pub const RESERVE_POLICY_SEED: &[u8] = b"reserve-policy";

/// Initial Reserve Policy account layout version.
pub const RESERVE_POLICY_VERSION: u16 = 1;

/// Maximum valid basis-point value.
pub const RESERVE_POLICY_BPS_DENOMINATOR: u16 = 10_000;''',
        "RESERVE_POLICY_SEED",
    )

    CONSTANTS.write_text(constants)

    # -------------------------------------------------------------
    # State
    # -------------------------------------------------------------
    STATE_FILE.write_text(r'''use anchor_lang::prelude::*;

use crate::{
    constants::RESERVE_POLICY_BPS_DENOMINATOR,
    errors::TreasuryRouterError,
};

/// Immutable parameters governing future Reserve Vault surplus deployment.
///
/// No instruction is provided to update this account after initialization.
#[account]
pub struct ReservePolicy {
    /// State layout version.
    pub version: u16,

    /// Root protocol linked to this policy.
    pub protocol: Pubkey,

    /// Treasury whose Reserve Vault is governed by this policy.
    pub treasury: Pubkey,

    /// Absolute settlement-token floor that can never be deployed.
    pub minimum_reserve_floor: u64,

    /// Additional floor calculated from a future liquidity reference.
    pub liquidity_floor_bps: u16,

    /// Fraction of current surplus deployable per Spillway execution.
    pub surplus_deployment_bps: u16,

    /// Minimum seconds between successful Spillway executions.
    pub cooldown_seconds: i64,

    /// Timestamp of the latest successful deployment.
    ///
    /// Zero means that no deployment has occurred.
    pub last_deployed_at: i64,

    /// Lifetime value deployed through the Spillway.
    pub lifetime_deployed: u64,

    /// Canonical Reserve Policy PDA bump.
    pub bump: u8,

    /// Reserved space for compatible extensions.
    pub reserved: [u8; 31],
}

impl ReservePolicy {
    pub const SPACE: usize = 8 +  // Anchor discriminator
        2 +                       // version
        32 +                      // protocol
        32 +                      // treasury
        8 +                       // minimum_reserve_floor
        2 +                       // liquidity_floor_bps
        2 +                       // surplus_deployment_bps
        8 +                       // cooldown_seconds
        8 +                       // last_deployed_at
        8 +                       // lifetime_deployed
        1 +                       // bump
        31;                       // reserved

    pub fn validate_parameters(
        minimum_reserve_floor: u64,
        liquidity_floor_bps: u16,
        surplus_deployment_bps: u16,
        cooldown_seconds: i64,
    ) -> Result<()> {
        require!(
            minimum_reserve_floor > 0,
            TreasuryRouterError::InvalidReserveMinimumFloor
        );

        require!(
            liquidity_floor_bps <= RESERVE_POLICY_BPS_DENOMINATOR,
            TreasuryRouterError::InvalidReserveLiquidityFloorRate
        );

        require!(
            surplus_deployment_bps > 0
                && surplus_deployment_bps <= RESERVE_POLICY_BPS_DENOMINATOR,
            TreasuryRouterError::InvalidReserveDeploymentRate
        );

        require!(
            cooldown_seconds > 0,
            TreasuryRouterError::InvalidReserveDeploymentCooldown
        );

        Ok(())
    }

    pub fn cooldown_has_elapsed(&self, current_timestamp: i64) -> Result<bool> {
        if self.last_deployed_at == 0 {
            return Ok(true);
        }

        let next_allowed = self
            .last_deployed_at
            .checked_add(self.cooldown_seconds)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        Ok(current_timestamp >= next_allowed)
    }

    pub fn record_deployment(
        &mut self,
        amount: u64,
        current_timestamp: i64,
    ) -> Result<()> {
        require!(
            amount > 0,
            TreasuryRouterError::InvalidReserveDeploymentAmount
        );

        require!(
            self.cooldown_has_elapsed(current_timestamp)?,
            TreasuryRouterError::ReserveDeploymentCooldownActive
        );

        let updated_lifetime = self
            .lifetime_deployed
            .checked_add(amount)
            .ok_or(TreasuryRouterError::ArithmeticOverflow)?;

        self.lifetime_deployed = updated_lifetime;
        self.last_deployed_at = current_timestamp;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> ReservePolicy {
        ReservePolicy {
            version: 1,
            protocol: Pubkey::new_unique(),
            treasury: Pubkey::new_unique(),
            minimum_reserve_floor: 1_000,
            liquidity_floor_bps: 2_000,
            surplus_deployment_bps: 2_500,
            cooldown_seconds: 86_400,
            last_deployed_at: 0,
            lifetime_deployed: 0,
            bump: 255,
            reserved: [0; 31],
        }
    }

    #[test]
    fn valid_parameters_are_accepted() {
        assert!(
            ReservePolicy::validate_parameters(
                1,
                2_000,
                2_500,
                86_400,
            )
            .is_ok()
        );
    }

    #[test]
    fn zero_minimum_floor_is_rejected() {
        assert!(
            ReservePolicy::validate_parameters(
                0,
                2_000,
                2_500,
                86_400,
            )
            .is_err()
        );
    }

    #[test]
    fn liquidity_floor_above_one_hundred_percent_is_rejected() {
        assert!(
            ReservePolicy::validate_parameters(
                1,
                10_001,
                2_500,
                86_400,
            )
            .is_err()
        );
    }

    #[test]
    fn zero_deployment_rate_is_rejected() {
        assert!(
            ReservePolicy::validate_parameters(
                1,
                2_000,
                0,
                86_400,
            )
            .is_err()
        );
    }

    #[test]
    fn deployment_rate_above_one_hundred_percent_is_rejected() {
        assert!(
            ReservePolicy::validate_parameters(
                1,
                2_000,
                10_001,
                86_400,
            )
            .is_err()
        );
    }

    #[test]
    fn non_positive_cooldown_is_rejected() {
        assert!(
            ReservePolicy::validate_parameters(
                1,
                2_000,
                2_500,
                0,
            )
            .is_err()
        );

        assert!(
            ReservePolicy::validate_parameters(
                1,
                2_000,
                2_500,
                -1,
            )
            .is_err()
        );
    }

    #[test]
    fn first_deployment_is_immediately_eligible() {
        let reserve_policy = policy();

        assert!(reserve_policy.cooldown_has_elapsed(1).unwrap());
    }

    #[test]
    fn cooldown_blocks_early_redeployment() {
        let mut reserve_policy = policy();
        reserve_policy.last_deployed_at = 100;

        assert!(!reserve_policy.cooldown_has_elapsed(86_499).unwrap());
        assert!(reserve_policy.cooldown_has_elapsed(86_500).unwrap());
    }

    #[test]
    fn successful_deployment_updates_timestamp_and_lifetime() {
        let mut reserve_policy = policy();

        reserve_policy.record_deployment(250, 1_000).unwrap();

        assert_eq!(reserve_policy.last_deployed_at, 1_000);
        assert_eq!(reserve_policy.lifetime_deployed, 250);
    }

    #[test]
    fn failed_cooldown_check_does_not_mutate_policy() {
        let mut reserve_policy = policy();
        reserve_policy.last_deployed_at = 1_000;
        reserve_policy.lifetime_deployed = 500;

        let before_timestamp = reserve_policy.last_deployed_at;
        let before_lifetime = reserve_policy.lifetime_deployed;

        assert!(reserve_policy.record_deployment(100, 2_000).is_err());

        assert_eq!(reserve_policy.last_deployed_at, before_timestamp);
        assert_eq!(reserve_policy.lifetime_deployed, before_lifetime);
    }

    #[test]
    fn lifetime_overflow_fails_without_mutation() {
        let mut reserve_policy = policy();
        reserve_policy.lifetime_deployed = u64::MAX;

        assert!(reserve_policy.record_deployment(1, 1_000).is_err());

        assert_eq!(reserve_policy.lifetime_deployed, u64::MAX);
        assert_eq!(reserve_policy.last_deployed_at, 0);
    }
}
''')

    # -------------------------------------------------------------
    # State module
    # -------------------------------------------------------------
    state_mod = STATE_MOD.read_text()

    if "pub mod reserve_policy;" in state_mod:
        raise RuntimeError("Reserve Policy state already exported")

    state_mod = state_mod.replace(
        "pub mod protocol_config;",
        "pub mod protocol_config;\npub mod reserve_policy;",
        1,
    )

    state_mod = state_mod.replace(
        "pub use protocol_config::*;",
        "pub use protocol_config::*;\npub use reserve_policy::*;",
        1,
    )

    STATE_MOD.write_text(state_mod)

    # -------------------------------------------------------------
    # Errors
    # -------------------------------------------------------------
    errors = ERRORS.read_text()

    new_errors = '''    #[msg("The Reserve Policy has already been initialized.")]
    ReservePolicyAlreadyInitialized,

    #[msg("The Reserve Policy minimum floor must be greater than zero.")]
    InvalidReserveMinimumFloor,

    #[msg("The Reserve Policy liquidity-floor rate is invalid.")]
    InvalidReserveLiquidityFloorRate,

    #[msg("The Reserve Policy surplus-deployment rate is invalid.")]
    InvalidReserveDeploymentRate,

    #[msg("The Reserve Policy deployment cooldown must be greater than zero.")]
    InvalidReserveDeploymentCooldown,

    #[msg("The requested Reserve deployment amount must be greater than zero.")]
    InvalidReserveDeploymentAmount,

    #[msg("The Reserve deployment cooldown is still active.")]
    ReserveDeploymentCooldownActive,

    #[msg("The supplied Reserve Policy is not linked to this protocol and treasury.")]
    InvalidReservePolicyLinkage,
'''

    errors = insert_before_last_brace(errors, new_errors)
    ERRORS.write_text(errors)

    # -------------------------------------------------------------
    # Initialization instruction
    # -------------------------------------------------------------
    INSTRUCTION_FILE.write_text(r'''use anchor_lang::prelude::*;

use crate::{
    constants::{
        PROTOCOL_SEED, RESERVE_POLICY_SEED, RESERVE_POLICY_VERSION,
        TREASURY_SEED,
    },
    errors::TreasuryRouterError,
    state::{ProtocolState, ReservePolicy, TreasuryState},
};

#[derive(Accounts)]
pub struct InitializeReservePolicy<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
        has_one = authority,
        constraint = !protocol_state.paused
            @ TreasuryRouterError::ProtocolPaused,
        constraint = protocol_state.treasury_state == treasury.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol
    )]
    pub protocol_state: Box<Account<'info, ProtocolState>>,

    #[account(
        seeds = [
            TREASURY_SEED,
            protocol_state.key().as_ref()
        ],
        bump = treasury.bump,
        constraint = treasury.protocol == protocol_state.key()
            @ TreasuryRouterError::InvalidTreasuryProtocol
    )]
    pub treasury: Box<Account<'info, TreasuryState>>,

    #[account(
        init,
        payer = authority,
        space = ReservePolicy::SPACE,
        seeds = [
            RESERVE_POLICY_SEED,
            treasury.key().as_ref()
        ],
        bump
    )]
    pub reserve_policy: Box<Account<'info, ReservePolicy>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeReservePolicy>,
    minimum_reserve_floor: u64,
    liquidity_floor_bps: u16,
    surplus_deployment_bps: u16,
    cooldown_seconds: i64,
) -> Result<()> {
    ReservePolicy::validate_parameters(
        minimum_reserve_floor,
        liquidity_floor_bps,
        surplus_deployment_bps,
        cooldown_seconds,
    )?;

    let reserve_policy = &mut ctx.accounts.reserve_policy;

    reserve_policy.version = RESERVE_POLICY_VERSION;
    reserve_policy.protocol = ctx.accounts.protocol_state.key();
    reserve_policy.treasury = ctx.accounts.treasury.key();

    reserve_policy.minimum_reserve_floor = minimum_reserve_floor;
    reserve_policy.liquidity_floor_bps = liquidity_floor_bps;
    reserve_policy.surplus_deployment_bps = surplus_deployment_bps;
    reserve_policy.cooldown_seconds = cooldown_seconds;

    reserve_policy.last_deployed_at = 0;
    reserve_policy.lifetime_deployed = 0;
    reserve_policy.bump = ctx.bumps.reserve_policy;
    reserve_policy.reserved = [0; 31];

    msg!("Immutable Reserve Policy initialized");
    msg!("Reserve Policy: {}", reserve_policy.key());
    msg!("Protocol: {}", reserve_policy.protocol);
    msg!("Treasury: {}", reserve_policy.treasury);
    msg!(
        "Minimum reserve floor: {}",
        reserve_policy.minimum_reserve_floor
    );
    msg!(
        "Liquidity floor: {} basis points",
        reserve_policy.liquidity_floor_bps
    );
    msg!(
        "Surplus deployment rate: {} basis points",
        reserve_policy.surplus_deployment_bps
    );
    msg!(
        "Deployment cooldown: {} seconds",
        reserve_policy.cooldown_seconds
    );

    Ok(())
}
''')

    # -------------------------------------------------------------
    # Instructions module
    # -------------------------------------------------------------
    instructions_mod = INSTRUCTIONS_MOD.read_text()

    if "pub mod initialize_reserve_policy;" in instructions_mod:
        raise RuntimeError("Reserve Policy instruction already exported")

    instructions_mod = instructions_mod.replace(
        "pub mod initialize_protocol_config;",
        "pub mod initialize_protocol_config;\npub mod initialize_reserve_policy;",
        1,
    )

    instructions_mod = instructions_mod.replace(
        "pub use initialize_protocol_config::*;",
        "pub use initialize_protocol_config::*;\npub use initialize_reserve_policy::*;",
        1,
    )

    INSTRUCTIONS_MOD.write_text(instructions_mod)

    # -------------------------------------------------------------
    # Program entrypoint
    # -------------------------------------------------------------
    lib = LIB.read_text()

    marker = '''    pub fn initialize_execution_config(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
        instructions::initialize_execution_config::handler(ctx)
    }
'''

    if marker not in lib:
        raise RuntimeError(
            "Could not locate initialize_execution_config entrypoint"
        )

    addition = marker + r'''
    pub fn initialize_reserve_policy(
        ctx: Context<InitializeReservePolicy>,
        minimum_reserve_floor: u64,
        liquidity_floor_bps: u16,
        surplus_deployment_bps: u16,
        cooldown_seconds: i64,
    ) -> Result<()> {
        instructions::initialize_reserve_policy::handler(
            ctx,
            minimum_reserve_floor,
            liquidity_floor_bps,
            surplus_deployment_bps,
            cooldown_seconds,
        )
    }
'''

    LIB.write_text(lib.replace(marker, addition, 1))

    # -------------------------------------------------------------
    # Verification
    # -------------------------------------------------------------
    checks = {
        CONSTANTS: [
            "RESERVE_POLICY_SEED",
            "RESERVE_POLICY_VERSION",
            "RESERVE_POLICY_BPS_DENOMINATOR",
        ],
        STATE_MOD: [
            "pub mod reserve_policy;",
            "pub use reserve_policy::*;",
        ],
        INSTRUCTIONS_MOD: [
            "pub mod initialize_reserve_policy;",
            "pub use initialize_reserve_policy::*;",
        ],
        LIB: [
            "pub fn initialize_reserve_policy(",
        ],
        STATE_FILE: [
            "pub struct ReservePolicy",
            "pub fn validate_parameters",
            "pub fn record_deployment",
        ],
        INSTRUCTION_FILE: [
            "pub struct InitializeReservePolicy",
            "ReservePolicy::validate_parameters",
        ],
    }

    for path, fragments in checks.items():
        content = path.read_text()

        for fragment in fragments:
            if fragment not in content:
                raise RuntimeError(
                    f"Verification failed for {path}: missing {fragment}"
                )

    print("Immutable Reserve Policy source added successfully.")
    print("Live Reserve Vault token movement changed: no")
    print("Policy update instruction added: no")

except Exception:
    for source in FILES_TO_BACKUP:
        stored = backup / source.relative_to(ROOT)

        if stored.exists():
            shutil.copy2(stored, source)

    STATE_FILE.unlink(missing_ok=True)
    INSTRUCTION_FILE.unlink(missing_ok=True)
    raise
