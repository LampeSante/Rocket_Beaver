# Rocket Beaver Treasury Router — Deep Security Audit

- **Date:** 2026-07-29T18:08:31-04:00
- **Branch:** phase2-autonomy
- **Commit:** 988930c6d6b17ebe791e2bf1ff3c793420dd185d
- **Rust:** rustc 1.97.1 (8bab26f4f 2026-07-14)
- **Nightly:** rustc 1.99.0-nightly (09ee43b2d 2026-07-27)
- **Anchor:** anchor-cli 1.1.2
- **Solana:** solana-cli 3.1.10 (src:7bc9c805; feat:1620780344, client:Agave)

This is an automated source and test review. It is not a substitute for an
independent professional audit.

## Repository condition

- ⚠️ **REVIEW:** The working tree contains uncommitted changes. Audit results apply to the exact local working tree, not merely the current commit.

```text
 M fuzz/Cargo.toml
 M programs/treasury-router/src/engines/mod.rs
 M programs/treasury-router/src/instructions/buyback.rs
 M programs/treasury-router/src/instructions/company.rs
 M programs/treasury-router/src/instructions/founder.rs
 M programs/treasury-router/src/instructions/liquidity.rs
 M programs/treasury-router/src/instructions/reserve.rs
?? .rbvr-security-audit/
?? RBVR-SECURITY-AUDIT-LATEST.md
?? fuzz/fuzz_targets/process_release.rs
?? programs/treasury-router/src/engines/release.rs
?? rbvr-deep-security-audit.sh
?? rbvr-release-fuzz-source.txt
```
- ✅ **PASS:** No whitespace or conflict-marker errors detected by git diff --check.

## Formatting and compiler checks

- ✅ **PASS:** Rust formatting check passed.
- ❌ **FAIL:** Strict Clippy failed.

```text
    Checking treasury-router v0.1.0 (/home/alec_elliott/rocket-beaver-contract/programs/treasury-router)
error: deref which would be done by auto-deref
   --> programs/treasury-router/src/instructions/process_fees.rs:181:9
    |
181 |         &*ctx.accounts.treasury,
    |         ^^^^^^^^^^^^^^^^^^^^^^^ help: try: `&ctx.accounts.treasury`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.97.0/index.html#explicit_auto_deref
    = note: `-D clippy::explicit-auto-deref` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::explicit_auto_deref)]`

error: deref which would be done by auto-deref
   --> programs/treasury-router/src/instructions/process_fees.rs:190:59
    |
190 |         sentinel::evaluate(&ctx.accounts.protocol_config, &*ctx.accounts.treasury)?;
    |                                                           ^^^^^^^^^^^^^^^^^^^^^^^ help: try: `&ctx.accounts.treasury`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.97.0/index.html#explicit_auto_deref

error: could not compile `treasury-router` (lib) due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `treasury-router` (lib test) due to 2 previous errors
```

## Production Rust tests

- ✅ **PASS:** All Rust production tests passed.

```text
test instructions::process_fees::tests::simultaneous_company_and_founder_overflow_is_conserved ... ok
test state::treasury::tests::buyback_accounting_fails_closed_on_overflow ... ok
test state::treasury::tests::buyback_accounting_is_invalid_when_values_do_not_balance ... ok
test state::treasury::tests::buyback_accounting_is_valid_when_lifetime_equals_pending_plus_released ... ok
test state::treasury::tests::company_accounting_fails_closed_on_overflow ... ok
test state::treasury::tests::company_accounting_is_invalid_when_values_do_not_balance ... ok
test state::treasury::tests::company_accounting_is_valid_when_lifetime_equals_pending_plus_released ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_liquidity_is_corrupted ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_multiple_buckets_are_corrupted ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_buyback_is_corrupted ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_reserve_is_corrupted ... ok
test state::treasury::tests::execution_accounting_is_valid_when_every_bucket_balances ... ok
test state::treasury::tests::founder_accounting_fails_closed_on_overflow ... ok
test state::treasury::tests::founder_accounting_is_invalid_when_values_do_not_balance ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_company_is_corrupted ... ok
test state::treasury::tests::liquidity_accounting_is_invalid_when_values_do_not_balance ... ok
test state::treasury::tests::liquidity_accounting_fails_closed_on_overflow ... ok
test state::treasury::tests::founder_accounting_is_valid_when_lifetime_equals_pending_plus_released ... ok
test state::treasury::tests::liquidity_accounting_is_valid_when_lifetime_equals_pending_plus_released ... ok
test state::treasury::tests::reserve_accounting_is_valid_when_lifetime_equals_pending_plus_released ... ok
test test_id ... ok
test state::treasury::tests::execution_accounting_is_invalid_when_founder_is_corrupted ... ok
test state::treasury::tests::zero_balances_are_valid_accounting ... ok
test state::treasury::tests::overflow_with_max_lifetime_fails_closed_for_every_bucket ... ok
test state::treasury::tests::reserve_accounting_fails_closed_on_overflow ... ok
test state::treasury::tests::reserve_accounting_is_invalid_when_values_do_not_balance ... ok

test result: ok. 129 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests treasury_router

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## Release-engine fuzz build

- ✅ **PASS:** The process_release fuzz target builds under nightly with sanitizer instrumentation.

## Bounded release-engine fuzz campaign

- ✅ **PASS:** The bounded 1,000,000-run process_release campaign completed without a crash.

## Previously saved slow-unit reproduction

- Artifact: `fuzz/artifacts/process_release/slow-unit-f752e1b37fc51dfbb46c97c41343412837e9d1de`
- ✅ **PASS:** The saved slow unit reproduced and terminated within the audit timeout.

## Unchecked and saturating arithmetic inventory

```text
programs/treasury-router/src/engines/sentinel.rs:28:        1_u64 << self as u8
programs/treasury-router/src/engines/sentinel.rs:311:        1_u64 << self as u8
programs/treasury-router/src/engines/integrity_firewall.rs:67:        1_u64 << self as u8
programs/treasury-router/src/engines/integrity_firewall.rs:659:        fixture.treasury.bump = fixture.treasury.bump.wrapping_add(1);
programs/treasury-router/src/engines/dam.rs:28:        self as u8
programs/treasury-router/src/engines/waterfall.rs:31:        self as u8
programs/treasury-router/src/engines/waterfall.rs:183:        self.reserve_bps as u32
programs/treasury-router/src/engines/waterfall.rs:184:            + self.buyback_burn_bps as u32
programs/treasury-router/src/engines/waterfall.rs:185:            + self.liquidity_bps as u32
programs/treasury-router/src/engines/waterfall.rs:186:            + self.company_bps as u32
programs/treasury-router/src/engines/waterfall.rs:187:            + self.founder_bps as u32
programs/treasury-router/src/engines/founder.rs:62:        .saturating_sub(founder.earned_current_period);
programs/treasury-router/src/engines/company.rs:62:        .saturating_sub(company.spent_current_period);
programs/treasury-router/src/state/protocol_config.rs:43:            .saturating_add(self.buyback_burn_bps)
programs/treasury-router/src/state/protocol_config.rs:44:            .saturating_add(self.liquidity_bps)
programs/treasury-router/src/state/protocol_config.rs:45:            .saturating_add(self.company_bps)
programs/treasury-router/src/state/protocol_config.rs:46:            .saturating_add(self.founder_bps)
```
- ⚠️ **REVIEW:** Saturating or wrapping arithmetic remains in production source. Each match must be justified as fail-closed or replaced with checked arithmetic.

## Explicit panic and assertion inventory

