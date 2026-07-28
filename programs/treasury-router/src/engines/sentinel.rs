use anchor_lang::prelude::*;

use crate::{
    engines::{
        beaver_score,
        dam::{self, DamLevel},
        waterfall::{self, WaterfallStage},
    },
    state::{ProtocolConfig, TreasuryState},
};

/// Stable identifiers for the invariants checked by Sentinel v1.
///
/// Existing numeric values must never be reordered because external monitoring
/// may interpret the failure mask using these values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SentinelInvariant {
    AllocationTotal = 0,
    TreasuryAccounting = 1,
    WaterfallStageRange = 2,
    DamReleaseRate = 3,
    DamWaterfallLink = 4,
}

impl SentinelInvariant {
    pub const fn mask(self) -> u64 {
        1_u64 << self as u8
    }
}

/// Read-only protocol integrity report.
///
/// Sentinel does not:
/// - move funds;
/// - authorize releases;
/// - modify accounts;
/// - pause or unpause the protocol;
/// - modify Beavernomics;
/// - replace the Waterfall, Dam, or Authorization Firewall.
///
/// A zero failure mask means that every Sentinel v1 invariant passed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SentinelReport {
    pub healthy: bool,
    pub failure_mask: u64,

    pub allocation_total_bps: u16,
    pub allocation_total_valid: bool,

    pub accounting_valid: bool,

    pub waterfall_stage: WaterfallStage,
    pub waterfall_stage_valid: bool,

    pub dam_level: DamLevel,
    pub release_bps: u16,
    pub dam_release_rate_valid: bool,
    pub dam_waterfall_link_valid: bool,

    pub reserve_ratio_bps: u16,
    pub total_pending: u64,
}

/// Evaluates current protocol integrity without modifying state.
///
/// The Waterfall and Dam are calculated from authoritative TreasuryState.
/// Sentinel deliberately does not maintain or introduce a second Dam state.
pub fn evaluate(
    protocol_config: &ProtocolConfig,
    treasury: &TreasuryState,
) -> Result<SentinelReport> {
    let mut failure_mask = 0_u64;

    /*
     * Invariant 1:
     * Locked Beavernomics must allocate exactly 100% of the eligible fee.
     */
    let allocation_total_bps = protocol_config.total_allocation_bps();
    let allocation_total_valid = allocation_total_bps == 10_000;

    if !allocation_total_valid {
        failure_mask |= SentinelInvariant::AllocationTotal.mask();
    }

    /*
     * Invariant 2:
     * Lifetime allocations must equal pending plus released allocations.
     */
    let accounting_valid = treasury.execution_accounting_is_valid();

    if !accounting_valid {
        failure_mask |= SentinelInvariant::TreasuryAccounting.mask();
    }

    /*
     * Derive the current Waterfall condition from authoritative treasury data.
     */
    let waterfall_evaluation = waterfall::evaluate(treasury)?;
    let waterfall_stage = waterfall_evaluation.stage;

    /*
     * Invariant 3:
     * The derived stage must be one of the five defined Waterfall stages.
     *
     * This explicit match is intentionally exhaustive. Adding a new stage
     * requires a conscious Sentinel review rather than silently accepting it.
     */
    let waterfall_stage_valid = matches!(
        waterfall_stage,
        WaterfallStage::Normal
            | WaterfallStage::Caution
            | WaterfallStage::Defensive
            | WaterfallStage::Survival
            | WaterfallStage::Emergency
    );

    if !waterfall_stage_valid {
        failure_mask |= SentinelInvariant::WaterfallStageRange.mask();
    }

    /*
     * Derive the Adaptive Dam from authoritative treasury health.
     */
    let pre_dam_health_score = beaver_score::pre_dam_health_score(
        treasury,
        waterfall_evaluation.reserve_ratio_bps,
        waterfall_stage,
    )?;

    let dam_evaluation = dam::evaluate_adaptive(waterfall_stage, pre_dam_health_score);

    let dam_level = dam_evaluation.level;
    let release_bps = dam_evaluation.release_bps;

    /*
     * Invariant 4:
     * A Dam may never permit more than 100% release, and the reported rate
     * must equal the canonical rate defined by its derived Dam level.
     */
    let dam_release_rate_valid = release_bps <= 10_000 && release_bps == dam_level.release_bps();

    if !dam_release_rate_valid {
        failure_mask |= SentinelInvariant::DamReleaseRate.mask();
    }

    /*
     * Invariant 5:
     * The Dam evaluation must retain the exact Waterfall stage supplied to it.
     * This detects an internally inconsistent Dam evaluation result.
     */
    let dam_waterfall_link_valid = dam_evaluation.waterfall_stage == waterfall_stage;

    if !dam_waterfall_link_valid {
        failure_mask |= SentinelInvariant::DamWaterfallLink.mask();
    }

    Ok(SentinelReport {
        healthy: failure_mask == 0,
        failure_mask,

        allocation_total_bps,
        allocation_total_valid,

        accounting_valid,

        waterfall_stage,
        waterfall_stage_valid,

        dam_level,
        release_bps,
        dam_release_rate_valid,
        dam_waterfall_link_valid,

        reserve_ratio_bps: waterfall_evaluation.reserve_ratio_bps,
        total_pending: waterfall_evaluation.total_pending,
    })
}

