use anchor_lang::prelude::*;

use crate::{
    constants::{
        COMPANY_STATE_SEED, COMPANY_STATE_VERSION, EXECUTION_CONFIG_SEED, EXECUTION_CONFIG_VERSION,
        FOUNDER_STATE_SEED, FOUNDER_STATE_VERSION, PROTOCOL_CONFIG_SEED, PROTOCOL_CONFIG_VERSION,
        PROTOCOL_SEED, PROTOCOL_VERSION, TREASURY_SEED, TREASURY_VAULT_SEED, TREASURY_VERSION,
    },
    state::{
        CompanyState, ExecutionConfig, FounderState, ProtocolConfig, ProtocolState, TreasuryState,
    },
};

/// Stable identifiers for Integrity Firewall failures.
///
/// Existing numeric values must never be reordered after deployment because
/// monitoring software may interpret the failure mask using these values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IntegrityInvariant {
    ProgramId = 0,

    ProtocolPda = 1,
    ProtocolBump = 2,
    ProtocolVersion = 3,
    ProtocolAuthority = 4,

    ProtocolConfigPda = 5,
    ProtocolConfigBump = 6,
    ProtocolConfigVersion = 7,
    ProtocolConfigLink = 8,

    TreasuryPda = 9,
    TreasuryBump = 10,
    TreasuryVersion = 11,
    TreasuryLink = 12,

    TreasuryVaultPda = 13,
    TreasuryVaultLink = 14,
    TreasuryVaultMint = 15,
    TreasuryVaultOwner = 16,

    FounderPda = 17,
    FounderBump = 18,
    FounderVersion = 19,
    FounderLink = 20,

    CompanyPda = 21,
    CompanyBump = 22,
    CompanyVersion = 23,
    CompanyLink = 24,

    ExecutionConfigPda = 25,
    ExecutionConfigBump = 26,
    ExecutionConfigVersion = 27,
    ExecutionConfigLink = 28,

    SettlementMintLink = 29,
    DestinationMints = 30,
    FounderDestinationOwner = 31,
    CompanyDestinationOwner = 32,
    DestinationIsolation = 33,
}

impl IntegrityInvariant {
    pub const fn mask(self) -> u64 {
        1_u64 << self as u8
    }
}

/// Read-only SPL token-account facts supplied to the firewall.
///
/// This is intentionally plain data. The firewall does not receive mutable
/// accounts, signer authority, or CPI capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenAccountFacts {
    pub key: Pubkey,
    pub mint: Pubkey,
    pub owner: Pubkey,
}

/// Canonical account keys supplied to the firewall.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntegrityAccountKeys {
    pub protocol: Pubkey,
    pub protocol_config: Pubkey,
    pub treasury: Pubkey,
    pub founder: Pubkey,
    pub company: Pubkey,
    pub execution_config: Pubkey,
}

/// Permanent execution destinations supplied to the firewall.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntegrityDestinations {
    pub reserve: TokenAccountFacts,
    pub buyback: TokenAccountFacts,
    pub liquidity: TokenAccountFacts,
    pub company: TokenAccountFacts,
    pub founder: TokenAccountFacts,
}

/// Read-only Integrity Firewall report.
///
/// A zero failure mask means every checked invariant passed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntegrityReport {
    pub healthy: bool,
    pub failure_mask: u64,
}