```text
programs/treasury-router/src/engines/sentinel.rs:205:            assert_eq!(mask.count_ones(), 1);
programs/treasury-router/src/engines/sentinel.rs:206:            assert_eq!(combined & mask, 0);
programs/treasury-router/src/engines/sentinel.rs:217:        assert!(!invariant_failed(mask, SentinelInvariant::AllocationTotal));
programs/treasury-router/src/engines/sentinel.rs:218:        assert!(invariant_failed(
programs/treasury-router/src/engines/sentinel.rs:222:        assert!(!invariant_failed(
programs/treasury-router/src/engines/sentinel.rs:226:        assert!(!invariant_failed(mask, SentinelInvariant::DamReleaseRate));
programs/treasury-router/src/engines/sentinel.rs:227:        assert!(invariant_failed(mask, SentinelInvariant::DamWaterfallLink));
programs/treasury-router/src/engines/sentinel.rs:241:            assert!(level.release_bps() <= 10_000);
programs/treasury-router/src/engines/sentinel.rs:258:            assert_eq!(evaluation.waterfall_stage, stage);
programs/treasury-router/src/engines/sentinel.rs:259:            assert_eq!(evaluation.release_bps, evaluation.level.release_bps());
programs/treasury-router/src/engines/sentinel.rs:260:            assert!(evaluation.release_bps <= 10_000);
programs/treasury-router/src/engines/sentinel.rs:266:        assert!(!invariant_failed(0, SentinelInvariant::AllocationTotal));
programs/treasury-router/src/engines/sentinel.rs:267:        assert!(!invariant_failed(0, SentinelInvariant::TreasuryAccounting));
programs/treasury-router/src/engines/sentinel.rs:268:        assert!(!invariant_failed(0, SentinelInvariant::WaterfallStageRange));
programs/treasury-router/src/engines/sentinel.rs:269:        assert!(!invariant_failed(0, SentinelInvariant::DamReleaseRate));
programs/treasury-router/src/engines/sentinel.rs:270:        assert!(!invariant_failed(0, SentinelInvariant::DamWaterfallLink));
programs/treasury-router/src/engines/sentinel.rs:539:        assert!(report.healthy);
programs/treasury-router/src/engines/sentinel.rs:540:        assert_eq!(report.failure_mask, 0);
programs/treasury-router/src/engines/sentinel.rs:541:        assert!(report.protocol_config_link_valid);
programs/treasury-router/src/engines/sentinel.rs:542:        assert!(report.treasury_protocol_link_valid);
programs/treasury-router/src/engines/sentinel.rs:543:        assert!(report.locked_beavernomics_valid);
programs/treasury-router/src/engines/sentinel.rs:544:        assert!(report.configuration_locked);
programs/treasury-router/src/engines/sentinel.rs:545:        assert!(report.settlement_mint_present);
programs/treasury-router/src/engines/sentinel.rs:546:        assert!(report.settlement_vault_present);
programs/treasury-router/src/engines/sentinel.rs:547:        assert_eq!(report.lifetime_allocation_sum, Some(1_000));
programs/treasury-router/src/engines/sentinel.rs:548:        assert!(report.lifetime_allocation_conserved);
programs/treasury-router/src/engines/sentinel.rs:549:        assert!(report.received_allocation_relationship_valid);
programs/treasury-router/src/engines/sentinel.rs:564:        assert!(!report.healthy);
programs/treasury-router/src/engines/sentinel.rs:565:        assert!(!report.locked_beavernomics_valid);
programs/treasury-router/src/engines/sentinel.rs:566:        assert!(linkage_invariant_failed(
programs/treasury-router/src/engines/sentinel.rs:582:        assert!(!report.configuration_locked);
programs/treasury-router/src/engines/sentinel.rs:583:        assert!(linkage_invariant_failed(
programs/treasury-router/src/engines/sentinel.rs:599:        assert!(!report.protocol_config_link_valid);
programs/treasury-router/src/engines/sentinel.rs:600:        assert!(!report.treasury_protocol_link_valid);
programs/treasury-router/src/engines/sentinel.rs:614:        assert!(!report.settlement_mint_present);
programs/treasury-router/src/engines/sentinel.rs:615:        assert!(!report.settlement_vault_present);
programs/treasury-router/src/engines/sentinel.rs:628:        assert_eq!(report.lifetime_allocation_sum, Some(999));
programs/treasury-router/src/engines/sentinel.rs:629:        assert!(!report.lifetime_allocation_conserved);
programs/treasury-router/src/engines/sentinel.rs:643:        assert_eq!(report.lifetime_allocation_sum, None);
programs/treasury-router/src/engines/sentinel.rs:644:        assert!(!report.lifetime_allocation_conserved);
programs/treasury-router/src/engines/sentinel.rs:657:        assert!(!report.received_allocation_relationship_valid);
programs/treasury-router/src/engines/sentinel.rs:679:            assert_eq!(mask.count_ones(), 1);
programs/treasury-router/src/engines/sentinel.rs:680:            assert_eq!(combined & mask, 0);
programs/treasury-router/src/engines/integrity_firewall.rs:628:        assert!(report.healthy);
programs/treasury-router/src/engines/integrity_firewall.rs:629:        assert_eq!(report.failure_mask, 0);
programs/treasury-router/src/engines/integrity_firewall.rs:649:        assert!(!report.healthy);
programs/treasury-router/src/engines/integrity_firewall.rs:650:        assert!(integrity_invariant_failed(
programs/treasury-router/src/engines/integrity_firewall.rs:663:        assert!(integrity_invariant_failed(
programs/treasury-router/src/engines/integrity_firewall.rs:676:        assert!(integrity_invariant_failed(
programs/treasury-router/src/engines/integrity_firewall.rs:689:        assert!(integrity_invariant_failed(
programs/treasury-router/src/engines/integrity_firewall.rs:702:        assert!(integrity_invariant_failed(
programs/treasury-router/src/engines/integrity_firewall.rs:716:        assert!(integrity_invariant_failed(
programs/treasury-router/src/engines/integrity_firewall.rs:730:        assert!(integrity_invariant_failed(
programs/treasury-router/src/engines/integrity_firewall.rs:781:            assert_eq!(mask.count_ones(), 1);
programs/treasury-router/src/engines/integrity_firewall.rs:782:            assert_eq!(combined & mask, 0);
programs/treasury-router/src/engines/dam.rs:131:        assert_eq!(result.level, DamLevel::Overflow);
programs/treasury-router/src/engines/dam.rs:132:        assert_eq!(result.release_bps, 10_000);
programs/treasury-router/src/engines/dam.rs:139:        assert_eq!(result.level, DamLevel::Normal);
programs/treasury-router/src/engines/dam.rs:140:        assert_eq!(result.release_bps, 7_500);
programs/treasury-router/src/engines/dam.rs:147:        assert_eq!(result.level, DamLevel::Controlled);
programs/treasury-router/src/engines/dam.rs:148:        assert_eq!(result.release_bps, 5_000);
programs/treasury-router/src/engines/dam.rs:155:        assert_eq!(result.level, DamLevel::Restricted);
programs/treasury-router/src/engines/dam.rs:156:        assert_eq!(result.release_bps, 2_500);
programs/treasury-router/src/engines/dam.rs:163:        assert_eq!(result.level, DamLevel::Filling);
programs/treasury-router/src/engines/dam.rs:164:        assert_eq!(result.release_bps, 0);
programs/treasury-router/src/engines/dam.rs:176:        assert_eq!(result.level, DamLevel::Overflow);
programs/treasury-router/src/engines/dam.rs:177:        assert_eq!(result.release_bps, 10_000);
programs/treasury-router/src/engines/dam.rs:184:        assert_eq!(result.level, DamLevel::Controlled);
programs/treasury-router/src/engines/dam.rs:185:        assert_eq!(result.release_bps, 5_000);
programs/treasury-router/src/engines/dam.rs:192:        assert_eq!(result.level, DamLevel::Restricted);
programs/treasury-router/src/engines/dam.rs:193:        assert_eq!(result.release_bps, 2_500);
programs/treasury-router/src/engines/dam.rs:200:        assert_eq!(result.level, DamLevel::Filling);
programs/treasury-router/src/engines/dam.rs:201:        assert_eq!(result.release_bps, 0);
programs/treasury-router/src/engines/dam.rs:208:        assert_eq!(result.level, DamLevel::Filling);
programs/treasury-router/src/engines/dam.rs:209:        assert_eq!(result.release_bps, 0);
programs/treasury-router/src/engines/release.rs:193:        let transition = process_release(&mut treasury, bucket, 25).unwrap();
programs/treasury-router/src/engines/release.rs:195:        assert_eq!(transition.bucket, bucket);
programs/treasury-router/src/engines/release.rs:196:        assert_eq!(transition.amount, 25);
programs/treasury-router/src/engines/release.rs:197:        assert_eq!(transition.remaining_pending_balance, expected_pending);
programs/treasury-router/src/engines/release.rs:198:        assert_eq!(transition.total_released_balance, expected_released);
programs/treasury-router/src/engines/release.rs:199:        assert!(treasury.execution_accounting_is_valid());
programs/treasury-router/src/engines/release.rs:202:            assert_eq!(
programs/treasury-router/src/engines/release.rs:209:            assert_eq!(
programs/treasury-router/src/engines/release.rs:219:            assert_eq!(
programs/treasury-router/src/engines/release.rs:226:            assert_eq!(
programs/treasury-router/src/engines/release.rs:233:            assert_eq!(
programs/treasury-router/src/engines/release.rs:270:        assert!(process_release(&mut treasury, ReleaseBucket::Reserve, 0).is_err());
programs/treasury-router/src/engines/release.rs:271:        assert_eq!(
programs/treasury-router/src/engines/release.rs:282:        assert!(process_release(&mut treasury, ReleaseBucket::Founder, 201).is_err());
programs/treasury-router/src/engines/release.rs:284:        assert_eq!(
programs/treasury-router/src/engines/release.rs:297:        assert!(process_release(&mut treasury, ReleaseBucket::Company, 25).is_err());
programs/treasury-router/src/engines/release.rs:299:        assert_eq!(
programs/treasury-router/src/engines/release.rs:312:        assert!(process_release(&mut treasury, ReleaseBucket::Reserve, 25).is_err());
programs/treasury-router/src/engines/release.rs:314:        assert_eq!(
programs/treasury-router/src/engines/release.rs:332:        assert!(process_release(&mut treasury, ReleaseBucket::Reserve, 1).is_err());
programs/treasury-router/src/engines/release.rs:334:        assert_eq!(
programs/treasury-router/src/engines/release.rs:346:            process_release(&mut treasury, ReleaseBucket::BuybackBurn, amount).unwrap();
programs/treasury-router/src/engines/release.rs:348:        assert_eq!(transition.remaining_pending_balance, 0);
programs/treasury-router/src/engines/release.rs:349:        assert_eq!(
programs/treasury-router/src/engines/release.rs:353:        assert!(treasury.buyback_accounting_is_valid());
programs/treasury-router/src/engines/release.rs:354:        assert!(treasury.execution_accounting_is_valid());
programs/treasury-router/src/engines/waterfall.rs:138:        assert_eq!(classify(3_000), WaterfallStage::Normal);
programs/treasury-router/src/engines/waterfall.rs:139:        assert_eq!(classify(10_000), WaterfallStage::Normal);
programs/treasury-router/src/engines/waterfall.rs:144:        assert_eq!(classify(2_000), WaterfallStage::Caution);
programs/treasury-router/src/engines/waterfall.rs:145:        assert_eq!(classify(2_999), WaterfallStage::Caution);
programs/treasury-router/src/engines/waterfall.rs:150:        assert_eq!(classify(1_250), WaterfallStage::Defensive);
programs/treasury-router/src/engines/waterfall.rs:151:        assert_eq!(classify(1_999), WaterfallStage::Defensive);
programs/treasury-router/src/engines/waterfall.rs:156:        assert_eq!(classify(500), WaterfallStage::Survival);
programs/treasury-router/src/engines/waterfall.rs:157:        assert_eq!(classify(1_249), WaterfallStage::Survival);
programs/treasury-router/src/engines/waterfall.rs:162:        assert_eq!(classify(0), WaterfallStage::Emergency);
programs/treasury-router/src/engines/waterfall.rs:163:        assert_eq!(classify(499), WaterfallStage::Emergency);
programs/treasury-router/src/engines/waterfall.rs:251:            assert_eq!(allocation_for_stage(stage).total_bps(), 10_000);
programs/treasury-router/src/engines/waterfall.rs:259:        assert_eq!(allocation.reserve_bps, 3_000);
programs/treasury-router/src/engines/waterfall.rs:260:        assert_eq!(allocation.buyback_burn_bps, 2_000);
programs/treasury-router/src/engines/waterfall.rs:261:        assert_eq!(allocation.liquidity_bps, 2_000);
programs/treasury-router/src/engines/waterfall.rs:262:        assert_eq!(allocation.company_bps, 2_000);
programs/treasury-router/src/engines/waterfall.rs:263:        assert_eq!(allocation.founder_bps, 1_000);
programs/treasury-router/src/engines/waterfall.rs:278:            assert!(allocation.reserve_bps >= normal.reserve_bps);
programs/treasury-router/src/engines/waterfall.rs:279:            assert!(allocation.company_bps <= normal.company_bps);
programs/treasury-router/src/engines/waterfall.rs:280:            assert!(allocation.founder_bps <= normal.founder_bps);
programs/treasury-router/src/engines/waterfall.rs:289:            assert_eq!(allocation.buyback_burn_bps, 0);
programs/treasury-router/src/engines/waterfall.rs:290:            assert_eq!(allocation.company_bps, 0);
programs/treasury-router/src/engines/waterfall.rs:291:            assert_eq!(allocation.founder_bps, 0);
programs/treasury-router/src/engines/waterfall.rs:293:            assert_eq!(
programs/treasury-router/src/engines/execution_guard.rs:241:                panic!("Expected Anchor error {expected_error_name}, but authorization succeeded.")
programs/treasury-router/src/engines/execution_guard.rs:248:                assert_eq!(
programs/treasury-router/src/engines/execution_guard.rs:253:            other => panic!("Expected AnchorError {expected_error_name}, received {other:?}."),
programs/treasury-router/src/engines/execution_guard.rs:259:        assert_eq!(calculate_maximum_release(1_000, 10_000).unwrap(), 1_000);
programs/treasury-router/src/engines/execution_guard.rs:264:        assert_eq!(calculate_maximum_release(1_000, 7_500).unwrap(), 750);
programs/treasury-router/src/engines/execution_guard.rs:269:        assert_eq!(calculate_maximum_release(1_000, 5_000).unwrap(), 500);
programs/treasury-router/src/engines/execution_guard.rs:274:        assert_eq!(calculate_maximum_release(1_000, 2_500).unwrap(), 250);
programs/treasury-router/src/engines/execution_guard.rs:279:        assert_eq!(calculate_maximum_release(1_000, 0).unwrap(), 0);
programs/treasury-router/src/engines/execution_guard.rs:284:        assert_eq!(calculate_maximum_release(3, 2_500).unwrap(), 0);
programs/treasury-router/src/engines/execution_guard.rs:286:        assert_eq!(calculate_maximum_release(7, 5_000).unwrap(), 3);
programs/treasury-router/src/engines/execution_guard.rs:314:            authorize_release(&protocol, &treasury, ReleaseBucket::Reserve, 750).unwrap();
programs/treasury-router/src/engines/execution_guard.rs:316:        assert_eq!(authorization.bucket, ReleaseBucket::Reserve);
programs/treasury-router/src/engines/execution_guard.rs:317:        assert_eq!(authorization.requested_amount, 750);
programs/treasury-router/src/engines/execution_guard.rs:318:        assert_eq!(authorization.pending_balance, 1_000);
programs/treasury-router/src/engines/execution_guard.rs:319:        assert_eq!(authorization.maximum_release, 1_000);
programs/treasury-router/src/engines/execution_guard.rs:320:        assert_eq!(authorization.waterfall_stage, WaterfallStage::Normal);
programs/treasury-router/src/engines/execution_guard.rs:321:        assert_eq!(authorization.dam_level, DamLevel::Overflow);
programs/treasury-router/src/engines/execution_guard.rs:322:        assert_eq!(authorization.release_bps, 10_000);
programs/treasury-router/src/engines/execution_guard.rs:323:        assert_eq!(authorization.reserve_ratio_bps, 7_142);
programs/treasury-router/src/engines/execution_guard.rs:332:            authorize_release(&protocol, &treasury, ReleaseBucket::BuybackBurn, 100).unwrap();
programs/treasury-router/src/engines/execution_guard.rs:334:        assert_eq!(authorization.bucket, ReleaseBucket::BuybackBurn);
programs/treasury-router/src/engines/execution_guard.rs:335:        assert_eq!(authorization.requested_amount, 100);
programs/treasury-router/src/engines/execution_guard.rs:336:        assert_eq!(authorization.pending_balance, 100);
programs/treasury-router/src/engines/execution_guard.rs:337:        assert_eq!(authorization.maximum_release, 100);
programs/treasury-router/src/engines/execution_guard.rs:385:            authorize_release(&protocol, &treasury, ReleaseBucket::Liquidity, 50).unwrap();
programs/treasury-router/src/engines/execution_guard.rs:387:        assert_eq!(authorization.bucket, ReleaseBucket::Liquidity);
programs/treasury-router/src/engines/execution_guard.rs:388:        assert_eq!(authorization.requested_amount, 50);
programs/treasury-router/src/engines/execution_guard.rs:482:            authorize_release(&protocol, &treasury, ReleaseBucket::Company, 400).unwrap();
programs/treasury-router/src/engines/execution_guard.rs:484:        assert_eq!(authorization.requested_amount, 400);
programs/treasury-router/src/engines/execution_guard.rs:485:        assert_eq!(authorization.pending_balance, 800);
programs/treasury-router/src/engines/execution_guard.rs:486:        assert_eq!(authorization.maximum_release, 400);
programs/treasury-router/src/engines/execution_guard.rs:487:        assert_eq!(authorization.waterfall_stage, WaterfallStage::Caution);
programs/treasury-router/src/engines/execution_guard.rs:488:        assert_eq!(authorization.dam_level, DamLevel::Controlled);
programs/treasury-router/src/engines/execution_guard.rs:489:        assert_eq!(authorization.release_bps, 5_000);
programs/treasury-router/src/engines/execution_guard.rs:490:        assert_eq!(authorization.reserve_ratio_bps, 2_000);
programs/treasury-router/src/engines/execution_guard.rs:552:        assert_eq!(pending_balance(&treasury, ReleaseBucket::Reserve), 1_000);
programs/treasury-router/src/engines/execution_guard.rs:553:        assert_eq!(pending_balance(&treasury, ReleaseBucket::BuybackBurn), 100);
programs/treasury-router/src/engines/execution_guard.rs:554:        assert_eq!(pending_balance(&treasury, ReleaseBucket::Liquidity), 100);
programs/treasury-router/src/engines/execution_guard.rs:555:        assert_eq!(pending_balance(&treasury, ReleaseBucket::Company), 100);
programs/treasury-router/src/engines/execution_guard.rs:556:        assert_eq!(pending_balance(&treasury, ReleaseBucket::Founder), 100);
programs/treasury-router/src/engines/beaver_score.rs:214:        .unwrap();
programs/treasury-router/src/engines/beaver_score.rs:216:        assert_eq!(result.total_score, 1_000);
programs/treasury-router/src/engines/beaver_score.rs:217:        assert_eq!(result.waterfall_points, 350);
programs/treasury-router/src/engines/beaver_score.rs:218:        assert_eq!(result.dam_points, 250);
programs/treasury-router/src/engines/beaver_score.rs:219:        assert_eq!(result.reserve_points, 250);
programs/treasury-router/src/engines/beaver_score.rs:220:        assert_eq!(result.accounting_points, 150);
programs/treasury-router/src/engines/beaver_score.rs:228:            evaluate(&treasury, 3_000, WaterfallStage::Normal, DamLevel::Overflow).unwrap();
programs/treasury-router/src/engines/beaver_score.rs:230:        assert_eq!(result.reserve_points, 75);
programs/treasury-router/src/engines/beaver_score.rs:231:        assert_eq!(result.total_score, 825);
programs/treasury-router/src/engines/beaver_score.rs:238:        let result = evaluate(&treasury, 0, WaterfallStage::Emergency, DamLevel::Filling).unwrap();
programs/treasury-router/src/engines/beaver_score.rs:240:        assert_eq!(result.waterfall_points, 0);
programs/treasury-router/src/engines/beaver_score.rs:241:        assert_eq!(result.dam_points, 0);
programs/treasury-router/src/engines/beaver_score.rs:242:        assert_eq!(result.reserve_points, 0);
programs/treasury-router/src/engines/beaver_score.rs:243:        assert_eq!(result.accounting_points, 150);
programs/treasury-router/src/engines/beaver_score.rs:244:        assert_eq!(result.total_score, 150);
programs/treasury-router/src/engines/beaver_score.rs:253:            evaluate(&treasury, 3_000, WaterfallStage::Normal, DamLevel::Overflow).unwrap();
programs/treasury-router/src/engines/beaver_score.rs:255:        assert_eq!(result.accounting_points, 50);
programs/treasury-router/src/engines/beaver_score.rs:256:        assert_eq!(result.total_score, 725);
programs/treasury-router/src/engines/beaver_score.rs:261:        assert_eq!(score_reserve(10_000).unwrap(), 250);
programs/treasury-router/src/engines/beaver_score.rs:262:        assert_eq!(score_reserve(u16::MAX).unwrap(), 250);
programs/treasury-router/src/engines/beaver_score.rs:311:        let score = pre_dam_health_score(&treasury, 10_000, WaterfallStage::Normal).unwrap();
programs/treasury-router/src/engines/beaver_score.rs:313:        assert_eq!(score, PRE_DAM_MAX_POINTS);
programs/treasury-router/src/engines/beaver_score.rs:314:        assert_eq!(score, 750);
programs/treasury-router/src/engines/beaver_score.rs:321:        let score = pre_dam_health_score(&treasury, 0, WaterfallStage::Emergency).unwrap();
programs/treasury-router/src/engines/beaver_score.rs:323:        assert_eq!(score, ACCOUNTING_MAX_POINTS);
programs/treasury-router/src/engines/beaver_score.rs:331:        let score = pre_dam_health_score(&treasury, 10_000, WaterfallStage::Normal).unwrap();
programs/treasury-router/src/engines/beaver_score.rs:333:        assert_eq!(score, WATERFALL_MAX_POINTS + RESERVE_MAX_POINTS + 50);
programs/treasury-router/src/state/treasury.rs:184:        assert!(treasury.reserve_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:193:        assert!(!treasury.reserve_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:204:        assert!(!treasury.reserve_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:211:        assert!(treasury.buyback_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:220:        assert!(!treasury.buyback_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:231:        assert!(!treasury.buyback_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:238:        assert!(treasury.liquidity_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:247:        assert!(!treasury.liquidity_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:258:        assert!(!treasury.liquidity_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:265:        assert!(treasury.company_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:274:        assert!(!treasury.company_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:285:        assert!(!treasury.company_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:292:        assert!(treasury.founder_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:301:        assert!(!treasury.founder_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:312:        assert!(!treasury.founder_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:319:        assert!(treasury.execution_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:328:        assert!(!treasury.execution_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:337:        assert!(!treasury.execution_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:346:        assert!(!treasury.execution_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:355:        assert!(!treasury.execution_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:364:        assert!(!treasury.execution_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:375:        assert!(!treasury.execution_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:402:        assert!(!treasury.reserve_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:403:        assert!(!treasury.buyback_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:404:        assert!(!treasury.liquidity_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:405:        assert!(!treasury.company_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:406:        assert!(!treasury.founder_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:407:        assert!(!treasury.execution_accounting_is_valid());
programs/treasury-router/src/state/treasury.rs:432:        assert!(treasury.execution_accounting_is_valid());
programs/treasury-router/src/instructions/process_fees.rs:589:        let result = calculate_share(1_000_000, 3_000).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:591:        assert_eq!(result, 300_000);
programs/treasury-router/src/instructions/process_fees.rs:596:        let result = calculate_share(101, 2_000).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:598:        assert_eq!(result, 20);
programs/treasury-router/src/instructions/process_fees.rs:603:        let result = calculate_share(0, 3_000).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:605:        assert_eq!(result, 0);
programs/treasury-router/src/instructions/process_fees.rs:610:        let result = calculate_share(1_000_000, 0).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:612:        assert_eq!(result, 0);
programs/treasury-router/src/instructions/process_fees.rs:617:        let result = calculate_share(1_000_000, BPS_DENOMINATOR).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:619:        assert_eq!(result, 1_000_000);
programs/treasury-router/src/instructions/process_fees.rs:624:        let result = calculate_share(u64::MAX, BPS_DENOMINATOR).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:626:        assert_eq!(result, u64::MAX);
programs/treasury-router/src/instructions/process_fees.rs:633:        let reserve = calculate_share(amount, 3_000).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:634:        let buyback = calculate_share(amount, 2_000).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:635:        let liquidity = calculate_share(amount, 2_000).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:636:        let company = calculate_share(amount, 2_000).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:637:        let founder = calculate_share(amount, 1_000).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:639:        let total = calculate_total(reserve, buyback, liquidity, company, founder).unwrap();
programs/treasury-router/src/instructions/process_fees.rs:641:        assert_eq!(reserve, 300_000);
programs/treasury-router/src/instructions/process_fees.rs:642:        assert_eq!(buyback, 200_000);
```
- ⚠️ **REVIEW:** Potential panic-producing calls exist. Review the listed locations and distinguish test-only modules from reachable production code.