/// Returns true when a specific Sentinel invariant failed.
pub const fn invariant_failed(failure_mask: u64, invariant: SentinelInvariant) -> bool {
    failure_mask & invariant.mask() != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invariant_masks_are_unique_and_single_bit() {
        let invariants = [
            SentinelInvariant::AllocationTotal,
            SentinelInvariant::TreasuryAccounting,
            SentinelInvariant::WaterfallStageRange,
            SentinelInvariant::DamReleaseRate,
            SentinelInvariant::DamWaterfallLink,
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

    #[test]
    fn invariant_failure_detection_is_exact() {
        let mask = SentinelInvariant::TreasuryAccounting.mask()
            | SentinelInvariant::DamWaterfallLink.mask();

        assert!(!invariant_failed(mask, SentinelInvariant::AllocationTotal));
        assert!(invariant_failed(
            mask,
            SentinelInvariant::TreasuryAccounting
        ));
        assert!(!invariant_failed(
            mask,
            SentinelInvariant::WaterfallStageRange
        ));
        assert!(!invariant_failed(mask, SentinelInvariant::DamReleaseRate));
        assert!(invariant_failed(mask, SentinelInvariant::DamWaterfallLink));
    }

    #[test]
    fn canonical_dam_rates_are_never_above_one_hundred_percent() {
        let levels = [
            DamLevel::Filling,
            DamLevel::Restricted,
            DamLevel::Controlled,
            DamLevel::Normal,
            DamLevel::Overflow,
        ];

        for level in levels {
            assert!(level.release_bps() <= 10_000);
        }
    }

    #[test]
    fn every_waterfall_stage_produces_a_consistent_dam_evaluation() {
        let stages = [
            WaterfallStage::Normal,
            WaterfallStage::Caution,
            WaterfallStage::Defensive,
            WaterfallStage::Survival,
            WaterfallStage::Emergency,
        ];

        for stage in stages {
            let evaluation = dam::evaluate(stage);

            assert_eq!(evaluation.waterfall_stage, stage);
            assert_eq!(evaluation.release_bps, evaluation.level.release_bps());
            assert!(evaluation.release_bps <= 10_000);
        }
    }

    #[test]
    fn zero_failure_mask_contains_no_failures() {
        assert!(!invariant_failed(0, SentinelInvariant::AllocationTotal));
        assert!(!invariant_failed(0, SentinelInvariant::TreasuryAccounting));
        assert!(!invariant_failed(0, SentinelInvariant::WaterfallStageRange));
        assert!(!invariant_failed(0, SentinelInvariant::DamReleaseRate));
        assert!(!invariant_failed(0, SentinelInvariant::DamWaterfallLink));
    }
}

/* ========================================================================
 * SENTINEL V2 — IMMUTABLE CONFIGURATION AND ACCOUNT LINKAGE
 * ========================================================================
 *
 * This module does not access AccountInfo, invoke programs, transfer tokens,
 * mutate state, or authorize execution.
 *
 * It receives plain references and verifies that their values remain
 * consistent with the locked RBVR protocol.
 */

/// Exact locked RBVR Beavernomics allocation.
pub const LOCKED_RESERVE_BPS: u16 = 3_000;
pub const LOCKED_BUYBACK_BURN_BPS: u16 = 2_000;
pub const LOCKED_LIQUIDITY_BPS: u16 = 2_000;
pub const LOCKED_COMPANY_BPS: u16 = 2_000;
pub const LOCKED_FOUNDER_BPS: u16 = 1_000;

/// Stable Sentinel v2 failure-mask identifiers.
///
/// These values are separate from the Sentinel v1 mask so that neither
/// version's existing identifiers need to be reordered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SentinelLinkageInvariant {
    ProtocolConfigLink = 0,
    TreasuryProtocolLink = 1,
    LockedBeavernomics = 2,
    ConfigurationLocked = 3,
    SettlementMintPresent = 4,
    SettlementVaultPresent = 5,
    LifetimeAllocationConservation = 6,
    ReceivedAllocationRelationship = 7,
}

impl SentinelLinkageInvariant {
    pub const fn mask(self) -> u64 {
        1_u64 << self as u8
    }
}

/// Read-only Sentinel v2 report.
///
/// A zero `failure_mask` means every linkage and immutable-configuration
/// invariant passed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SentinelLinkageReport {
    pub healthy: bool,
    pub failure_mask: u64,

    pub protocol_config_link_valid: bool,
    pub treasury_protocol_link_valid: bool,

    pub locked_beavernomics_valid: bool,
    pub configuration_locked: bool,

    pub settlement_mint_present: bool,
    pub settlement_vault_present: bool,

    pub lifetime_allocation_sum: Option<u64>,
    pub lifetime_allocation_conserved: bool,

    pub received_allocation_relationship_valid: bool,
}