/// Evaluates account identity and wiring before execution.
///
/// This function:
/// - does not mutate state;
/// - does not transfer tokens;
/// - does not authorize releases;
/// - does not invoke CPI;
/// - does not replace ExecutionGuard.
///
/// It verifies that the live accounts supplied to execution still match the
/// canonical RBVR account architecture.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_integrity(
    program_id: Pubkey,
    keys: IntegrityAccountKeys,
    protocol: &ProtocolState,
    protocol_config: &ProtocolConfig,
    treasury: &TreasuryState,
    treasury_vault: TokenAccountFacts,
    founder: &FounderState,
    company: &CompanyState,
    execution_config: &ExecutionConfig,
    destinations: IntegrityDestinations,
) -> IntegrityReport {
    let mut failure_mask = 0_u64;

    let mut fail = |invariant: IntegrityInvariant| {
        failure_mask |= invariant.mask();
    };

    /*
     * Program identity
     */
    if program_id != crate::ID {
        fail(IntegrityInvariant::ProgramId);
    }

    /*
     * ProtocolState
     */
    let (expected_protocol, expected_protocol_bump) =
        Pubkey::find_program_address(&[PROTOCOL_SEED], &program_id);

    if keys.protocol != expected_protocol {
        fail(IntegrityInvariant::ProtocolPda);
    }

    if protocol.bump != expected_protocol_bump {
        fail(IntegrityInvariant::ProtocolBump);
    }

    if protocol.version != PROTOCOL_VERSION {
        fail(IntegrityInvariant::ProtocolVersion);
    }

    if protocol.authority == Pubkey::default() {
        fail(IntegrityInvariant::ProtocolAuthority);
    }

    /*
     * ProtocolConfig
     */
    let (expected_protocol_config, expected_protocol_config_bump) =
        Pubkey::find_program_address(&[PROTOCOL_CONFIG_SEED, keys.protocol.as_ref()], &program_id);

    if keys.protocol_config != expected_protocol_config {
        fail(IntegrityInvariant::ProtocolConfigPda);
    }

    if protocol_config.bump != expected_protocol_config_bump {
        fail(IntegrityInvariant::ProtocolConfigBump);
    }

    if protocol_config.version != PROTOCOL_CONFIG_VERSION {
        fail(IntegrityInvariant::ProtocolConfigVersion);
    }

    if protocol.protocol_config != keys.protocol_config || protocol_config.protocol != keys.protocol
    {
        fail(IntegrityInvariant::ProtocolConfigLink);
    }

    /*
     * TreasuryState
     */
    let (expected_treasury, expected_treasury_bump) =
        Pubkey::find_program_address(&[TREASURY_SEED, keys.protocol.as_ref()], &program_id);

    if keys.treasury != expected_treasury {
        fail(IntegrityInvariant::TreasuryPda);
    }

    if treasury.bump != expected_treasury_bump {
        fail(IntegrityInvariant::TreasuryBump);
    }

    if treasury.version != TREASURY_VERSION {
        fail(IntegrityInvariant::TreasuryVersion);
    }

    if protocol.treasury_state != keys.treasury || treasury.protocol != keys.protocol {
        fail(IntegrityInvariant::TreasuryLink);
    }

    /*
     * Treasury settlement vault
     */
    let (expected_treasury_vault, _) =
        Pubkey::find_program_address(&[TREASURY_VAULT_SEED, keys.treasury.as_ref()], &program_id);

    if treasury_vault.key != expected_treasury_vault {
        fail(IntegrityInvariant::TreasuryVaultPda);
    }

    if treasury.settlement_vault != treasury_vault.key {
        fail(IntegrityInvariant::TreasuryVaultLink);
    }

    if treasury_vault.mint != treasury.settlement_mint {
        fail(IntegrityInvariant::TreasuryVaultMint);
    }

    if treasury_vault.owner != keys.treasury {
        fail(IntegrityInvariant::TreasuryVaultOwner);
    }

    /*
     * FounderState
     */
    let (expected_founder, expected_founder_bump) =
        Pubkey::find_program_address(&[FOUNDER_STATE_SEED, keys.protocol.as_ref()], &program_id);

    if keys.founder != expected_founder {
        fail(IntegrityInvariant::FounderPda);
    }

    if founder.bump != expected_founder_bump {
        fail(IntegrityInvariant::FounderBump);
    }

    if founder.version != FOUNDER_STATE_VERSION {
        fail(IntegrityInvariant::FounderVersion);
    }

    if protocol.founder_state != keys.founder || founder.protocol != keys.protocol {
        fail(IntegrityInvariant::FounderLink);
    }

    /*
     * CompanyState
     */
    let (expected_company, expected_company_bump) =
        Pubkey::find_program_address(&[COMPANY_STATE_SEED, keys.protocol.as_ref()], &program_id);

    if keys.company != expected_company {
        fail(IntegrityInvariant::CompanyPda);
    }

    if company.bump != expected_company_bump {
        fail(IntegrityInvariant::CompanyBump);
    }

    if company.version != COMPANY_STATE_VERSION {
        fail(IntegrityInvariant::CompanyVersion);
    }

    if protocol.company_state != keys.company || company.protocol != keys.protocol {
        fail(IntegrityInvariant::CompanyLink);
    }

    /*
     * Immutable ExecutionConfig
     */
    let (expected_execution_config, expected_execution_config_bump) = Pubkey::find_program_address(
        &[EXECUTION_CONFIG_SEED, keys.protocol.as_ref()],
        &program_id,
    );

    if keys.execution_config != expected_execution_config {
        fail(IntegrityInvariant::ExecutionConfigPda);
    }

    if execution_config.bump != expected_execution_config_bump {
        fail(IntegrityInvariant::ExecutionConfigBump);
    }

    if execution_config.version != EXECUTION_CONFIG_VERSION {
        fail(IntegrityInvariant::ExecutionConfigVersion);
    }

    if execution_config.protocol_state != keys.protocol {
        fail(IntegrityInvariant::ExecutionConfigLink);
    }

    /*
     * Settlement mint linkage
     */
    if execution_config.settlement_mint != treasury.settlement_mint {
        fail(IntegrityInvariant::SettlementMintLink);
    }

    /*
     * Permanent destination identity
     */
    if destinations.reserve.key != execution_config.reserve_destination
        || destinations.buyback.key != execution_config.buyback_destination
        || destinations.liquidity.key != execution_config.liquidity_destination
        || destinations.company.key != execution_config.company_destination
        || destinations.founder.key != execution_config.founder_destination
    {
        fail(IntegrityInvariant::ExecutionConfigLink);
    }

    /*
     * All destination accounts must use the settlement mint.
     */
    let settlement_mint = treasury.settlement_mint;

    if destinations.reserve.mint != settlement_mint
        || destinations.buyback.mint != settlement_mint
        || destinations.liquidity.mint != settlement_mint
        || destinations.company.mint != settlement_mint
        || destinations.founder.mint != settlement_mint
    {
        fail(IntegrityInvariant::DestinationMints);
    }

    /*
     * Recipient ownership
     */
    if destinations.founder.owner != founder.recipient {
        fail(IntegrityInvariant::FounderDestinationOwner);
    }

    if destinations.company.owner != company.recipient {
        fail(IntegrityInvariant::CompanyDestinationOwner);
    }

    /*
     * Destination isolation:
     * - no destination may equal the treasury vault;
     * - every destination must be unique.
     */
    let destination_keys = [
        destinations.reserve.key,
        destinations.buyback.key,
        destinations.liquidity.key,
        destinations.company.key,
        destinations.founder.key,
    ];

    let points_to_vault = destination_keys.contains(&treasury_vault.key);

    let mut duplicate_found = false;

    for left in 0..destination_keys.len() {
        for right in (left + 1)..destination_keys.len() {
            if destination_keys[left] == destination_keys[right] {
                duplicate_found = true;
            }
        }
    }

    if points_to_vault || duplicate_found {
        fail(IntegrityInvariant::DestinationIsolation);
    }

    IntegrityReport {
        healthy: failure_mask == 0,
        failure_mask,
    }
}