## Authority and privileged-control inventory

```text
programs/treasury-router/src/engines/sentinel.rs:345:/// receive mutable accounts or signer authority.
programs/treasury-router/src/engines/sentinel.rs:397:     * Ownership and SPL-token mint checks require actual token-account data
programs/treasury-router/src/engines/integrity_firewall.rs:26:    ProtocolAuthority = 4,
programs/treasury-router/src/engines/integrity_firewall.rs:41:    TreasuryVaultOwner = 16,
programs/treasury-router/src/engines/integrity_firewall.rs:60:    FounderDestinationOwner = 31,
programs/treasury-router/src/engines/integrity_firewall.rs:61:    CompanyDestinationOwner = 32,
programs/treasury-router/src/engines/integrity_firewall.rs:74:/// accounts, signer authority, or CPI capability.
programs/treasury-router/src/engines/integrity_firewall.rs:79:    pub owner: Pubkey,
programs/treasury-router/src/engines/integrity_firewall.rs:167:    if protocol.authority == Pubkey::default() {
programs/treasury-router/src/engines/integrity_firewall.rs:168:        fail(IntegrityInvariant::ProtocolAuthority);
programs/treasury-router/src/engines/integrity_firewall.rs:234:    if treasury_vault.owner != keys.treasury {
programs/treasury-router/src/engines/integrity_firewall.rs:235:        fail(IntegrityInvariant::TreasuryVaultOwner);
programs/treasury-router/src/engines/integrity_firewall.rs:340:     * Recipient ownership
programs/treasury-router/src/engines/integrity_firewall.rs:342:    if destinations.founder.owner != founder.recipient {
programs/treasury-router/src/engines/integrity_firewall.rs:343:        fail(IntegrityInvariant::FounderDestinationOwner);
programs/treasury-router/src/engines/integrity_firewall.rs:346:    if destinations.company.owner != company.recipient {
programs/treasury-router/src/engines/integrity_firewall.rs:347:        fail(IntegrityInvariant::CompanyDestinationOwner);
programs/treasury-router/src/engines/integrity_firewall.rs:448:            authority: Pubkey::new_unique(),
programs/treasury-router/src/engines/integrity_firewall.rs:557:            owner: treasury_key,
programs/treasury-router/src/engines/integrity_firewall.rs:564:                owner: Pubkey::new_unique(),
programs/treasury-router/src/engines/integrity_firewall.rs:569:                owner: Pubkey::new_unique(),
programs/treasury-router/src/engines/integrity_firewall.rs:574:                owner: Pubkey::new_unique(),
programs/treasury-router/src/engines/integrity_firewall.rs:579:                owner: company_recipient,
programs/treasury-router/src/engines/integrity_firewall.rs:584:                owner: founder_recipient,
programs/treasury-router/src/engines/integrity_firewall.rs:670:    fn wrong_treasury_vault_owner_is_detected() {
programs/treasury-router/src/engines/integrity_firewall.rs:672:        fixture.treasury_vault.owner = Pubkey::new_unique();
programs/treasury-router/src/engines/integrity_firewall.rs:678:            IntegrityInvariant::TreasuryVaultOwner
programs/treasury-router/src/engines/integrity_firewall.rs:696:    fn wrong_founder_destination_owner_is_detected() {
programs/treasury-router/src/engines/integrity_firewall.rs:698:        fixture.destinations.founder.owner = Pubkey::new_unique();
programs/treasury-router/src/engines/integrity_firewall.rs:704:            IntegrityInvariant::FounderDestinationOwner
programs/treasury-router/src/engines/integrity_firewall.rs:743:            IntegrityInvariant::ProtocolAuthority,
programs/treasury-router/src/engines/integrity_firewall.rs:755:            IntegrityInvariant::TreasuryVaultOwner,
programs/treasury-router/src/engines/integrity_firewall.rs:770:            IntegrityInvariant::FounderDestinationOwner,
programs/treasury-router/src/engines/integrity_firewall.rs:771:            IntegrityInvariant::CompanyDestinationOwner,
programs/treasury-router/src/engines/execution_guard.rs:37:/// Account ownership, PDA seeds, signer authority, mint validation, and
programs/treasury-router/src/engines/execution_guard.rs:170:            authority: Pubkey::new_unique(),
programs/treasury-router/src/events/mod.rs:11:    pub authority: Pubkey,
programs/treasury-router/src/events/mod.rs:33:    pub authority: Pubkey,
programs/treasury-router/src/events/mod.rs:55:    pub authority: Pubkey,
programs/treasury-router/src/events/mod.rs:78:    pub authority: Pubkey,
programs/treasury-router/src/events/mod.rs:105:    pub authority: Pubkey,
programs/treasury-router/src/state/protocol.rs:8:    /// Current protocol administration authority.
programs/treasury-router/src/state/protocol.rs:9:    pub authority: Pubkey,
programs/treasury-router/src/state/protocol.rs:41:        (32 * 8) + // authority, config, and six module Pubkeys
programs/treasury-router/src/instructions/deposit_settlement.rs:15:        has_one = authority,
programs/treasury-router/src/instructions/deposit_settlement.rs:48:        constraint = source_token_account.owner == authority.key()
programs/treasury-router/src/instructions/deposit_settlement.rs:59:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/deposit_settlement.rs:64:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/deposit_settlement.rs:81:        authority: ctx.accounts.authority.to_account_info(),
programs/treasury-router/src/instructions/initialize_company.rs:15:        has_one = authority
programs/treasury-router/src/instructions/initialize_company.rs:21:        payer = authority,
programs/treasury-router/src/instructions/initialize_company.rs:32:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/process_fees.rs:77:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/reserve.rs:28:        has_one = authority,
programs/treasury-router/src/instructions/reserve.rs:116:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/reserve.rs:161:        constraint = company_destination.owner == company_state.recipient
programs/treasury-router/src/instructions/reserve.rs:173:        constraint = founder_destination.owner == founder_state.recipient
programs/treasury-router/src/instructions/reserve.rs:178:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/reserve.rs:209:            owner: ctx.accounts.settlement_vault.owner,
programs/treasury-router/src/instructions/reserve.rs:218:                owner: ctx.accounts.reserve_destination.owner,
programs/treasury-router/src/instructions/reserve.rs:223:                owner: ctx.accounts.buyback_destination.owner,
programs/treasury-router/src/instructions/reserve.rs:228:                owner: ctx.accounts.liquidity_destination.owner,
programs/treasury-router/src/instructions/reserve.rs:233:                owner: ctx.accounts.company_destination.owner,
programs/treasury-router/src/instructions/reserve.rs:238:                owner: ctx.accounts.founder_destination.owner,
programs/treasury-router/src/instructions/reserve.rs:263:    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];
programs/treasury-router/src/instructions/reserve.rs:265:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/reserve.rs:271:        authority: ctx.accounts.treasury.to_account_info(),
programs/treasury-router/src/instructions/reserve.rs:274:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/reserve.rs:277:        signer_seeds,
programs/treasury-router/src/instructions/reserve.rs:294:        authority: ctx.accounts.authority.key(),
programs/treasury-router/src/instructions/initialize_founder.rs:15:        has_one = authority
programs/treasury-router/src/instructions/initialize_founder.rs:21:        payer = authority,
programs/treasury-router/src/instructions/initialize_founder.rs:32:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/founder.rs:18:        has_one = authority,
programs/treasury-router/src/instructions/founder.rs:80:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/founder.rs:95:        constraint = founder_destination.owner == founder_state.recipient
programs/treasury-router/src/instructions/founder.rs:100:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/founder.rs:123:    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];
programs/treasury-router/src/instructions/founder.rs:125:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/founder.rs:131:        authority: ctx.accounts.treasury.to_account_info(),
programs/treasury-router/src/instructions/founder.rs:134:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/founder.rs:137:        signer_seeds,
programs/treasury-router/src/instructions/founder.rs:155:        authority: ctx.accounts.authority.key(),
programs/treasury-router/src/instructions/company.rs:18:        has_one = authority,
programs/treasury-router/src/instructions/company.rs:80:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/company.rs:95:        constraint = company_destination.owner == company_state.recipient
programs/treasury-router/src/instructions/company.rs:100:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/company.rs:123:    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];
programs/treasury-router/src/instructions/company.rs:125:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/company.rs:131:        authority: ctx.accounts.treasury.to_account_info(),
programs/treasury-router/src/instructions/company.rs:134:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/company.rs:137:        signer_seeds,
programs/treasury-router/src/instructions/company.rs:155:        authority: ctx.accounts.authority.key(),
programs/treasury-router/src/instructions/initialize_execution_config.rs:18:        has_one = authority,
programs/treasury-router/src/instructions/initialize_execution_config.rs:78:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:104:        constraint = company_destination.owner == company_state.recipient
programs/treasury-router/src/instructions/initialize_execution_config.rs:112:        constraint = founder_destination.owner == founder_state.recipient
programs/treasury-router/src/instructions/initialize_execution_config.rs:119:        payer = authority,
programs/treasury-router/src/instructions/initialize_execution_config.rs:130:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize.rs:12:        payer = authority,
programs/treasury-router/src/instructions/initialize.rs:20:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize.rs:30:    protocol_state.authority = ctx.accounts.authority.key();
programs/treasury-router/src/instructions/initialize_treasury.rs:19:        has_one = authority
programs/treasury-router/src/instructions/initialize_treasury.rs:25:        payer = authority,
programs/treasury-router/src/instructions/initialize_treasury.rs:41:        payer = authority,
programs/treasury-router/src/instructions/initialize_treasury.rs:48:        token::authority = treasury_state
programs/treasury-router/src/instructions/initialize_treasury.rs:53:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/initialize_treasury.rs:115:    msg!("Settlement vault authority: {}", treasury_state.key());
programs/treasury-router/src/instructions/initialize_protocol_config.rs:19:        has_one = authority
programs/treasury-router/src/instructions/initialize_protocol_config.rs:25:        payer = authority,
programs/treasury-router/src/instructions/initialize_protocol_config.rs:36:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/liquidity.rs:28:        has_one = authority,
programs/treasury-router/src/instructions/liquidity.rs:116:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/liquidity.rs:161:        constraint = company_destination.owner == company_state.recipient
programs/treasury-router/src/instructions/liquidity.rs:173:        constraint = founder_destination.owner == founder_state.recipient
programs/treasury-router/src/instructions/liquidity.rs:178:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/liquidity.rs:206:            owner: ctx.accounts.settlement_vault.owner,
programs/treasury-router/src/instructions/liquidity.rs:215:                owner: ctx.accounts.reserve_destination.owner,
programs/treasury-router/src/instructions/liquidity.rs:220:                owner: ctx.accounts.buyback_destination.owner,
programs/treasury-router/src/instructions/liquidity.rs:225:                owner: ctx.accounts.liquidity_destination.owner,
programs/treasury-router/src/instructions/liquidity.rs:230:                owner: ctx.accounts.company_destination.owner,
programs/treasury-router/src/instructions/liquidity.rs:235:                owner: ctx.accounts.founder_destination.owner,
programs/treasury-router/src/instructions/liquidity.rs:260:    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];
programs/treasury-router/src/instructions/liquidity.rs:262:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/liquidity.rs:268:        authority: ctx.accounts.treasury.to_account_info(),
programs/treasury-router/src/instructions/liquidity.rs:271:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/liquidity.rs:274:        signer_seeds,
programs/treasury-router/src/instructions/liquidity.rs:291:        authority: ctx.accounts.authority.key(),
programs/treasury-router/src/instructions/buyback.rs:28:        has_one = authority,
programs/treasury-router/src/instructions/buyback.rs:116:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/buyback.rs:161:        constraint = company_destination.owner == company_state.recipient
programs/treasury-router/src/instructions/buyback.rs:173:        constraint = founder_destination.owner == founder_state.recipient
programs/treasury-router/src/instructions/buyback.rs:178:    pub authority: Signer<'info>,
programs/treasury-router/src/instructions/buyback.rs:209:            owner: ctx.accounts.settlement_vault.owner,
programs/treasury-router/src/instructions/buyback.rs:218:                owner: ctx.accounts.reserve_destination.owner,
programs/treasury-router/src/instructions/buyback.rs:223:                owner: ctx.accounts.buyback_destination.owner,
programs/treasury-router/src/instructions/buyback.rs:228:                owner: ctx.accounts.liquidity_destination.owner,
programs/treasury-router/src/instructions/buyback.rs:233:                owner: ctx.accounts.company_destination.owner,
programs/treasury-router/src/instructions/buyback.rs:238:                owner: ctx.accounts.founder_destination.owner,
programs/treasury-router/src/instructions/buyback.rs:263:    let treasury_signer_seeds: &[&[u8]] = &[TREASURY_SEED, protocol_key.as_ref(), &treasury_bump];
programs/treasury-router/src/instructions/buyback.rs:265:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/buyback.rs:271:        authority: ctx.accounts.treasury.to_account_info(),
programs/treasury-router/src/instructions/buyback.rs:274:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/buyback.rs:277:        signer_seeds,
programs/treasury-router/src/instructions/buyback.rs:298:        authority: ctx.accounts.authority.key(),
programs/treasury-router/src/errors/mod.rs:5:    #[msg("The supplied authority is not authorized.")]
```

