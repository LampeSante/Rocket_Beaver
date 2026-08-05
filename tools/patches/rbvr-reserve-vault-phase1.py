from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
import re
import shutil

ROOT = Path.cwd()

CONSTANTS = ROOT / "programs/treasury-router/src/constants.rs"
INITIALIZE_CONFIG = (
    ROOT
    / "programs/treasury-router/src/instructions/initialize_execution_config.rs"
)
RESERVE = ROOT / "programs/treasury-router/src/instructions/reserve.rs"
TESTS = ROOT / "tests/rbvr_protocol.ts"

FILES = [
    CONSTANTS,
    INITIALIZE_CONFIG,
    RESERVE,
    TESTS,
]

for path in FILES:
    if not path.exists():
        raise RuntimeError(f"Missing required file: {path}")

MARKER = 'pub const RESERVE_VAULT_SEED: &[u8] = b"reserve-vault";'

if MARKER in CONSTANTS.read_text():
    raise RuntimeError("Reserve Vault Phase 1 already appears to be installed.")

timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
backup = ROOT / f".reserve-vault-phase1-backup-{timestamp}"
backup.mkdir(parents=True, exist_ok=False)

for source in FILES:
    destination = backup / source.relative_to(ROOT)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)

print(f"Backup created: {backup}")


def replace_once(
    text: str,
    old: str,
    new: str,
    description: str,
) -> str:
    count = text.count(old)

    if count != 1:
        raise RuntimeError(
            f"{description}: expected exactly 1 occurrence, found {count}"
        )

    return text.replace(old, new, 1)


# -----------------------------------------------------------------
# 1. Add canonical Reserve Vault seed.
# -----------------------------------------------------------------

constants = CONSTANTS.read_text()

constants = replace_once(
    constants,
    'pub const TREASURY_VAULT_SEED: &[u8] = b"treasury-vault";',
    '''pub const TREASURY_VAULT_SEED: &[u8] = b"treasury-vault";

/// Protocol-controlled reserve settlement-token vault.
pub const RESERVE_VAULT_SEED: &[u8] = b"reserve-vault";''',
    "Add RESERVE_VAULT_SEED",
)

CONSTANTS.write_text(constants)


# -----------------------------------------------------------------
# 2. Create Reserve Vault during immutable ExecutionConfig setup.
# -----------------------------------------------------------------

initialize = INITIALIZE_CONFIG.read_text()

initialize = replace_once(
    initialize,
    "use anchor_spl::token::{Mint, TokenAccount};",
    "use anchor_spl::token::{Mint, Token, TokenAccount};",
    "Import Token program",
)

initialize = replace_once(
    initialize,
    """        COMPANY_STATE_SEED, EXECUTION_CONFIG_SEED, EXECUTION_CONFIG_VERSION, FOUNDER_STATE_SEED,
        PROTOCOL_SEED, TREASURY_SEED,
""",
    """        COMPANY_STATE_SEED, EXECUTION_CONFIG_SEED, EXECUTION_CONFIG_VERSION, FOUNDER_STATE_SEED,
        PROTOCOL_SEED, RESERVE_VAULT_SEED, TREASURY_SEED,
""",
    "Import RESERVE_VAULT_SEED",
)

old_reserve_account = """    #[account(
        constraint = reserve_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub reserve_destination: Box<Account<'info, TokenAccount>>,
"""

new_reserve_account = """    /// Canonical protocol-controlled Reserve Vault.
    ///
    /// The account is created exactly once alongside the immutable
    /// ExecutionConfig. TreasuryState is the SPL token authority, so no
    /// external wallet can withdraw reserve assets.
    #[account(
        init,
        payer = authority,
        seeds = [
            RESERVE_VAULT_SEED,
            treasury.key().as_ref()
        ],
        bump,
        token::mint = settlement_mint,
        token::authority = treasury
    )]
    pub reserve_destination: Box<Account<'info, TokenAccount>>,
"""

initialize = replace_once(
    initialize,
    old_reserve_account,
    new_reserve_account,
    "Replace external reserve destination with Reserve Vault PDA",
)

initialize = replace_once(
    initialize,
    """    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
""",
    """    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
""",
    "Add token program to execution-config initialization",
)

initialize = replace_once(
    initialize,
    '    msg!("Reserve destination: {}", reserve_destination);',
    '''    msg!("Protocol Reserve Vault: {}", reserve_destination);
    msg!("Reserve Vault authority: {}", ctx.accounts.treasury.key());''',
    "Update Reserve Vault initialization logs",
)