/// Evaluates immutable configuration and protocol-account linkage.
///
/// `expected_protocol` must be the canonical ProtocolState public key supplied
/// by the caller or monitoring layer.
///
/// Sentinel deliberately accepts only plain values and references. It does not
/// receive mutable accounts or signer authority.
pub fn evaluate_linkage(
    expected_protocol: Pubkey,
    protocol_config: &ProtocolConfig,
    treasury: &TreasuryState,
) -> SentinelLinkageReport {
    let mut failure_mask = 0_u64;

    /*
     * The configuration and treasury must both point to the same canonical
     * ProtocolState.
     */
    let protocol_config_link_valid = protocol_config.protocol == expected_protocol;

    if !protocol_config_link_valid {
        failure_mask |= SentinelLinkageInvariant::ProtocolConfigLink.mask();
    }

    let treasury_protocol_link_valid = treasury.protocol == expected_protocol;

    if !treasury_protocol_link_valid {
        failure_mask |= SentinelLinkageInvariant::TreasuryProtocolLink.mask();
    }

    /*
     * Verify the exact locked 30/20/20/20/10 allocation.
     *
     * Merely totalling 10,000 basis points is insufficient: a corrupted
     * configuration could still total 100% while changing the distribution.
     */
    let locked_beavernomics_valid = protocol_config.reserve_bps == LOCKED_RESERVE_BPS
        && protocol_config.buyback_burn_bps == LOCKED_BUYBACK_BURN_BPS
        && protocol_config.liquidity_bps == LOCKED_LIQUIDITY_BPS
        && protocol_config.company_bps == LOCKED_COMPANY_BPS
        && protocol_config.founder_bps == LOCKED_FOUNDER_BPS;

    if !locked_beavernomics_valid {
        failure_mask |= SentinelLinkageInvariant::LockedBeavernomics.mask();
    }

    /*
     * Locked configuration must not permit updates.
     */
    let configuration_locked = !protocol_config.updates_enabled;

    if !configuration_locked {
        failure_mask |= SentinelLinkageInvariant::ConfigurationLocked.mask();
    }

    /*
     * Settlement references must have been initialized.
     *
     * Ownership and SPL-token mint checks require actual token-account data
     * and will be added later through a dedicated account-view verifier.
     */
    let settlement_mint_present = treasury.settlement_mint != Pubkey::default();

    if !settlement_mint_present {
        failure_mask |= SentinelLinkageInvariant::SettlementMintPresent.mask();
    }

    let settlement_vault_present = treasury.settlement_vault != Pubkey::default();

    if !settlement_vault_present {
        failure_mask |= SentinelLinkageInvariant::SettlementVaultPresent.mask();
    }

    /*
     * Every lifetime bucket must add up exactly to the protocol's recorded
     * total allocated fees.
     *
     * checked_add makes overflow a failed invariant rather than a panic.
     */
    let lifetime_allocation_sum = treasury
        .lifetime_reserve
        .checked_add(treasury.lifetime_buyback_burn)
        .and_then(|value| value.checked_add(treasury.lifetime_liquidity))
        .and_then(|value| value.checked_add(treasury.lifetime_company))
        .and_then(|value| value.checked_add(treasury.lifetime_founder));

    let lifetime_allocation_conserved =
        lifetime_allocation_sum == Some(treasury.total_fees_allocated);

    if !lifetime_allocation_conserved {
        failure_mask |= SentinelLinkageInvariant::LifetimeAllocationConservation.mask();
    }

    /*
     * Allocated fees may never exceed received fees.
     *
     * In the current accounting design these values should ordinarily be
     * equal after successful processing, but <= is the fundamental safety
     * invariant and remains valid between accounting operations.
     */
    let received_allocation_relationship_valid =
        treasury.total_fees_allocated <= treasury.total_fees_received;

    if !received_allocation_relationship_valid {
        failure_mask |= SentinelLinkageInvariant::ReceivedAllocationRelationship.mask();
    }

    SentinelLinkageReport {
        healthy: failure_mask == 0,
        failure_mask,

        protocol_config_link_valid,
        treasury_protocol_link_valid,

        locked_beavernomics_valid,
        configuration_locked,

        settlement_mint_present,
        settlement_vault_present,

        lifetime_allocation_sum,
        lifetime_allocation_conserved,

        received_allocation_relationship_valid,
    }
}