## PDA and account-linkage inventory

```text
programs/treasury-router/src/engines/integrity_firewall.rs:24:    ProtocolBump = 2,
programs/treasury-router/src/engines/integrity_firewall.rs:29:    ProtocolConfigBump = 6,
programs/treasury-router/src/engines/integrity_firewall.rs:34:    TreasuryBump = 10,
programs/treasury-router/src/engines/integrity_firewall.rs:44:    FounderBump = 18,
programs/treasury-router/src/engines/integrity_firewall.rs:49:    CompanyBump = 22,
programs/treasury-router/src/engines/integrity_firewall.rs:54:    ExecutionConfigBump = 26,
programs/treasury-router/src/engines/integrity_firewall.rs:659:        fixture.treasury.bump = fixture.treasury.bump.wrapping_add(1);
programs/treasury-router/src/instructions/deposit_settlement.rs:13:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/deposit_settlement.rs:14:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/deposit_settlement.rs:15:        has_one = authority,
programs/treasury-router/src/instructions/deposit_settlement.rs:16:        constraint = protocol_state.treasury_state != Pubkey::default()
programs/treasury-router/src/instructions/deposit_settlement.rs:18:        constraint = protocol_state.treasury_state == treasury.key()
programs/treasury-router/src/instructions/deposit_settlement.rs:24:        seeds = [
programs/treasury-router/src/instructions/deposit_settlement.rs:28:        bump = treasury.bump,
programs/treasury-router/src/instructions/deposit_settlement.rs:29:        constraint = treasury.protocol == protocol_state.key()
programs/treasury-router/src/instructions/deposit_settlement.rs:31:        constraint = treasury.settlement_mint == settlement_mint.key()
programs/treasury-router/src/instructions/deposit_settlement.rs:33:        constraint = treasury.settlement_vault == settlement_vault.key()
programs/treasury-router/src/instructions/deposit_settlement.rs:39:        constraint = settlement_mint.key() == treasury.settlement_mint
programs/treasury-router/src/instructions/deposit_settlement.rs:46:        constraint = source_token_account.mint == settlement_mint.key()
programs/treasury-router/src/instructions/deposit_settlement.rs:48:        constraint = source_token_account.owner == authority.key()
programs/treasury-router/src/instructions/deposit_settlement.rs:55:        constraint = settlement_vault.key() == treasury.settlement_vault
programs/treasury-router/src/instructions/deposit_settlement.rs:57:        constraint = settlement_vault.mint == settlement_mint.key()
programs/treasury-router/src/instructions/deposit_settlement.rs:59:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/initialize_company.rs:13:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/initialize_company.rs:14:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/initialize_company.rs:15:        has_one = authority
programs/treasury-router/src/instructions/initialize_company.rs:23:        seeds = [
programs/treasury-router/src/instructions/initialize_company.rs:78:    company_state.bump = ctx.bumps.company_state;
programs/treasury-router/src/instructions/process_fees.rs:20:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/process_fees.rs:21:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/process_fees.rs:22:        has_one = protocol_config,
programs/treasury-router/src/instructions/process_fees.rs:23:        constraint = protocol_state.treasury_state == treasury.key(),
programs/treasury-router/src/instructions/process_fees.rs:24:        constraint = protocol_state.founder_state == founder_state.key(),
programs/treasury-router/src/instructions/process_fees.rs:25:        constraint = protocol_state.company_state == company_state.key()
programs/treasury-router/src/instructions/process_fees.rs:30:        seeds = [
programs/treasury-router/src/instructions/process_fees.rs:34:        bump = protocol_config.bump,
programs/treasury-router/src/instructions/process_fees.rs:35:        constraint = protocol_config.protocol == protocol_state.key()
programs/treasury-router/src/instructions/process_fees.rs:41:        seeds = [
programs/treasury-router/src/instructions/process_fees.rs:45:        bump = treasury.bump,
programs/treasury-router/src/instructions/process_fees.rs:46:        constraint = treasury.protocol == protocol_state.key()
programs/treasury-router/src/instructions/process_fees.rs:52:        seeds = [
programs/treasury-router/src/instructions/process_fees.rs:56:        bump = founder_state.bump,
programs/treasury-router/src/instructions/process_fees.rs:57:        constraint = founder_state.protocol == protocol_state.key()
programs/treasury-router/src/instructions/process_fees.rs:63:        seeds = [
programs/treasury-router/src/instructions/process_fees.rs:67:        bump = company_state.bump,
programs/treasury-router/src/instructions/process_fees.rs:68:        constraint = company_state.protocol == protocol_state.key()
programs/treasury-router/src/instructions/process_fees.rs:73:        constraint = settlement_vault.key() == treasury.settlement_vault
programs/treasury-router/src/instructions/process_fees.rs:75:        constraint = settlement_vault.mint == treasury.settlement_mint
programs/treasury-router/src/instructions/process_fees.rs:77:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/reserve.rs:26:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/reserve.rs:27:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/reserve.rs:28:        has_one = authority,
programs/treasury-router/src/instructions/reserve.rs:29:        constraint = protocol_state.treasury_state != Pubkey::default()
programs/treasury-router/src/instructions/reserve.rs:31:        constraint = protocol_state.treasury_state == treasury.key()
programs/treasury-router/src/instructions/reserve.rs:37:        seeds = [
programs/treasury-router/src/instructions/reserve.rs:41:        bump = protocol_config.bump,
programs/treasury-router/src/instructions/reserve.rs:42:        constraint = protocol_state.protocol_config == protocol_config.key()
programs/treasury-router/src/instructions/reserve.rs:44:        constraint = protocol_config.protocol == protocol_state.key()
programs/treasury-router/src/instructions/reserve.rs:50:        seeds = [
programs/treasury-router/src/instructions/reserve.rs:54:        bump = founder_state.bump,
programs/treasury-router/src/instructions/reserve.rs:55:        constraint = protocol_state.founder_state == founder_state.key()
programs/treasury-router/src/instructions/reserve.rs:57:        constraint = founder_state.protocol == protocol_state.key()
programs/treasury-router/src/instructions/reserve.rs:63:        seeds = [
programs/treasury-router/src/instructions/reserve.rs:67:        bump = company_state.bump,
programs/treasury-router/src/instructions/reserve.rs:68:        constraint = protocol_state.company_state == company_state.key()
programs/treasury-router/src/instructions/reserve.rs:70:        constraint = company_state.protocol == protocol_state.key()
programs/treasury-router/src/instructions/reserve.rs:77:        seeds = [
programs/treasury-router/src/instructions/reserve.rs:81:        bump = treasury.bump,
programs/treasury-router/src/instructions/reserve.rs:82:        constraint = treasury.protocol == protocol_state.key()
programs/treasury-router/src/instructions/reserve.rs:84:        constraint = treasury.settlement_mint == settlement_mint.key()
programs/treasury-router/src/instructions/reserve.rs:86:        constraint = treasury.settlement_vault == settlement_vault.key()
programs/treasury-router/src/instructions/reserve.rs:92:        seeds = [
programs/treasury-router/src/instructions/reserve.rs:96:        bump = execution_config.bump,
programs/treasury-router/src/instructions/reserve.rs:97:        constraint = execution_config.protocol_state == protocol_state.key()
programs/treasury-router/src/instructions/reserve.rs:99:        constraint = execution_config.settlement_mint == settlement_mint.key()
programs/treasury-router/src/instructions/reserve.rs:105:        constraint = settlement_mint.key() == treasury.settlement_mint
programs/treasury-router/src/instructions/reserve.rs:112:        constraint = settlement_vault.key() == treasury.settlement_vault
programs/treasury-router/src/instructions/reserve.rs:114:        constraint = settlement_vault.mint == settlement_mint.key()
programs/treasury-router/src/instructions/reserve.rs:116:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/reserve.rs:124:        constraint = reserve_destination.key()
programs/treasury-router/src/instructions/reserve.rs:127:        constraint = reserve_destination.key() != settlement_vault.key()
programs/treasury-router/src/instructions/reserve.rs:129:        constraint = reserve_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/reserve.rs:136:        constraint = buyback_destination.key()
programs/treasury-router/src/instructions/reserve.rs:139:        constraint = buyback_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/reserve.rs:146:        constraint = liquidity_destination.key()
programs/treasury-router/src/instructions/reserve.rs:149:        constraint = liquidity_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/reserve.rs:156:        constraint = company_destination.key()
programs/treasury-router/src/instructions/reserve.rs:159:        constraint = company_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/reserve.rs:161:        constraint = company_destination.owner == company_state.recipient
programs/treasury-router/src/instructions/reserve.rs:168:        constraint = founder_destination.key()
programs/treasury-router/src/instructions/reserve.rs:171:        constraint = founder_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/reserve.rs:173:        constraint = founder_destination.owner == founder_state.recipient
programs/treasury-router/src/instructions/reserve.rs:261:    let treasury_bump = [ctx.accounts.treasury.bump];
programs/treasury-router/src/instructions/reserve.rs:265:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/initialize_founder.rs:13:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/initialize_founder.rs:14:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/initialize_founder.rs:15:        has_one = authority
programs/treasury-router/src/instructions/initialize_founder.rs:23:        seeds = [
programs/treasury-router/src/instructions/initialize_founder.rs:79:    founder_state.bump = ctx.bumps.founder_state;
programs/treasury-router/src/instructions/founder.rs:16:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/founder.rs:17:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/founder.rs:18:        has_one = authority,
programs/treasury-router/src/instructions/founder.rs:19:        constraint = protocol_state.treasury_state != Pubkey::default()
programs/treasury-router/src/instructions/founder.rs:21:        constraint = protocol_state.treasury_state == treasury.key()
programs/treasury-router/src/instructions/founder.rs:23:        constraint = protocol_state.founder_state == founder_state.key()
programs/treasury-router/src/instructions/founder.rs:30:        seeds = [
programs/treasury-router/src/instructions/founder.rs:34:        bump = treasury.bump,
programs/treasury-router/src/instructions/founder.rs:35:        constraint = treasury.protocol == protocol_state.key()
programs/treasury-router/src/instructions/founder.rs:37:        constraint = treasury.settlement_mint == settlement_mint.key()
programs/treasury-router/src/instructions/founder.rs:39:        constraint = treasury.settlement_vault == settlement_vault.key()
programs/treasury-router/src/instructions/founder.rs:45:        seeds = [
programs/treasury-router/src/instructions/founder.rs:49:        bump = founder_state.bump,
programs/treasury-router/src/instructions/founder.rs:50:        constraint = founder_state.protocol == protocol_state.key()
programs/treasury-router/src/instructions/founder.rs:56:        seeds = [
programs/treasury-router/src/instructions/founder.rs:60:        bump = execution_config.bump,
programs/treasury-router/src/instructions/founder.rs:61:        constraint = execution_config.protocol_state == protocol_state.key()
programs/treasury-router/src/instructions/founder.rs:63:        constraint = execution_config.settlement_mint == settlement_mint.key()
programs/treasury-router/src/instructions/founder.rs:69:        constraint = settlement_mint.key() == treasury.settlement_mint
programs/treasury-router/src/instructions/founder.rs:76:        constraint = settlement_vault.key() == treasury.settlement_vault
programs/treasury-router/src/instructions/founder.rs:78:        constraint = settlement_vault.mint == settlement_mint.key()
programs/treasury-router/src/instructions/founder.rs:80:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/founder.rs:88:        constraint = founder_destination.key()
programs/treasury-router/src/instructions/founder.rs:91:        constraint = founder_destination.key() != settlement_vault.key()
programs/treasury-router/src/instructions/founder.rs:93:        constraint = founder_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/founder.rs:95:        constraint = founder_destination.owner == founder_state.recipient
programs/treasury-router/src/instructions/founder.rs:121:    let treasury_bump = [ctx.accounts.treasury.bump];
programs/treasury-router/src/instructions/founder.rs:125:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/company.rs:16:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/company.rs:17:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/company.rs:18:        has_one = authority,
programs/treasury-router/src/instructions/company.rs:19:        constraint = protocol_state.treasury_state != Pubkey::default()
programs/treasury-router/src/instructions/company.rs:21:        constraint = protocol_state.treasury_state == treasury.key()
programs/treasury-router/src/instructions/company.rs:23:        constraint = protocol_state.company_state == company_state.key()
programs/treasury-router/src/instructions/company.rs:30:        seeds = [
programs/treasury-router/src/instructions/company.rs:34:        bump = treasury.bump,
programs/treasury-router/src/instructions/company.rs:35:        constraint = treasury.protocol == protocol_state.key()
programs/treasury-router/src/instructions/company.rs:37:        constraint = treasury.settlement_mint == settlement_mint.key()
programs/treasury-router/src/instructions/company.rs:39:        constraint = treasury.settlement_vault == settlement_vault.key()
programs/treasury-router/src/instructions/company.rs:45:        seeds = [
programs/treasury-router/src/instructions/company.rs:49:        bump = company_state.bump,
programs/treasury-router/src/instructions/company.rs:50:        constraint = company_state.protocol == protocol_state.key()
programs/treasury-router/src/instructions/company.rs:56:        seeds = [
programs/treasury-router/src/instructions/company.rs:60:        bump = execution_config.bump,
programs/treasury-router/src/instructions/company.rs:61:        constraint = execution_config.protocol_state == protocol_state.key()
programs/treasury-router/src/instructions/company.rs:63:        constraint = execution_config.settlement_mint == settlement_mint.key()
programs/treasury-router/src/instructions/company.rs:69:        constraint = settlement_mint.key() == treasury.settlement_mint
programs/treasury-router/src/instructions/company.rs:76:        constraint = settlement_vault.key() == treasury.settlement_vault
programs/treasury-router/src/instructions/company.rs:78:        constraint = settlement_vault.mint == settlement_mint.key()
programs/treasury-router/src/instructions/company.rs:80:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/company.rs:88:        constraint = company_destination.key()
programs/treasury-router/src/instructions/company.rs:91:        constraint = company_destination.key() != settlement_vault.key()
programs/treasury-router/src/instructions/company.rs:93:        constraint = company_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/company.rs:95:        constraint = company_destination.owner == company_state.recipient
programs/treasury-router/src/instructions/company.rs:121:    let treasury_bump = [ctx.accounts.treasury.bump];
programs/treasury-router/src/instructions/company.rs:125:    let signer_seeds = &[treasury_signer_seeds];
programs/treasury-router/src/instructions/initialize_execution_config.rs:16:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/initialize_execution_config.rs:17:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/initialize_execution_config.rs:18:        has_one = authority,
programs/treasury-router/src/instructions/initialize_execution_config.rs:19:        constraint = !protocol_state.paused
programs/treasury-router/src/instructions/initialize_execution_config.rs:21:        constraint = protocol_state.treasury_state == treasury.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:23:        constraint = protocol_state.founder_state == founder_state.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:25:        constraint = protocol_state.company_state == company_state.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:31:        seeds = [
programs/treasury-router/src/instructions/initialize_execution_config.rs:35:        bump = treasury.bump,
programs/treasury-router/src/instructions/initialize_execution_config.rs:36:        constraint = treasury.protocol == protocol_state.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:38:        constraint = treasury.settlement_mint == settlement_mint.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:40:        constraint = treasury.settlement_vault == settlement_vault.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:46:        seeds = [
programs/treasury-router/src/instructions/initialize_execution_config.rs:50:        bump = founder_state.bump,
programs/treasury-router/src/instructions/initialize_execution_config.rs:51:        constraint = founder_state.protocol == protocol_state.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:57:        seeds = [
programs/treasury-router/src/instructions/initialize_execution_config.rs:61:        bump = company_state.bump,
programs/treasury-router/src/instructions/initialize_execution_config.rs:62:        constraint = company_state.protocol == protocol_state.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:68:        constraint = settlement_mint.key() == treasury.settlement_mint
programs/treasury-router/src/instructions/initialize_execution_config.rs:74:        constraint = settlement_vault.key() == treasury.settlement_vault
programs/treasury-router/src/instructions/initialize_execution_config.rs:76:        constraint = settlement_vault.mint == settlement_mint.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:78:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:84:        constraint = reserve_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:90:        constraint = buyback_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:96:        constraint = liquidity_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:102:        constraint = company_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:104:        constraint = company_destination.owner == company_state.recipient
programs/treasury-router/src/instructions/initialize_execution_config.rs:110:        constraint = founder_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/initialize_execution_config.rs:112:        constraint = founder_destination.owner == founder_state.recipient
programs/treasury-router/src/instructions/initialize_execution_config.rs:121:        seeds = [
programs/treasury-router/src/instructions/initialize_execution_config.rs:235:    execution_config.bump = ctx.bumps.execution_config;
programs/treasury-router/src/instructions/initialize.rs:14:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/initialize.rs:43:    protocol_state.bump = ctx.bumps.protocol_state;
programs/treasury-router/src/instructions/initialize_treasury.rs:17:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/initialize_treasury.rs:18:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/initialize_treasury.rs:19:        has_one = authority
programs/treasury-router/src/instructions/initialize_treasury.rs:27:        seeds = [
programs/treasury-router/src/instructions/initialize_treasury.rs:42:        seeds = [
programs/treasury-router/src/instructions/initialize_treasury.rs:102:    treasury_state.bump = ctx.bumps.treasury_state;
programs/treasury-router/src/instructions/initialize_protocol_config.rs:17:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/initialize_protocol_config.rs:18:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/initialize_protocol_config.rs:19:        has_one = authority
programs/treasury-router/src/instructions/initialize_protocol_config.rs:27:        seeds = [
programs/treasury-router/src/instructions/initialize_protocol_config.rs:78:    protocol_config.bump = ctx.bumps.protocol_config;
programs/treasury-router/src/instructions/liquidity.rs:26:        seeds = [PROTOCOL_SEED],
programs/treasury-router/src/instructions/liquidity.rs:27:        bump = protocol_state.bump,
programs/treasury-router/src/instructions/liquidity.rs:28:        has_one = authority,
programs/treasury-router/src/instructions/liquidity.rs:29:        constraint = protocol_state.treasury_state != Pubkey::default()
programs/treasury-router/src/instructions/liquidity.rs:31:        constraint = protocol_state.treasury_state == treasury.key()
programs/treasury-router/src/instructions/liquidity.rs:37:        seeds = [
programs/treasury-router/src/instructions/liquidity.rs:41:        bump = protocol_config.bump,
programs/treasury-router/src/instructions/liquidity.rs:42:        constraint = protocol_state.protocol_config == protocol_config.key()
programs/treasury-router/src/instructions/liquidity.rs:44:        constraint = protocol_config.protocol == protocol_state.key()
programs/treasury-router/src/instructions/liquidity.rs:50:        seeds = [
programs/treasury-router/src/instructions/liquidity.rs:54:        bump = founder_state.bump,
programs/treasury-router/src/instructions/liquidity.rs:55:        constraint = protocol_state.founder_state == founder_state.key()
programs/treasury-router/src/instructions/liquidity.rs:57:        constraint = founder_state.protocol == protocol_state.key()
programs/treasury-router/src/instructions/liquidity.rs:63:        seeds = [
programs/treasury-router/src/instructions/liquidity.rs:67:        bump = company_state.bump,
programs/treasury-router/src/instructions/liquidity.rs:68:        constraint = protocol_state.company_state == company_state.key()
programs/treasury-router/src/instructions/liquidity.rs:70:        constraint = company_state.protocol == protocol_state.key()
programs/treasury-router/src/instructions/liquidity.rs:77:        seeds = [
programs/treasury-router/src/instructions/liquidity.rs:81:        bump = treasury.bump,
programs/treasury-router/src/instructions/liquidity.rs:82:        constraint = treasury.protocol == protocol_state.key()
programs/treasury-router/src/instructions/liquidity.rs:84:        constraint = treasury.settlement_mint == settlement_mint.key()
programs/treasury-router/src/instructions/liquidity.rs:86:        constraint = treasury.settlement_vault == settlement_vault.key()
programs/treasury-router/src/instructions/liquidity.rs:92:        seeds = [
programs/treasury-router/src/instructions/liquidity.rs:96:        bump = execution_config.bump,
programs/treasury-router/src/instructions/liquidity.rs:97:        constraint = execution_config.protocol_state == protocol_state.key()
programs/treasury-router/src/instructions/liquidity.rs:99:        constraint = execution_config.settlement_mint == settlement_mint.key()
programs/treasury-router/src/instructions/liquidity.rs:105:        constraint = settlement_mint.key() == treasury.settlement_mint
programs/treasury-router/src/instructions/liquidity.rs:112:        constraint = settlement_vault.key() == treasury.settlement_vault
programs/treasury-router/src/instructions/liquidity.rs:114:        constraint = settlement_vault.mint == settlement_mint.key()
programs/treasury-router/src/instructions/liquidity.rs:116:        constraint = settlement_vault.owner == treasury.key()
programs/treasury-router/src/instructions/liquidity.rs:124:        constraint = liquidity_destination.key()
programs/treasury-router/src/instructions/liquidity.rs:127:        constraint = liquidity_destination.key() != settlement_vault.key()
programs/treasury-router/src/instructions/liquidity.rs:129:        constraint = liquidity_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/liquidity.rs:136:        constraint = reserve_destination.key()
programs/treasury-router/src/instructions/liquidity.rs:139:        constraint = reserve_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/liquidity.rs:146:        constraint = buyback_destination.key()
programs/treasury-router/src/instructions/liquidity.rs:149:        constraint = buyback_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/liquidity.rs:156:        constraint = company_destination.key()
programs/treasury-router/src/instructions/liquidity.rs:159:        constraint = company_destination.mint == settlement_mint.key()
programs/treasury-router/src/instructions/liquidity.rs:161:        constraint = company_destination.owner == company_state.recipient
programs/treasury-router/src/instructions/liquidity.rs:168:        constraint = founder_destination.key()
```