INITIALIZE_CONFIG.write_text(initialize)


# -----------------------------------------------------------------
# 3. Require canonical Reserve Vault in reserve funding instruction.
# -----------------------------------------------------------------

reserve = RESERVE.read_text()

reserve = replace_once(
    reserve,
    """        COMPANY_STATE_SEED, EXECUTION_CONFIG_SEED, FOUNDER_STATE_SEED, PROTOCOL_CONFIG_SEED,
        PROTOCOL_SEED, TREASURY_SEED,
""",
    """        COMPANY_STATE_SEED, EXECUTION_CONFIG_SEED, FOUNDER_STATE_SEED, PROTOCOL_CONFIG_SEED,
        PROTOCOL_SEED, RESERVE_VAULT_SEED, TREASURY_SEED,
""",
    "Import Reserve Vault seed in reserve instruction",
)

old_reserve_constraints = """    /// Reserve-controlled settlement-token account receiving the release.
    #[account(
        mut,
        constraint = reserve_destination.key()
            == execution_config.reserve_destination
                @ TreasuryRouterError::InvalidExecutionDestination,
        constraint = reserve_destination.key() != settlement_vault.key()
            @ TreasuryRouterError::InvalidExecutionDestination,
        constraint = reserve_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint
    )]
    pub reserve_destination: Box<Account<'info, TokenAccount>>,
"""

new_reserve_constraints = """    /// Canonical protocol-controlled Reserve Vault receiving reserve funding.
    #[account(
        mut,
        seeds = [
            RESERVE_VAULT_SEED,
            treasury.key().as_ref()
        ],
        bump,
        constraint = reserve_destination.key()
            == execution_config.reserve_destination
                @ TreasuryRouterError::InvalidExecutionDestination,
        constraint = reserve_destination.key() != settlement_vault.key()
            @ TreasuryRouterError::InvalidExecutionDestination,
        constraint = reserve_destination.mint == settlement_mint.key()
            @ TreasuryRouterError::InvalidSettlementMint,
        constraint = reserve_destination.owner == treasury.key()
            @ TreasuryRouterError::InvalidExecutionDestination
    )]
    pub reserve_destination: Box<Account<'info, TokenAccount>>,
"""

reserve = replace_once(
    reserve,
    old_reserve_constraints,
    new_reserve_constraints,
    "Lock reserve instruction to canonical Reserve Vault",
)

reserve = reserve.replace(
    "Reserve settlement-token execution completed",
    "Reserve Vault funding completed",
)

reserve = reserve.replace(
    "Reserve destination: {}",
    "Protocol Reserve Vault: {}",
)

RESERVE.write_text(reserve)


# -----------------------------------------------------------------
# 4. Update integration tests to derive and use the Reserve Vault PDA.
# -----------------------------------------------------------------

tests = TESTS.read_text()

# Remove the now-unused external reserve recipient keypair.
reserve_owner_pattern = re.compile(
    r"\n\s*const reserveRecipientOwner = Keypair\.generate\(\);\n"
)

tests, removed_owner_count = reserve_owner_pattern.subn("\n", tests, count=1)

if removed_owner_count != 1:
    raise RuntimeError(
        "Could not remove exactly one reserveRecipientOwner declaration"
    )

# Add Reserve Vault PDA derivation after settlement vault derivation.
settlement_derivation_pattern = re.compile(
    r"""(\s*\[settlementVaultPda\] = PublicKey\.findProgramAddressSync\(
\s*\[Buffer\.from\("treasury-vault"\), treasuryStatePda\.toBuffer\(\)\],
\s*PROGRAM_ID,
\s*\);
)"""
)

reserve_derivation = r'''\1
    [reserveDestination] = PublicKey.findProgramAddressSync(
      [Buffer.from("reserve-vault"), treasuryStatePda.toBuffer()],
      PROGRAM_ID,
    );
'''

tests, derivation_count = settlement_derivation_pattern.subn(
    reserve_derivation,
    tests,
    count=1,
)

if derivation_count != 1:
    raise RuntimeError(
        "Could not insert Reserve Vault PDA derivation exactly once"
    )

# Remove creation of the former externally owned Reserve ATA.
old_reserve_ata = """    reserveDestination = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        settlementMint,
        reserveRecipientOwner.publicKey,
      )
    ).address;

"""

if old_reserve_ata not in tests:
    raise RuntimeError(
        "Could not locate the exact external reserve ATA creation block"
    )