/// Returns true when a Sentinel v2 invariant failed.
pub const fn linkage_invariant_failed(
    failure_mask: u64,
    invariant: SentinelLinkageInvariant,
) -> bool {
    failure_mask & invariant.mask() != 0
}

#[cfg(test)]
mod sentinel_v2_tests {
    use super::*;

    fn valid_protocol_config(protocol: Pubkey) -> ProtocolConfig {
        ProtocolConfig {
            version: 1,
            protocol,
            reserve_bps: LOCKED_RESERVE_BPS,
            buyback_burn_bps: LOCKED_BUYBACK_BURN_BPS,
            liquidity_bps: LOCKED_LIQUIDITY_BPS,
            company_bps: LOCKED_COMPANY_BPS,
            founder_bps: LOCKED_FOUNDER_BPS,
            updates_enabled: false,
            bump: 255,
            updated_at: 0,
            reserved: [0; 64],
        }
    }

    fn valid_treasury(protocol: Pubkey) -> TreasuryState {
        TreasuryState {
            version: 1,
            protocol,
            settlement_mint: Pubkey::new_unique(),
            settlement_vault: Pubkey::new_unique(),

            total_fees_received: 1_000,
            total_fees_allocated: 1_000,

            pending_reserve: 300,
            pending_buyback_burn: 200,
            pending_liquidity: 200,
            pending_company: 200,
            pending_founder: 100,

            released_reserve: 0,
            released_buyback_burn: 0,
            released_liquidity: 0,
            released_company: 0,
            released_founder: 0,

            lifetime_reserve: 300,
            lifetime_buyback_burn: 200,
            lifetime_liquidity: 200,
            lifetime_company: 200,
            lifetime_founder: 100,

            last_processed_at: 0,
            processing_epoch: 1,
            waterfall_stage: WaterfallStage::Normal.as_u8(),
            buybacks_paused: false,
            bump: 255,
            reserved: [0; 24],
        }
    }

    #[test]
    fn valid_locked_protocol_passes_every_linkage_check() {
        let protocol = Pubkey::new_unique();
        let config = valid_protocol_config(protocol);
        let treasury = valid_treasury(protocol);

        let report = evaluate_linkage(protocol, &config, &treasury);

        assert!(report.healthy);
        assert_eq!(report.failure_mask, 0);
        assert!(report.protocol_config_link_valid);
        assert!(report.treasury_protocol_link_valid);
        assert!(report.locked_beavernomics_valid);
        assert!(report.configuration_locked);
        assert!(report.settlement_mint_present);
        assert!(report.settlement_vault_present);
        assert_eq!(report.lifetime_allocation_sum, Some(1_000));
        assert!(report.lifetime_allocation_conserved);
        assert!(report.received_allocation_relationship_valid);
    }