## CPI and external-program inventory

```text
programs/treasury-router/src/engines/sentinel.rs:287:pub const LOCKED_BUYBACK_BURN_BPS: u16 = 2_000;
programs/treasury-router/src/engines/sentinel.rs:376:        && protocol_config.buyback_burn_bps == LOCKED_BUYBACK_BURN_BPS
programs/treasury-router/src/engines/sentinel.rs:420:        .checked_add(treasury.lifetime_buyback_burn)
programs/treasury-router/src/engines/sentinel.rs:483:            buyback_burn_bps: LOCKED_BUYBACK_BURN_BPS,
programs/treasury-router/src/engines/sentinel.rs:505:            pending_buyback_burn: 200,
programs/treasury-router/src/engines/sentinel.rs:511:            released_buyback_burn: 0,
programs/treasury-router/src/engines/sentinel.rs:517:            lifetime_buyback_burn: 200,
programs/treasury-router/src/engines/sentinel.rs:639:        treasury.lifetime_buyback_burn = 1;
programs/treasury-router/src/engines/integrity_firewall.rs:468:            buyback_burn_bps: 2_000,
programs/treasury-router/src/engines/integrity_firewall.rs:488:            pending_buyback_burn: 200,
programs/treasury-router/src/engines/integrity_firewall.rs:494:            lifetime_buyback_burn: 200,
programs/treasury-router/src/engines/integrity_firewall.rs:500:            released_buyback_burn: 0,
programs/treasury-router/src/engines/release.rs:89:        ReleaseBucket::BuybackBurn => (
programs/treasury-router/src/engines/release.rs:90:            treasury.pending_buyback_burn,
programs/treasury-router/src/engines/release.rs:91:            treasury.released_buyback_burn,
programs/treasury-router/src/engines/release.rs:110:        ReleaseBucket::BuybackBurn => {
programs/treasury-router/src/engines/release.rs:111:            treasury.pending_buyback_burn = pending;
programs/treasury-router/src/engines/release.rs:112:            treasury.released_buyback_burn = released;
programs/treasury-router/src/engines/release.rs:132:        ReleaseBucket::BuybackBurn => treasury.buyback_accounting_is_valid(),
programs/treasury-router/src/engines/release.rs:155:            pending_buyback_burn: 250,
programs/treasury-router/src/engines/release.rs:161:            lifetime_buyback_burn: 300,
programs/treasury-router/src/engines/release.rs:167:            released_buyback_burn: 50,
programs/treasury-router/src/engines/release.rs:186:            treasury.pending_buyback_burn,
programs/treasury-router/src/engines/release.rs:187:            treasury.released_buyback_burn,
programs/treasury-router/src/engines/release.rs:208:        if bucket != ReleaseBucket::BuybackBurn {
programs/treasury-router/src/engines/release.rs:211:                    treasury.pending_buyback_burn,
programs/treasury-router/src/engines/release.rs:212:                    treasury.released_buyback_burn
programs/treasury-router/src/engines/release.rs:247:        assert_release(ReleaseBucket::BuybackBurn, 225, 75);
programs/treasury-router/src/engines/release.rs:343:        let amount = treasury.pending_buyback_burn;
programs/treasury-router/src/engines/release.rs:346:            process_release(&mut treasury, ReleaseBucket::BuybackBurn, amount).unwrap();
programs/treasury-router/src/engines/release.rs:351:            treasury.lifetime_buyback_burn
programs/treasury-router/src/engines/waterfall.rs:83:        .checked_add(treasury.pending_buyback_burn)
programs/treasury-router/src/engines/waterfall.rs:175:    pub buyback_burn_bps: u16,
programs/treasury-router/src/engines/waterfall.rs:184:            + self.buyback_burn_bps as u32
programs/treasury-router/src/engines/waterfall.rs:202:            buyback_burn_bps: 2_000,
programs/treasury-router/src/engines/waterfall.rs:209:            buyback_burn_bps: 1_500,
programs/treasury-router/src/engines/waterfall.rs:216:            buyback_burn_bps: 1_000,
programs/treasury-router/src/engines/waterfall.rs:223:            buyback_burn_bps: 0,
programs/treasury-router/src/engines/waterfall.rs:230:            buyback_burn_bps: 0,
programs/treasury-router/src/engines/waterfall.rs:260:        assert_eq!(allocation.buyback_burn_bps, 2_000);
programs/treasury-router/src/engines/waterfall.rs:289:            assert_eq!(allocation.buyback_burn_bps, 0);
programs/treasury-router/src/engines/execution_guard.rs:17:    BuybackBurn,
programs/treasury-router/src/engines/execution_guard.rs:60:    if bucket == ReleaseBucket::BuybackBurn {
programs/treasury-router/src/engines/execution_guard.rs:135:        ReleaseBucket::BuybackBurn => treasury.pending_buyback_burn,
programs/treasury-router/src/engines/execution_guard.rs:212:            pending_buyback_burn: 100,
programs/treasury-router/src/engines/execution_guard.rs:218:            lifetime_buyback_burn: 100,
programs/treasury-router/src/engines/execution_guard.rs:224:            released_buyback_burn: 0,
programs/treasury-router/src/engines/execution_guard.rs:332:            authorize_release(&protocol, &treasury, ReleaseBucket::BuybackBurn, 100).unwrap();
programs/treasury-router/src/engines/execution_guard.rs:334:        assert_eq!(authorization.bucket, ReleaseBucket::BuybackBurn);
programs/treasury-router/src/engines/execution_guard.rs:372:            authorize_release(&protocol, &treasury, ReleaseBucket::BuybackBurn, 1),
programs/treasury-router/src/engines/execution_guard.rs:408:        treasury.pending_buyback_burn = 240;
programs/treasury-router/src/engines/execution_guard.rs:414:        treasury.lifetime_buyback_burn = 240;
programs/treasury-router/src/engines/execution_guard.rs:436:        treasury.pending_buyback_burn = 0;
programs/treasury-router/src/engines/execution_guard.rs:442:        treasury.lifetime_buyback_burn = 0;
programs/treasury-router/src/engines/execution_guard.rs:467:        treasury.pending_buyback_burn = 0;
programs/treasury-router/src/engines/execution_guard.rs:473:        treasury.lifetime_buyback_burn = 0;
programs/treasury-router/src/engines/execution_guard.rs:553:        assert_eq!(pending_balance(&treasury, ReleaseBucket::BuybackBurn), 100);
programs/treasury-router/src/engines/beaver_score.rs:181:            pending_buyback_burn: 200,
programs/treasury-router/src/engines/beaver_score.rs:186:            lifetime_buyback_burn: 200,
programs/treasury-router/src/engines/beaver_score.rs:191:            released_buyback_burn: 0,
programs/treasury-router/src/engines/beaver_score.rs:281:            pending_buyback_burn: 200,
programs/treasury-router/src/engines/beaver_score.rs:287:            lifetime_buyback_burn: 200,
programs/treasury-router/src/engines/beaver_score.rs:293:            released_buyback_burn: 0,
programs/treasury-router/src/engines/buyback.rs:7:    /// Amount credited to the pending buyback-and-burn bucket.
programs/treasury-router/src/engines/buyback.rs:10:    /// Updated pending buyback-and-burn balance.
programs/treasury-router/src/engines/buyback.rs:13:    /// Updated lifetime buyback-and-burn allocation.
programs/treasury-router/src/engines/buyback.rs:17:/// Credits a buyback-and-burn allocation to TreasuryState.
programs/treasury-router/src/engines/buyback.rs:20:/// burn execution, slippage controls, and DEX interaction will be handled
programs/treasury-router/src/engines/buyback.rs:23:    treasury.pending_buyback_burn = treasury
programs/treasury-router/src/engines/buyback.rs:24:        .pending_buyback_burn
programs/treasury-router/src/engines/buyback.rs:28:    treasury.lifetime_buyback_burn = treasury
programs/treasury-router/src/engines/buyback.rs:29:        .lifetime_buyback_burn
programs/treasury-router/src/engines/buyback.rs:35:        pending_balance: treasury.pending_buyback_burn,
programs/treasury-router/src/engines/buyback.rs:36:        lifetime_total: treasury.lifetime_buyback_burn,
programs/treasury-router/src/events/mod.rs:28:/// swap or RBVR burn has already occurred.
programs/treasury-router/src/state/execution_config.rs:19:    /// Permanent destination for Automatic Buyback & Burn funding.
programs/treasury-router/src/state/treasury.rs:25:    pub pending_buyback_burn: u64,
programs/treasury-router/src/state/treasury.rs:32:    pub lifetime_buyback_burn: u64,
programs/treasury-router/src/state/treasury.rs:39:    pub released_buyback_burn: u64,
programs/treasury-router/src/state/treasury.rs:100:            self.pending_buyback_burn,
programs/treasury-router/src/state/treasury.rs:101:            self.released_buyback_burn,
programs/treasury-router/src/state/treasury.rs:102:            self.lifetime_buyback_burn,
programs/treasury-router/src/state/treasury.rs:154:            pending_buyback_burn: 200,
programs/treasury-router/src/state/treasury.rs:160:            lifetime_buyback_burn: 250,
programs/treasury-router/src/state/treasury.rs:166:            released_buyback_burn: 50,
programs/treasury-router/src/state/treasury.rs:218:        treasury.lifetime_buyback_burn = 249;
programs/treasury-router/src/state/treasury.rs:227:        treasury.pending_buyback_burn = u64::MAX;
programs/treasury-router/src/state/treasury.rs:228:        treasury.released_buyback_burn = 1;
programs/treasury-router/src/state/treasury.rs:229:        treasury.lifetime_buyback_burn = 0;
programs/treasury-router/src/state/treasury.rs:335:        treasury.lifetime_buyback_burn += 1;
programs/treasury-router/src/state/treasury.rs:386:        treasury.pending_buyback_burn = u64::MAX;
programs/treasury-router/src/state/treasury.rs:387:        treasury.released_buyback_burn = u64::MAX;
programs/treasury-router/src/state/treasury.rs:388:        treasury.lifetime_buyback_burn = u64::MAX;
programs/treasury-router/src/state/treasury.rs:415:        treasury.pending_buyback_burn = 0;
programs/treasury-router/src/state/treasury.rs:421:        treasury.lifetime_buyback_burn = 0;
programs/treasury-router/src/state/treasury.rs:427:        treasury.released_buyback_burn = 0;
programs/treasury-router/src/state/protocol_config.rs:13:    pub buyback_burn_bps: u16,
programs/treasury-router/src/state/protocol_config.rs:43:            .saturating_add(self.buyback_burn_bps)
programs/treasury-router/src/instructions/deposit_settlement.rs:84:    let cpi_context = CpiContext::new(ctx.accounts.token_program.key(), transfer_accounts);
programs/treasury-router/src/instructions/deposit_settlement.rs:86:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/process_fees.rs:115:        .checked_add(ctx.accounts.treasury.released_buyback_burn)
programs/treasury-router/src/instructions/process_fees.rs:137:        .checked_add(u32::from(config.buyback_burn_bps))
programs/treasury-router/src/instructions/process_fees.rs:209:        adaptive_allocation.buyback_burn_bps,
programs/treasury-router/src/instructions/process_fees.rs:238:        "Buyback-and-burn allocation deposited: {}",
programs/treasury-router/src/instructions/process_fees.rs:243:        "Pending buyback-and-burn balance: {}",
programs/treasury-router/src/instructions/process_fees.rs:248:        "Lifetime buyback-and-burn allocation: {}",
programs/treasury-router/src/instructions/process_fees.rs:416:    let buyback_amount = calculate_share(amount, adaptive_allocation.buyback_burn_bps)?;
programs/treasury-router/src/instructions/reserve.rs:274:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/reserve.rs:280:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/founder.rs:134:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/founder.rs:140:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/company.rs:134:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/company.rs:140:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/initialize_treasury.rs:81:    treasury_state.pending_buyback_burn = 0;
programs/treasury-router/src/instructions/initialize_treasury.rs:87:    treasury_state.lifetime_buyback_burn = 0;
programs/treasury-router/src/instructions/initialize_treasury.rs:93:    treasury_state.released_buyback_burn = 0;
programs/treasury-router/src/instructions/initialize_protocol_config.rs:5:        BPS_DENOMINATOR, INITIAL_BUYBACK_BURN_BPS, INITIAL_COMPANY_BPS, INITIAL_FOUNDER_BPS,
programs/treasury-router/src/instructions/initialize_protocol_config.rs:53:        .checked_add(INITIAL_BUYBACK_BURN_BPS)
programs/treasury-router/src/instructions/initialize_protocol_config.rs:72:    protocol_config.buyback_burn_bps = INITIAL_BUYBACK_BURN_BPS;
programs/treasury-router/src/instructions/initialize_protocol_config.rs:88:        protocol_config.buyback_burn_bps,
programs/treasury-router/src/instructions/liquidity.rs:271:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/liquidity.rs:277:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/buyback.rs:183:/// Releases previously allocated buyback-and-burn funds.
programs/treasury-router/src/instructions/buyback.rs:188:/// - decreases pending buyback-and-burn accounting;
programs/treasury-router/src/instructions/buyback.rs:189:/// - increases released buyback-and-burn accounting.
programs/treasury-router/src/instructions/buyback.rs:191:/// It does not allocate fees, execute a DEX swap, purchase RBVR, or burn tokens.
programs/treasury-router/src/instructions/buyback.rs:256:        ReleaseBucket::BuybackBurn,
programs/treasury-router/src/instructions/buyback.rs:274:    let cpi_context = CpiContext::new_with_signer(
programs/treasury-router/src/instructions/buyback.rs:280:    token::transfer_checked(cpi_context, amount, ctx.accounts.settlement_mint.decimals)?;
programs/treasury-router/src/instructions/buyback.rs:286:        ReleaseBucket::BuybackBurn,
programs/treasury-router/src/instructions/buyback.rs:357:    msg!("No DEX swap, RBVR purchase, or token burn CPI was performed");
programs/treasury-router/src/constants.rs:38:pub const INITIAL_BUYBACK_BURN_BPS: u16 = 2_000;
```
- ⚠️ **REVIEW:** CPI or token-authority operations exist and require manual account-program, mint, owner, signer-seed, and destination review.