tests = tests.replace(old_reserve_ata, "", 1)

# Add tokenProgram to every initializeExecutionConfig account block.
method_marker = ".initializeExecutionConfig()"
search_position = 0
patched_calls = 0

while True:
    method_position = tests.find(method_marker, search_position)

    if method_position == -1:
        break

    accounts_start = tests.find(".accountsPartial({", method_position)

    if accounts_start == -1:
        raise RuntimeError(
            "initializeExecutionConfig call has no accountsPartial block"
        )

    block_end = tests.find("      })", accounts_start)

    if block_end == -1:
        raise RuntimeError(
            "Could not locate end of initializeExecutionConfig accounts block"
        )

    block = tests[accounts_start:block_end]

    if "tokenProgram: TOKEN_PROGRAM_ID" not in block:
        system_line = "        systemProgram: SystemProgram.programId,"

        if system_line not in block:
            raise RuntimeError(
                "initializeExecutionConfig block is missing systemProgram"
            )

        patched_block = block.replace(
            system_line,
            """        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,""",
            1,
        )

        tests = (
            tests[:accounts_start]
            + patched_block
            + tests[block_end:]
        )

        patched_calls += 1
        search_position = accounts_start + len(patched_block)
    else:
        search_position = block_end

if patched_calls == 0:
    raise RuntimeError(
        "No initializeExecutionConfig account blocks required patching"
    )

# Add explicit Reserve Vault ownership checks to the successful initialization test.
config_assertion = """    assert.equal(
      config.reserveDestination.toBase58(),
      reserveDestination.toBase58(),
    );
"""

vault_assertions = """    assert.equal(
      config.reserveDestination.toBase58(),
      reserveDestination.toBase58(),
    );

    const reserveVault = await getAccount(
      provider.connection,
      reserveDestination,
    );

    assert.equal(
      reserveVault.mint.toBase58(),
      settlementMint.toBase58(),
      "Reserve Vault must use the settlement mint",
    );

    assert.equal(
      reserveVault.owner.toBase58(),
      treasuryStatePda.toBase58(),
      "TreasuryState must be the Reserve Vault token authority",
    );
"""

tests = replace_once(
    tests,
    config_assertion,
    vault_assertions,
    "Add Reserve Vault integration assertions",
)

# Strengthen the locked-PDA test without adding another Mocha test.
locked_pda_assertion = """    assert.notEqual(founderStatePda.toBase58(), companyStatePda.toBase58());
"""

locked_pda_replacement = """    assert.notEqual(founderStatePda.toBase58(), companyStatePda.toBase58());

    assert.equal(
      reserveDestination.toBase58(),
      PublicKey.findProgramAddressSync(
        [Buffer.from("reserve-vault"), treasuryStatePda.toBuffer()],
        PROGRAM_ID,
      )[0].toBase58(),
      "Reserve Vault must use the canonical PDA",
    );
"""

tests = replace_once(
    tests,
    locked_pda_assertion,
    locked_pda_replacement,
    "Add canonical Reserve Vault PDA assertion",
)

TESTS.write_text(tests)


# -----------------------------------------------------------------
# 5. Verification.
# -----------------------------------------------------------------

verification = {
    CONSTANTS: [
        'RESERVE_VAULT_SEED: &[u8] = b"reserve-vault"',
    ],
    INITIALIZE_CONFIG: [
        "token::authority = treasury",
        "RESERVE_VAULT_SEED",
        "pub token_program: Program<'info, Token>",
        "Protocol Reserve Vault",
    ],
    RESERVE: [
        "RESERVE_VAULT_SEED",
        "reserve_destination.owner == treasury.key()",
        "Reserve Vault funding completed",
    ],
    TESTS: [
        'Buffer.from("reserve-vault")',
        "Reserve Vault must use the canonical PDA",
        "TreasuryState must be the Reserve Vault token authority",
        "tokenProgram: TOKEN_PROGRAM_ID",
    ],
}

for path, fragments in verification.items():
    content = path.read_text()

    for fragment in fragments:
        if fragment not in content:
            raise RuntimeError(
                f"Verification failed for {path}: missing {fragment}"
            )

if "reserveRecipientOwner" in TESTS.read_text():
    raise RuntimeError(
        "Stale reserveRecipientOwner reference remains in integration tests"
    )

print("Reserve Vault Phase 1 patch completed.")
print(f"Execution-config calls updated: {patched_calls}")
print("Live reserve destination is now a treasury-owned PDA.")
