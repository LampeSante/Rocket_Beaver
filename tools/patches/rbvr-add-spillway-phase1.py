from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import shutil

ROOT = Path.cwd()

ERRORS = ROOT / "programs/treasury-router/src/errors/mod.rs"
EVENTS = ROOT / "programs/treasury-router/src/events/mod.rs"
LIB = ROOT / "programs/treasury-router/src/lib.rs"
INSTRUCTIONS_MOD = ROOT / "programs/treasury-router/src/instructions/mod.rs"
TESTS = ROOT / "tests/rbvr_protocol.ts"

SPILLWAY = (
    ROOT
    / "programs/treasury-router/src/instructions/spillway_release.rs"
)

FILES_TO_BACKUP = [
    ERRORS,
    EVENTS,
    LIB,
    INSTRUCTIONS_MOD,
    TESTS,
]

for path in FILES_TO_BACKUP:
    if not path.exists():
        raise RuntimeError(f"Missing required file: {path}")

if SPILLWAY.exists():
    raise RuntimeError(
        "spillway_release.rs already exists. Refusing to overwrite it."
    )

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup = ROOT / f".spillway-phase1-backup-{timestamp}"
backup.mkdir(parents=True, exist_ok=False)

for source in FILES_TO_BACKUP:
    destination = backup / source.relative_to(ROOT)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)

print(f"Backup created: {backup}")


def insert_before_last_brace(text: str, addition: str) -> str:
    position = text.rfind("}")

    if position == -1:
        raise RuntimeError("Could not locate final closing brace")

    return (
        text[:position].rstrip()
        + "\n\n"
        + addition.rstrip()
        + "\n"
        + text[position:]
    )