## Initialization and reinitialization inventory

```text
programs/treasury-router/src/engines/integrity_firewall.rs:25:    ProtocolVersion = 3,
programs/treasury-router/src/engines/integrity_firewall.rs:30:    ProtocolConfigVersion = 7,
programs/treasury-router/src/engines/integrity_firewall.rs:35:    TreasuryVersion = 11,
programs/treasury-router/src/engines/integrity_firewall.rs:45:    FounderVersion = 19,
programs/treasury-router/src/engines/integrity_firewall.rs:50:    CompanyVersion = 23,
programs/treasury-router/src/engines/integrity_firewall.rs:55:    ExecutionConfigVersion = 27,
programs/treasury-router/src/engines/integrity_firewall.rs:460:            initialized_at: 1,
programs/treasury-router/src/engines/execution_guard.rs:182:            initialized_at: 1,
programs/treasury-router/src/state/protocol.rs:32:    pub initialized_at: i64,
programs/treasury-router/src/state/protocol.rs:46:        8 +       // initialized_at
programs/treasury-router/src/instructions/initialize_company.rs:69:    company_state.version = COMPANY_STATE_VERSION;
programs/treasury-router/src/instructions/initialize_founder.rs:69:    founder_state.version = FOUNDER_STATE_VERSION;
programs/treasury-router/src/instructions/initialize_execution_config.rs:234:    execution_config.version = EXECUTION_CONFIG_VERSION;
programs/treasury-router/src/instructions/initialize.rs:29:    protocol_state.version = PROTOCOL_VERSION;
programs/treasury-router/src/instructions/initialize.rs:44:    protocol_state.initialized_at = clock.unix_timestamp;
programs/treasury-router/src/instructions/initialize_treasury.rs:72:    treasury_state.version = TREASURY_VERSION;
programs/treasury-router/src/instructions/initialize_protocol_config.rs:68:    protocol_config.version = PROTOCOL_CONFIG_VERSION;
programs/treasury-router/src/lib.rs:19:    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
programs/treasury-router/src/lib.rs:23:    pub fn initialize_protocol_config(ctx: Context<InitializeProtocolConfig>) -> Result<()> {
programs/treasury-router/src/lib.rs:27:    pub fn initialize_founder(
programs/treasury-router/src/lib.rs:36:    pub fn initialize_company(
programs/treasury-router/src/lib.rs:45:    pub fn initialize_treasury(ctx: Context<InitializeTreasury>) -> Result<()> {
programs/treasury-router/src/lib.rs:49:    pub fn initialize_execution_config(ctx: Context<InitializeExecutionConfig>) -> Result<()> {
```