/// Returns true when a specific Integrity Firewall invariant failed.
pub const fn integrity_invariant_failed(failure_mask: u64, invariant: IntegrityInvariant) -> bool {
    failure_mask & invariant.mask() != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        keys: IntegrityAccountKeys,
        protocol: ProtocolState,
        protocol_config: ProtocolConfig,
        treasury: TreasuryState,
        treasury_vault: TokenAccountFacts,
        founder: FounderState,
        company: CompanyState,
        execution_config: ExecutionConfig,
        destinations: IntegrityDestinations,
    }

    fn valid_fixture() -> Fixture {
        let program_id = crate::ID;

        let (protocol_key, protocol_bump) =
            Pubkey::find_program_address(&[PROTOCOL_SEED], &program_id);

        let (protocol_config_key, protocol_config_bump) = Pubkey::find_program_address(
            &[PROTOCOL_CONFIG_SEED, protocol_key.as_ref()],
            &program_id,
        );

        let (treasury_key, treasury_bump) =
            Pubkey::find_program_address(&[TREASURY_SEED, protocol_key.as_ref()], &program_id);

        let (treasury_vault_key, _) = Pubkey::find_program_address(
            &[TREASURY_VAULT_SEED, treasury_key.as_ref()],
            &program_id,
        );

        let (founder_key, founder_bump) =
            Pubkey::find_program_address(&[FOUNDER_STATE_SEED, protocol_key.as_ref()], &program_id);

        let (company_key, company_bump) =
            Pubkey::find_program_address(&[COMPANY_STATE_SEED, protocol_key.as_ref()], &program_id);

        let (execution_config_key, execution_config_bump) = Pubkey::find_program_address(
            &[EXECUTION_CONFIG_SEED, protocol_key.as_ref()],
            &program_id,
        );

        let settlement_mint = Pubkey::new_unique();
        let founder_recipient = Pubkey::new_unique();
        let company_recipient = Pubkey::new_unique();

        let reserve_destination = Pubkey::new_unique();
        let buyback_destination = Pubkey::new_unique();
        let liquidity_destination = Pubkey::new_unique();
        let company_destination = Pubkey::new_unique();
        let founder_destination = Pubkey::new_unique();

        let protocol = ProtocolState {
            version: PROTOCOL_VERSION,
            authority: Pubkey::new_unique(),
            protocol_config: protocol_config_key,
            treasury_state: treasury_key,
            reserve_state: Pubkey::default(),
            liquidity_state: Pubkey::default(),
            founder_state: founder_key,
            company_state: company_key,
            buyback_state: Pubkey::default(),
            beaver_score: 0,
            dam_level: 0,
            paused: false,
            bump: protocol_bump,
            initialized_at: 1,
            reserved: [0; 64],
        };

        let protocol_config = ProtocolConfig {
            version: PROTOCOL_CONFIG_VERSION,
            protocol: protocol_key,
            reserve_bps: 3_000,
            buyback_burn_bps: 2_000,
            liquidity_bps: 2_000,
            company_bps: 2_000,
            founder_bps: 1_000,
            updates_enabled: false,
            bump: protocol_config_bump,
            updated_at: 1,
            reserved: [0; 64],
        };

        let treasury = TreasuryState {
            version: TREASURY_VERSION,
            protocol: protocol_key,
            settlement_mint,
            settlement_vault: treasury_vault_key,

            total_fees_received: 1_000,
            total_fees_allocated: 1_000,

            pending_reserve: 300,
            pending_buyback_burn: 200,
            pending_liquidity: 200,
            pending_company: 200,
            pending_founder: 100,

            lifetime_reserve: 300,
            lifetime_buyback_burn: 200,
            lifetime_liquidity: 200,
            lifetime_company: 200,
            lifetime_founder: 100,

            released_reserve: 0,
            released_buyback_burn: 0,
            released_liquidity: 0,
            released_company: 0,
            released_founder: 0,

            last_processed_at: 1,
            processing_epoch: 1,
            waterfall_stage: 0,
            buybacks_paused: false,
            bump: treasury_bump,
            reserved: [0; 24],
        };

        let founder = FounderState {
            version: FOUNDER_STATE_VERSION,
            protocol: protocol_key,
            recipient: founder_recipient,
            period_cap: 1_000,
            earned_current_period: 0,
            lifetime_earned: 0,
            period_started_at: 1,
            period_duration: 1,
            current_tier: 0,
            enabled: true,
            bump: founder_bump,
            reserved: [0; 64],
        };

        let company = CompanyState {
            version: COMPANY_STATE_VERSION,
            protocol: protocol_key,
            recipient: company_recipient,
            period_cap: 1_000,
            spent_current_period: 0,
            lifetime_spent: 0,
            period_started_at: 1,
            period_duration: 1,
            enabled: true,
            bump: company_bump,
            reserved: [0; 64],
        };

        let execution_config = ExecutionConfig {
            protocol_state: protocol_key,
            settlement_mint,
            reserve_destination,
            buyback_destination,
            liquidity_destination,
            company_destination,
            founder_destination,
            version: EXECUTION_CONFIG_VERSION,
            bump: execution_config_bump,
        };

        let treasury_vault = TokenAccountFacts {
            key: treasury_vault_key,
            mint: settlement_mint,
            owner: treasury_key,
        };

        let destinations = IntegrityDestinations {
            reserve: TokenAccountFacts {
                key: reserve_destination,
                mint: settlement_mint,
                owner: Pubkey::new_unique(),
            },
            buyback: TokenAccountFacts {
                key: buyback_destination,
                mint: settlement_mint,
                owner: Pubkey::new_unique(),
            },
            liquidity: TokenAccountFacts {
                key: liquidity_destination,
                mint: settlement_mint,
                owner: Pubkey::new_unique(),
            },
            company: TokenAccountFacts {
                key: company_destination,
                mint: settlement_mint,
                owner: company_recipient,
            },
            founder: TokenAccountFacts {
                key: founder_destination,
                mint: settlement_mint,
                owner: founder_recipient,
            },
        };

        Fixture {
            keys: IntegrityAccountKeys {
                protocol: protocol_key,
                protocol_config: protocol_config_key,
                treasury: treasury_key,
                founder: founder_key,
                company: company_key,
                execution_config: execution_config_key,
            },
            protocol,
            protocol_config,
            treasury,
            treasury_vault,
            founder,
            company,
            execution_config,
            destinations,
        }
    }

    fn evaluate_fixture(fixture: &Fixture) -> IntegrityReport {
        evaluate_integrity(
            crate::ID,
            fixture.keys,
            &fixture.protocol,
            &fixture.protocol_config,
            &fixture.treasury,
            fixture.treasury_vault,
            &fixture.founder,
            &fixture.company,
            &fixture.execution_config,
            fixture.destinations,
        )
    }

    #[test]
    fn valid_architecture_passes_every_integrity_check() {
        let fixture = valid_fixture();
        let report = evaluate_fixture(&fixture);

        assert!(report.healthy);
        assert_eq!(report.failure_mask, 0);
    }

    #[test]
    fn wrong_program_id_fails_closed() {
        let fixture = valid_fixture();

        let report = evaluate_integrity(
            Pubkey::new_unique(),
            fixture.keys,
            &fixture.protocol,
            &fixture.protocol_config,
            &fixture.treasury,
            fixture.treasury_vault,
            &fixture.founder,
            &fixture.company,
            &fixture.execution_config,
            fixture.destinations,
        );

        assert!(!report.healthy);
        assert!(integrity_invariant_failed(
            report.failure_mask,
            IntegrityInvariant::ProgramId
        ));
    }

    #[test]
    fn corrupted_treasury_bump_is_detected() {
        let mut fixture = valid_fixture();
        fixture.treasury.bump = fixture.treasury.bump.wrapping_add(1);

        let report = evaluate_fixture(&fixture);

        assert!(integrity_invariant_failed(
            report.failure_mask,
            IntegrityInvariant::TreasuryBump
        ));
    }

    #[test]
    fn wrong_treasury_vault_owner_is_detected() {
        let mut fixture = valid_fixture();
        fixture.treasury_vault.owner = Pubkey::new_unique();

        let report = evaluate_fixture(&fixture);

        assert!(integrity_invariant_failed(
            report.failure_mask,
            IntegrityInvariant::TreasuryVaultOwner
        ));
    }

    #[test]
    fn wrong_destination_mint_is_detected() {
        let mut fixture = valid_fixture();
        fixture.destinations.liquidity.mint = Pubkey::new_unique();

        let report = evaluate_fixture(&fixture);

        assert!(integrity_invariant_failed(
            report.failure_mask,
            IntegrityInvariant::DestinationMints
        ));
    }

    #[test]
    fn wrong_founder_destination_owner_is_detected() {
        let mut fixture = valid_fixture();
        fixture.destinations.founder.owner = Pubkey::new_unique();

        let report = evaluate_fixture(&fixture);

        assert!(integrity_invariant_failed(
            report.failure_mask,
            IntegrityInvariant::FounderDestinationOwner
        ));
    }

    #[test]
    fn duplicate_destinations_are_detected() {
        let mut fixture = valid_fixture();

        fixture.destinations.buyback.key = fixture.destinations.reserve.key;

        let report = evaluate_fixture(&fixture);

        assert!(integrity_invariant_failed(
            report.failure_mask,
            IntegrityInvariant::DestinationIsolation
        ));
    }

    #[test]
    fn destination_pointing_to_treasury_vault_is_detected() {
        let mut fixture = valid_fixture();

        fixture.destinations.reserve.key = fixture.treasury_vault.key;

        let report = evaluate_fixture(&fixture);

        assert!(integrity_invariant_failed(
            report.failure_mask,
            IntegrityInvariant::DestinationIsolation
        ));
    }

    #[test]
    fn invariant_masks_are_unique_and_single_bit() {
        let invariants = [
            IntegrityInvariant::ProgramId,
            IntegrityInvariant::ProtocolPda,
            IntegrityInvariant::ProtocolBump,
            IntegrityInvariant::ProtocolVersion,
            IntegrityInvariant::ProtocolAuthority,
            IntegrityInvariant::ProtocolConfigPda,
            IntegrityInvariant::ProtocolConfigBump,
            IntegrityInvariant::ProtocolConfigVersion,
            IntegrityInvariant::ProtocolConfigLink,
            IntegrityInvariant::TreasuryPda,
            IntegrityInvariant::TreasuryBump,
            IntegrityInvariant::TreasuryVersion,
            IntegrityInvariant::TreasuryLink,
            IntegrityInvariant::TreasuryVaultPda,
            IntegrityInvariant::TreasuryVaultLink,
            IntegrityInvariant::TreasuryVaultMint,
            IntegrityInvariant::TreasuryVaultOwner,
            IntegrityInvariant::FounderPda,
            IntegrityInvariant::FounderBump,
            IntegrityInvariant::FounderVersion,
            IntegrityInvariant::FounderLink,
            IntegrityInvariant::CompanyPda,
            IntegrityInvariant::CompanyBump,
            IntegrityInvariant::CompanyVersion,
            IntegrityInvariant::CompanyLink,
            IntegrityInvariant::ExecutionConfigPda,
            IntegrityInvariant::ExecutionConfigBump,
            IntegrityInvariant::ExecutionConfigVersion,
            IntegrityInvariant::ExecutionConfigLink,
            IntegrityInvariant::SettlementMintLink,
            IntegrityInvariant::DestinationMints,
            IntegrityInvariant::FounderDestinationOwner,
            IntegrityInvariant::CompanyDestinationOwner,
            IntegrityInvariant::DestinationIsolation,
        ];

        let mut combined = 0_u64;

        for invariant in invariants {
            let mask = invariant.mask();

            assert_ne!(mask, 0);
            assert_eq!(mask.count_ones(), 1);
            assert_eq!(combined & mask, 0);

            combined |= mask;
        }
    }
}