try:
    # -------------------------------------------------------------
    # Errors
    # -------------------------------------------------------------
    errors = ERRORS.read_text()

    if "NoDeployableReserveSurplus" in errors:
        raise RuntimeError("Spillway errors already exist")

    new_errors = '''    #[msg("The Reserve Vault does not currently contain deployable surplus.")]
    NoDeployableReserveSurplus,

    #[msg("The Reserve deployment calculation failed closed.")]
    ReserveDeploymentEvaluationFailed,

    #[msg("The Spillway destination is invalid.")]
    InvalidSpillwayDestination,

    #[msg("The Spillway would breach the protected Reserve floor.")]
    ReserveFloorViolation,
'''

    ERRORS.write_text(insert_before_last_brace(errors, new_errors))

    # -------------------------------------------------------------
    # Event
    # -------------------------------------------------------------
    events = EVENTS.read_text()

    if "SpillwayReleaseExecuted" in events:
        raise RuntimeError("Spillway event already exists")

    event_source = '''/// Emitted after a successful permissionless Reserve Spillway release.
#[event]
pub struct SpillwayReleaseExecuted {
    pub protocol: Pubkey,
    pub treasury: Pubkey,
    pub reserve_policy: Pubkey,
    pub reserve_vault: Pubkey,
    pub destination: Pubkey,

    pub reserve_balance_before: u64,
    pub protected_floor: u64,
    pub gross_surplus: u64,
    pub deployed_amount: u64,
    pub reserve_balance_after: u64,

    pub liquidity_reference: u64,
    pub deployment_bps: u16,

    pub lifetime_deployed: u64,
    pub executed_at: i64,
}
'''

    EVENTS.write_text(events.rstrip() + "\n\n" + event_source)

    # -------------------------------------------------------------
    # Spillway instruction
    # -------------------------------------------------------------
    SPILLWAY.write_text(r'''use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, Mint, Token, TokenAccount, TransferChecked,
};

use crate::{
    constants::{
        EXECUTION_CONFIG_SEED, PROTOCOL_SEED, RESERVE_POLICY_SEED,
        RESERVE_VAULT_SEED, TREASURY_SEED,
    },
    engines::reserve_deployment::{
        evaluate_reserve_deployment, ReserveDeploymentPolicy,
    },
    errors::TreasuryRouterError,
    events::SpillwayReleaseExecuted,
    state::{
        ExecutionConfig, ProtocolState, ReservePolicy, TreasuryState,
    },
};

#[derive(Accounts)]
pub struct SpillwayRelease<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol_state.bump,
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
            @ TreasuryRouterError::InvalidTreasuryProtocol,
        constraint = treasury.settlement_mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub treasury: Box<Account<'info, TreasuryState>>,

    #[account(
        seeds = [
            EXECUTION_CONFIG_SEED,
            protocol_state.key().as_ref()
        ],
        bump = execution_config.bump,
        constraint = execution_config.protocol_state
            == protocol_state.key()
                @ TreasuryRouterError::InvalidTreasuryProtocol,
        constraint = execution_config.settlement_mint
            == settlement_mint.key()
                @ TreasuryRouterError::InvalidSettlementMint,
        constraint = execution_config.reserve_destination
            == reserve_vault.key()
                @ TreasuryRouterError::InvalidSettlementVault,
        constraint = execution_config.liquidity_destination
            == liquidity_destination.key()
                @ TreasuryRouterError::InvalidSpillwayDestination
    )]
    pub execution_config: Box<Account<'info, ExecutionConfig>>,

    #[account(
        mut,
        seeds = [
            RESERVE_POLICY_SEED,
            treasury.key().as_ref()
        ],
        bump = reserve_policy.bump,
        constraint = reserve_policy.protocol
            == protocol_state.key()
                @ TreasuryRouterError::InvalidReservePolicyLinkage,
        constraint = reserve_policy.treasury
            == treasury.key()
                @ TreasuryRouterError::InvalidReservePolicyLinkage
    )]
    pub reserve_policy: Box<Account<'info, ReservePolicy>>,

    #[account(
        constraint = settlement_mint.key()
            == treasury.settlement_mint
                @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub settlement_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            RESERVE_VAULT_SEED,
            treasury.key().as_ref()
        ],
        bump,
        constraint = reserve_vault.key()
            == execution_config.reserve_destination
                @ TreasuryRouterError::InvalidSettlementVault,
        constraint = reserve_vault.mint
            == settlement_mint.key()
                @ TreasuryRouterError::InvalidSettlementMint,
        constraint = reserve_vault.owner
            == treasury.key()
                @ TreasuryRouterError::InvalidSettlementVault
    )]
    pub reserve_vault: Box<Account<'info, TokenAccount>>,

    /// Immutable liquidity destination configured during protocol setup.
    ///
    /// Its current settlement-token balance is used as the liquidity
    /// reference for calculating the dynamic Reserve floor.
    #[account(
        mut,
        constraint = liquidity_destination.key()
            == execution_config.liquidity_destination
                @ TreasuryRouterError::InvalidSpillwayDestination,
        constraint = liquidity_destination.key()
            != reserve_vault.key()
                @ TreasuryRouterError::InvalidSpillwayDestination,
        constraint = liquidity_destination.mint
            == settlement_mint.key()
                @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub liquidity_destination: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

/// Deploys only the Reserve Engine-approved surplus to the immutable
/// Liquidity Growth destination.
///
/// This instruction is permissionless. No caller signer controls the amount,
/// source, destination, Reserve floor, deployment rate or cooldown.
pub fn handler(ctx: Context<SpillwayRelease>) -> Result<()> {
    let clock = Clock::get()?;

    require!(
        ctx.accounts
            .reserve_policy
            .cooldown_has_elapsed(clock.unix_timestamp)?,
        TreasuryRouterError::ReserveDeploymentCooldownActive
    );

    let reserve_balance_before = ctx.accounts.reserve_vault.amount;
    let liquidity_reference = ctx.accounts.liquidity_destination.amount;

    let evaluation = evaluate_reserve_deployment(
        reserve_balance_before,
        liquidity_reference,
        ReserveDeploymentPolicy {
            minimum_reserve_floor: ctx
                .accounts
                .reserve_policy
                .minimum_reserve_floor,
            liquidity_floor_bps: ctx
                .accounts
                .reserve_policy
                .liquidity_floor_bps,
            surplus_deployment_bps: ctx
                .accounts
                .reserve_policy
                .surplus_deployment_bps,
        },
    )
    .map_err(|_| {
        error!(TreasuryRouterError::ReserveDeploymentEvaluationFailed)
    })?;

    require!(
        evaluation.deployable_amount > 0,
        TreasuryRouterError::NoDeployableReserveSurplus
    );

    require!(
        evaluation.remaining_reserve >= evaluation.reserve_floor,
        TreasuryRouterError::ReserveFloorViolation
    );

    let protocol_key = ctx.accounts.protocol_state.key();
    let treasury_bump = [ctx.accounts.treasury.bump];

    let treasury_signer_seeds: &[&[u8]] = &[
        TREASURY_SEED,
        protocol_key.as_ref(),
        &treasury_bump,
    ];

    let signer_seeds = &[treasury_signer_seeds];

    let transfer_accounts = TransferChecked {
        from: ctx.accounts.reserve_vault.to_account_info(),
        mint: ctx.accounts.settlement_mint.to_account_info(),
        to: ctx.accounts.liquidity_destination.to_account_info(),
        authority: ctx.accounts.treasury.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        transfer_accounts,
        signer_seeds,
    );

    token::transfer_checked(
        cpi_context,
        evaluation.deployable_amount,
        ctx.accounts.settlement_mint.decimals,
    )?;

    ctx.accounts.reserve_policy.record_deployment(
        evaluation.deployable_amount,
        clock.unix_timestamp,
    )?;

    emit!(SpillwayReleaseExecuted {
        protocol: ctx.accounts.protocol_state.key(),
        treasury: ctx.accounts.treasury.key(),
        reserve_policy: ctx.accounts.reserve_policy.key(),
        reserve_vault: ctx.accounts.reserve_vault.key(),
        destination: ctx.accounts.liquidity_destination.key(),

        reserve_balance_before,
        protected_floor: evaluation.reserve_floor,
        gross_surplus: evaluation.gross_surplus,
        deployed_amount: evaluation.deployable_amount,
        reserve_balance_after: evaluation.remaining_reserve,

        liquidity_reference,
        deployment_bps: ctx
            .accounts
            .reserve_policy
            .surplus_deployment_bps,

        lifetime_deployed: ctx
            .accounts
            .reserve_policy
            .lifetime_deployed,
        executed_at: clock.unix_timestamp,
    });

    msg!("Permissionless Reserve Spillway release completed");
    msg!(
        "Reserve balance before: {}",
        reserve_balance_before
    );
    msg!("Protected Reserve floor: {}", evaluation.reserve_floor);
    msg!("Gross Reserve surplus: {}", evaluation.gross_surplus);
    msg!(
        "Spillway deployed amount: {}",
        evaluation.deployable_amount
    );
    msg!(
        "Reserve balance after: {}",
        evaluation.remaining_reserve
    );
    msg!(
        "Liquidity destination: {}",
        ctx.accounts.liquidity_destination.key()
    );

    Ok(())
}
''')

    # -------------------------------------------------------------
    # Instruction exports
    # -------------------------------------------------------------
    instructions_mod = INSTRUCTIONS_MOD.read_text()

    if "pub mod spillway_release;" in instructions_mod:
        raise RuntimeError("Spillway instruction already exported")

    instructions_mod = instructions_mod.replace(
        "pub mod reserve;",
        "pub mod reserve;\npub mod spillway_release;",
        1,
    )

    instructions_mod = instructions_mod.replace(
        "pub use reserve::*;",
        "pub use reserve::*;\npub use spillway_release::*;",
        1,
    )

    INSTRUCTIONS_MOD.write_text(instructions_mod)

    # -------------------------------------------------------------
    # Program entrypoint
    # -------------------------------------------------------------
    lib = LIB.read_text()

    marker = '''    pub fn authorize_reserve_execution(ctx: Context<AuthorizeReserveExecution>) -> Result<()> {
        instructions::reserve::handler(ctx)
    }
'''

    if marker not in lib:
        raise RuntimeError(
            "Could not locate reserve execution entrypoint"
        )

    addition = marker + '''
    pub fn spillway_release(
        ctx: Context<SpillwayRelease>,
    ) -> Result<()> {
        instructions::spillway_release::handler(ctx)
    }
'''

    LIB.write_text(lib.replace(marker, addition, 1))

    # -------------------------------------------------------------
    # Integration test
    # -------------------------------------------------------------
    tests = TESTS.read_text()

    if "deploys only Reserve surplus through the permissionless Spillway" in tests:
        raise RuntimeError("Spillway integration test already exists")

    close_marker = "\n});"
    insertion_point = tests.rfind(close_marker)

    if insertion_point == -1:
        raise RuntimeError(
            "Could not locate the end of the integration describe block"
        )

    integration_test = r'''
  it("deploys only Reserve surplus through the permissionless Spillway", async () => {
    const [reservePolicyPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("reserve-policy"),
        treasuryStatePda.toBuffer(),
      ],
      PROGRAM_ID,
    );

    const reserveBefore = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const liquidityBefore = await getAccount(
      provider.connection,
      liquidityDestination,
    );

    assert.isTrue(
      reserveBefore.amount > 100_000n,
      "Reserve Vault must contain enough value to test a protected floor",
    );

    /*
     * Lock the absolute floor exactly 100,000 base units beneath the
     * current Reserve balance. With a 50% deployment rate, the first
     * Spillway execution must deploy exactly 50,000 units.
     *
     * The liquidity-floor rate is zero in this integration test so the
     * expected amount depends only on the absolute immutable floor.
     */
    const minimumFloor = reserveBefore.amount - 100_000n;

    await program.methods
      .initializeReservePolicy(
        new anchor.BN(minimumFloor.toString()),
        0,
        5_000,
        new anchor.BN(1),
      )
      .accountsPartial({
        protocolState: protocolStatePda,
        treasury: treasuryStatePda,
        reservePolicy: reservePolicyPda,
        authority,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const policyBefore =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    assert.equal(
      policyBefore.minimumReserveFloor.toString(),
      minimumFloor.toString(),
    );

    assert.equal(policyBefore.liquidityFloorBps, 0);
    assert.equal(policyBefore.surplusDeploymentBps, 5_000);
    assert.equal(policyBefore.lifetimeDeployed.toString(), "0");

    await program.methods
      .spillwayRelease()
      .accountsPartial({
        protocolState: protocolStatePda,
        treasury: treasuryStatePda,
        executionConfig: executionConfigPda,
        reservePolicy: reservePolicyPda,
        settlementMint,
        reserveVault: reserveDestination,
        liquidityDestination,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const reserveAfter = await getAccount(
      provider.connection,
      reserveDestination,
    );

    const liquidityAfter = await getAccount(
      provider.connection,
      liquidityDestination,
    );

    const policyAfter =
      await program.account.reservePolicy.fetch(reservePolicyPda);

    const expectedDeployment = 50_000n;

    assert.equal(
      (reserveBefore.amount - reserveAfter.amount).toString(),
      expectedDeployment.toString(),
      "Spillway must debit only the policy-approved surplus amount",
    );

    assert.equal(
      (liquidityAfter.amount - liquidityBefore.amount).toString(),
      expectedDeployment.toString(),
      "Spillway must send the approved amount to Liquidity Growth",
    );

    assert.isAtLeast(
      Number(reserveAfter.amount),
      Number(minimumFloor),
      "Spillway must never breach the protected Reserve floor",
    );

    assert.equal(
      policyAfter.lifetimeDeployed.toString(),
      expectedDeployment.toString(),
    );

    assert.isAbove(
      Number(policyAfter.lastDeployedAt),
      0,
      "Successful Spillway execution must record its timestamp",
    );
  });
'''

    TESTS.write_text(
        tests[:insertion_point]
        + "\n"
        + integration_test.rstrip()
        + tests[insertion_point:]
    )

    # -------------------------------------------------------------
    # Verification
    # -------------------------------------------------------------
    checks = {
        ERRORS: [
            "NoDeployableReserveSurplus",
            "ReserveDeploymentEvaluationFailed",
            "InvalidSpillwayDestination",
            "ReserveFloorViolation",
        ],
        EVENTS: [
            "pub struct SpillwayReleaseExecuted",
        ],
        LIB: [
            "pub fn spillway_release(",
        ],
        INSTRUCTIONS_MOD: [
            "pub mod spillway_release;",
            "pub use spillway_release::*;",
        ],
        SPILLWAY: [
            "pub struct SpillwayRelease",
            "evaluate_reserve_deployment(",
            "cooldown_has_elapsed",
            "record_deployment(",
            "TransferChecked",
            "SpillwayReleaseExecuted",
        ],
        TESTS: [
            "deploys only Reserve surplus through the permissionless Spillway",
            ".initializeReservePolicy(",
            ".spillwayRelease()",
            "Spillway must never breach the protected Reserve floor",
        ],
    }

    for path, fragments in checks.items():
        content = path.read_text()

        for fragment in fragments:
            if fragment not in content:
                raise RuntimeError(
                    f"Verification failed for {path}: missing {fragment}"
                )

    print("Permissionless Spillway Phase 1 source added.")
    print("Spillway destination: immutable Liquidity Growth account")
    print("Caller signer required: no")
    print("Expected integration tests after patch: 38")

except Exception:
    for source in FILES_TO_BACKUP:
        stored = backup / source.relative_to(ROOT)

        if stored.exists():
            shutil.copy2(stored, source)

    SPILLWAY.unlink(missing_ok=True)
    raise