## Pause, configuration and upgrade controls

```text
programs/treasury-router/src/engines/sentinel.rs:525:            buybacks_paused: false,
programs/treasury-router/src/engines/sentinel.rs:573:    fn configuration_updates_enabled_fails_closed() {
programs/treasury-router/src/engines/sentinel.rs:633:    fn lifetime_sum_overflow_fails_closed() {
programs/treasury-router/src/engines/integrity_firewall.rs:458:            paused: false,
programs/treasury-router/src/engines/integrity_firewall.rs:508:            buybacks_paused: false,
programs/treasury-router/src/engines/integrity_firewall.rs:633:    fn wrong_program_id_fails_closed() {
programs/treasury-router/src/engines/dam.rs:10:    /// The Dam is closed while protocol reserves are rebuilt.
programs/treasury-router/src/engines/dam.rs:160:    fn emergency_waterfall_closes_dam() {
programs/treasury-router/src/engines/dam.rs:197:    fn critically_low_health_closes_the_dam() {
programs/treasury-router/src/engines/dam.rs:205:    fn emergency_waterfall_always_closes_the_dam() {
programs/treasury-router/src/engines/release.rs:175:            buybacks_paused: false,
programs/treasury-router/src/engines/release.rs:331:        // overflows, so the transition must fail closed before mutation.
programs/treasury-router/src/engines/execution_guard.rs:46:    require!(!protocol.paused, TreasuryRouterError::ProtocolPaused);
programs/treasury-router/src/engines/execution_guard.rs:62:            !treasury.buybacks_paused,
programs/treasury-router/src/engines/execution_guard.rs:63:            TreasuryRouterError::BuybacksPaused
programs/treasury-router/src/engines/execution_guard.rs:84:        TreasuryRouterError::DamClosed
programs/treasury-router/src/engines/execution_guard.rs:180:            paused: false,
programs/treasury-router/src/engines/execution_guard.rs:232:            buybacks_paused: false,
programs/treasury-router/src/engines/execution_guard.rs:341:    fn rejects_release_when_protocol_is_paused() {
programs/treasury-router/src/engines/execution_guard.rs:345:        protocol.paused = true;
programs/treasury-router/src/engines/execution_guard.rs:349:            "ProtocolPaused",
programs/treasury-router/src/engines/execution_guard.rs:365:    fn rejects_buyback_when_buybacks_are_paused() {
programs/treasury-router/src/engines/execution_guard.rs:369:        treasury.buybacks_paused = true;
programs/treasury-router/src/engines/execution_guard.rs:373:            "BuybacksPaused",
programs/treasury-router/src/engines/execution_guard.rs:382:        treasury.buybacks_paused = true;
programs/treasury-router/src/engines/execution_guard.rs:403:    fn rejects_release_when_dam_is_closed() {
programs/treasury-router/src/engines/execution_guard.rs:424:            "DamClosed",
programs/treasury-router/src/engines/beaver_score.rs:198:            buybacks_paused: false,
programs/treasury-router/src/engines/beaver_score.rs:301:            buybacks_paused: false,
programs/treasury-router/src/state/protocol.rs:26:    pub paused: bool,
programs/treasury-router/src/state/protocol.rs:44:        1 +       // paused
programs/treasury-router/src/state/treasury.rs:53:    /// Buyback execution can be paused independently.
programs/treasury-router/src/state/treasury.rs:54:    pub buybacks_paused: bool,
programs/treasury-router/src/state/treasury.rs:71:        1 +        // buybacks_paused
programs/treasury-router/src/state/treasury.rs:75:    /// Validates one accounting bucket using fail-closed checked arithmetic.
programs/treasury-router/src/state/treasury.rs:174:            buybacks_paused: false,
programs/treasury-router/src/state/treasury.rs:197:    fn reserve_accounting_fails_closed_on_overflow() {
programs/treasury-router/src/state/treasury.rs:224:    fn buyback_accounting_fails_closed_on_overflow() {
programs/treasury-router/src/state/treasury.rs:251:    fn liquidity_accounting_fails_closed_on_overflow() {
programs/treasury-router/src/state/treasury.rs:278:    fn company_accounting_fails_closed_on_overflow() {
programs/treasury-router/src/state/treasury.rs:305:    fn founder_accounting_fails_closed_on_overflow() {
programs/treasury-router/src/state/treasury.rs:379:    fn overflow_with_max_lifetime_fails_closed_for_every_bucket() {
programs/treasury-router/src/instructions/deposit_settlement.rs:71:        !ctx.accounts.protocol_state.paused,
programs/treasury-router/src/instructions/deposit_settlement.rs:72:        TreasuryRouterError::ProtocolPaused
programs/treasury-router/src/instructions/initialize_company.rs:44:        !ctx.accounts.protocol_state.paused,
programs/treasury-router/src/instructions/initialize_company.rs:45:        TreasuryRouterError::ProtocolPaused
programs/treasury-router/src/instructions/process_fees.rs:85:        !ctx.accounts.protocol_state.paused,
programs/treasury-router/src/instructions/process_fees.rs:86:        TreasuryRouterError::ProtocolPaused
programs/treasury-router/src/instructions/process_fees.rs:89:    // Fail closed before any accounting mutation. Sentinel V2 verifies the
programs/treasury-router/src/instructions/initialize_founder.rs:44:        !ctx.accounts.protocol_state.paused,
programs/treasury-router/src/instructions/initialize_founder.rs:45:        TreasuryRouterError::ProtocolPaused
programs/treasury-router/src/instructions/initialize_execution_config.rs:19:        constraint = !protocol_state.paused
programs/treasury-router/src/instructions/initialize_execution_config.rs:20:            @ TreasuryRouterError::ProtocolPaused,
programs/treasury-router/src/instructions/initialize.rs:42:    protocol_state.paused = false;
programs/treasury-router/src/instructions/initialize_treasury.rs:61:        !ctx.accounts.protocol_state.paused,
programs/treasury-router/src/instructions/initialize_treasury.rs:62:        TreasuryRouterError::ProtocolPaused
programs/treasury-router/src/instructions/initialize_treasury.rs:101:    treasury_state.buybacks_paused = false;
programs/treasury-router/src/instructions/initialize_protocol_config.rs:43:        !ctx.accounts.protocol_state.paused,
programs/treasury-router/src/instructions/initialize_protocol_config.rs:44:        TreasuryRouterError::ProtocolPaused
programs/treasury-router/src/errors/mod.rs:8:    #[msg("The protocol is currently paused.")]
programs/treasury-router/src/errors/mod.rs:9:    ProtocolPaused,
programs/treasury-router/src/errors/mod.rs:62:    #[msg("Buyback execution is currently paused.")]
programs/treasury-router/src/errors/mod.rs:63:    BuybacksPaused,
programs/treasury-router/src/errors/mod.rs:65:    #[msg("The Dam is closed and no treasury release is permitted.")]
programs/treasury-router/src/errors/mod.rs:66:    DamClosed,
```