    #[test]
    fn allocation_total_of_one_hundred_percent_is_not_enough() {
        let protocol = Pubkey::new_unique();
        let mut config = valid_protocol_config(protocol);
        let treasury = valid_treasury(protocol);

        // Still totals 10,000 bps, but silently changes Beavernomics.
        config.reserve_bps = 2_900;
        config.founder_bps = 1_100;

        let report = evaluate_linkage(protocol, &config, &treasury);

        assert!(!report.healthy);
        assert!(!report.locked_beavernomics_valid);
        assert!(linkage_invariant_failed(
            report.failure_mask,
            SentinelLinkageInvariant::LockedBeavernomics
        ));
    }

    #[test]
    fn configuration_updates_enabled_fails_closed() {
        let protocol = Pubkey::new_unique();
        let mut config = valid_protocol_config(protocol);
        let treasury = valid_treasury(protocol);

        config.updates_enabled = true;

        let report = evaluate_linkage(protocol, &config, &treasury);

        assert!(!report.configuration_locked);
        assert!(linkage_invariant_failed(
            report.failure_mask,
            SentinelLinkageInvariant::ConfigurationLocked
        ));
    }

    #[test]
    fn wrong_protocol_links_are_detected_independently() {
        let expected_protocol = Pubkey::new_unique();
        let wrong_protocol = Pubkey::new_unique();

        let config = valid_protocol_config(wrong_protocol);
        let treasury = valid_treasury(wrong_protocol);

        let report = evaluate_linkage(expected_protocol, &config, &treasury);

        assert!(!report.protocol_config_link_valid);
        assert!(!report.treasury_protocol_link_valid);
    }

    #[test]
    fn missing_settlement_references_fail() {
        let protocol = Pubkey::new_unique();
        let config = valid_protocol_config(protocol);
        let mut treasury = valid_treasury(protocol);

        treasury.settlement_mint = Pubkey::default();
        treasury.settlement_vault = Pubkey::default();

        let report = evaluate_linkage(protocol, &config, &treasury);

        assert!(!report.settlement_mint_present);
        assert!(!report.settlement_vault_present);
    }

    #[test]
    fn corrupted_lifetime_total_is_detected() {
        let protocol = Pubkey::new_unique();
        let config = valid_protocol_config(protocol);
        let mut treasury = valid_treasury(protocol);

        treasury.lifetime_founder = 99;

        let report = evaluate_linkage(protocol, &config, &treasury);

        assert_eq!(report.lifetime_allocation_sum, Some(999));
        assert!(!report.lifetime_allocation_conserved);
    }

    #[test]
    fn lifetime_sum_overflow_fails_closed() {
        let protocol = Pubkey::new_unique();
        let config = valid_protocol_config(protocol);
        let mut treasury = valid_treasury(protocol);

        treasury.lifetime_reserve = u64::MAX;
        treasury.lifetime_buyback_burn = 1;

        let report = evaluate_linkage(protocol, &config, &treasury);

        assert_eq!(report.lifetime_allocation_sum, None);
        assert!(!report.lifetime_allocation_conserved);
    }

    #[test]
    fn allocated_fees_cannot_exceed_received_fees() {
        let protocol = Pubkey::new_unique();
        let config = valid_protocol_config(protocol);
        let mut treasury = valid_treasury(protocol);

        treasury.total_fees_received = 999;

        let report = evaluate_linkage(protocol, &config, &treasury);

        assert!(!report.received_allocation_relationship_valid);
    }

    #[test]
    fn linkage_masks_are_unique() {
        let invariants = [
            SentinelLinkageInvariant::ProtocolConfigLink,
            SentinelLinkageInvariant::TreasuryProtocolLink,
            SentinelLinkageInvariant::LockedBeavernomics,
            SentinelLinkageInvariant::ConfigurationLocked,
            SentinelLinkageInvariant::SettlementMintPresent,
            SentinelLinkageInvariant::SettlementVaultPresent,
            SentinelLinkageInvariant::LifetimeAllocationConservation,
            SentinelLinkageInvariant::ReceivedAllocationRelationship,
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