## Protocol authority architecture

- ⚠️ **REVIEW:** Protocol authority fields or assignments remain present. Confirm whether they are initialization-only, whether any privileged instruction consumes them, and how final authority revocation is enforced.

```text
programs/treasury-router/src/engines/integrity_firewall.rs:26:    ProtocolAuthority = 4,
programs/treasury-router/src/engines/integrity_firewall.rs:167:    if protocol.authority == Pubkey::default() {
programs/treasury-router/src/instructions/initialize.rs:30:    protocol_state.authority = ctx.accounts.authority.key();
programs/treasury-router/src/instructions/initialize_treasury.rs:48:        token::authority = treasury_state
```

## Release instruction consistency

- ✅ **PASS:** reserve delegates accounting mutation to the centralized process_release engine.
- ✅ **PASS:** reserve contains a token transfer operation.
- ✅ **PASS:** liquidity delegates accounting mutation to the centralized process_release engine.
- ✅ **PASS:** liquidity contains a token transfer operation.
- ✅ **PASS:** company delegates accounting mutation to the centralized process_release engine.
- ✅ **PASS:** company contains a token transfer operation.
- ✅ **PASS:** founder delegates accounting mutation to the centralized process_release engine.
- ✅ **PASS:** founder contains a token transfer operation.
- ✅ **PASS:** buyback delegates accounting mutation to the centralized process_release engine.
- ✅ **PASS:** buyback contains a token transfer operation.

## Dependency vulnerability checks

- ⚠️ **REVIEW:** cargo-audit is not installed. Install with: cargo install cargo-audit --locked
- ⚠️ **REVIEW:** npm reported production dependency vulnerabilities or audit errors.

```text
# npm audit report

bigint-buffer  *
Severity: high
bigint-buffer Vulnerable to Buffer Overflow via toBigIntLE() Function - https://github.com/advisories/GHSA-3gc7-fjrx-p6mg
No fix available
node_modules/bigint-buffer
  @solana/buffer-layout-utils  *
  Depends on vulnerable versions of @solana/web3.js
  Depends on vulnerable versions of bigint-buffer
  node_modules/@solana/buffer-layout-utils
    @solana/spl-token  *
    Depends on vulnerable versions of @solana/buffer-layout-utils
    Depends on vulnerable versions of @solana/spl-token-group
    Depends on vulnerable versions of @solana/spl-token-metadata
    Depends on vulnerable versions of @solana/web3.js
    node_modules/@solana/spl-token
      @meteora-ag/dynamic-bonding-curve-sdk  *
      Depends on vulnerable versions of @coral-xyz/anchor
      Depends on vulnerable versions of @solana/spl-token
      Depends on vulnerable versions of @solana/web3.js
      node_modules/@meteora-ag/dynamic-bonding-curve-sdk

uuid  <11.1.1
Severity: moderate
uuid: Missing buffer bounds check in v3/v5/v6 when buf is provided - https://github.com/advisories/GHSA-w5hq-g745-h8pq
No fix available
node_modules/uuid
  jayson  >=2.0.6
  Depends on vulnerable versions of uuid
  node_modules/jayson
    @solana/web3.js  <=0.0.0-pr-29130 || 0.0.4 - 1.98.4
    Depends on vulnerable versions of jayson
    node_modules/@solana/web3.js
      @coral-xyz/anchor  *
      Depends on vulnerable versions of @coral-xyz/borsh
      Depends on vulnerable versions of @solana/web3.js
      node_modules/@meteora-ag/dynamic-bonding-curve-sdk/node_modules/@coral-xyz/anchor
      @coral-xyz/borsh  *
      Depends on vulnerable versions of @solana/web3.js
      node_modules/@coral-xyz/borsh
      @solana/spl-token-group  *
      Depends on vulnerable versions of @solana/web3.js
      node_modules/@solana/spl-token-group
      @solana/spl-token-metadata  *
      Depends on vulnerable versions of @solana/web3.js
      node_modules/@solana/spl-token-metadata

11 vulnerabilities (7 moderate, 4 high)

To address issues that do not require attention, run:
  npm audit fix

Some issues need review, and may require choosing
a different dependency.
```

## Secret and private-key filename scan

- ⚠️ **REVIEW:** Potentially sensitive filenames were detected. Confirm none contain deployer, treasury, mint, upgrade-authority, or wallet secret bytes.

```text
./scripts/devnet/create-devnet-wallet.mjs
./.localnet/devnet-v10/rbvr-mint-keypair.json
./.localnet/devnet-v8/payer.json
./.localnet/devnet-v8/fee-claimer.json
./.localnet/devnet-v8/leftover-receiver.json
./.localnet/devnet-v9/rbvr-mint-keypair.json
./config/meteora/dbc-model-v1.json
./.anchor/test-ledger/stake-account-keypair.json
./.anchor/test-ledger/validator-keypair.json
./.anchor/test-ledger/vote-account-keypair.json
./.anchor/test-ledger/faucet-keypair.json
./test-ledger/stake-account-keypair.json
./test-ledger/validator-keypair.json
./test-ledger/vote-account-keypair.json
./test-ledger/faucet-keypair.json
```

## Final automated assessment

- **Passes:** 16
- **Manual reviews/warnings:** 8
- **Failures:** 1

**AUTOMATED RESULT: FAIL — DO NOT DEPLOY**
