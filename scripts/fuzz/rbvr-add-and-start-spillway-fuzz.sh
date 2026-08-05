#!/usr/bin/env bash
set -Eeuo pipefail

cd ~/rocket-beaver-contract

TARGET="spillway_invariants"
FUZZ_FILE="fuzz/fuzz_targets/${TARGET}.rs"
FUZZ_MANIFEST="fuzz/Cargo.toml"
EXPECTED_COMMIT_PREFIX="21cc0a4"
TAG="rbvr-spillway-fuzz-v1"

RUN_SECONDS=28800
RUN_DIR=".rbvr-fuzz-runs"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="$RUN_DIR/${TARGET}-${TIMESTAMP}.log"
PID_FILE="$RUN_DIR/${TARGET}.pid"

echo "============================================================"
echo "RBVR SPILLWAY INVARIANTS FUZZ TARGET"
echo "============================================================"

if [[ "$(git branch --show-current)" != "red-team" ]]; then
    echo "ERROR: Expected branch red-team."
    echo "Current branch: $(git branch --show-current)"
    exit 1
fi

CURRENT_COMMIT="$(git rev-parse HEAD)"

if [[ "$CURRENT_COMMIT" != "$EXPECTED_COMMIT_PREFIX"* ]]; then
    echo "ERROR: Expected RT-005 commit beginning with $EXPECTED_COMMIT_PREFIX."
    echo "Current commit: $CURRENT_COMMIT"
    exit 1
fi

if [[ ! -f "$FUZZ_MANIFEST" ]]; then
    echo "ERROR: Missing $FUZZ_MANIFEST"
    exit 1
fi

if [[ -e "$FUZZ_FILE" ]]; then
    echo "ERROR: $FUZZ_FILE already exists."
    exit 1
fi

if pgrep -af "cargo.*fuzz run ${TARGET}|/${TARGET} " >/dev/null; then
    echo "ERROR: A $TARGET fuzz campaign already appears to be running."
    pgrep -af "cargo.*fuzz run ${TARGET}|/${TARGET} " || true
    exit 1
fi

BACKUP_DIR=".spillway-fuzz-backup-$TIMESTAMP"
mkdir -p "$BACKUP_DIR/fuzz"
cp "$FUZZ_MANIFEST" "$BACKUP_DIR/fuzz/Cargo.toml"

rollback() {
    code=$?

    if [[ $code -ne 0 ]]; then
        echo
        echo "============================================================"
        echo "SPILLWAY FUZZ TARGET SETUP FAILED"
        echo "============================================================"

        cp "$BACKUP_DIR/fuzz/Cargo.toml" "$FUZZ_MANIFEST"
        rm -f "$FUZZ_FILE"

        echo "Original fuzz manifest restored."
    fi

    exit "$code"
}

trap rollback EXIT

echo
echo "===== CREATE SPILLWAY FUZZ TARGET ====="

cat > "$FUZZ_FILE" <<'RS'
#![no_main]

use anchor_lang::prelude::Pubkey;
use libfuzzer_sys::fuzz_target;
use treasury_router::{
    engines::reserve_deployment::{
        evaluate_reserve_deployment, ReserveDeploymentPolicy,
        ReserveDeploymentStage,
    },
    state::ReservePolicy,
};

struct Cursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn u8(&mut self) -> u8 {
        let value = self.data.get(self.offset).copied().unwrap_or(0);
        self.offset = self.offset.saturating_add(1);
        value
    }

    fn u16(&mut self) -> u16 {
        let mut bytes = [0u8; 2];

        for byte in &mut bytes {
            *byte = self.u8();
        }

        u16::from_le_bytes(bytes)
    }

    fn u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];

        for byte in &mut bytes {
            *byte = self.u8();
        }

        u64::from_le_bytes(bytes)
    }

    fn i64(&mut self) -> i64 {
        self.u64() as i64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PolicySnapshot {
    minimum_reserve_floor: u64,
    liquidity_floor_bps: u16,
    surplus_deployment_bps: u16,
    cooldown_seconds: i64,
    last_deployed_at: i64,
    lifetime_deployed: u64,
}

fn snapshot(policy: &ReservePolicy) -> PolicySnapshot {
    PolicySnapshot {
        minimum_reserve_floor: policy.minimum_reserve_floor,
        liquidity_floor_bps: policy.liquidity_floor_bps,
        surplus_deployment_bps: policy.surplus_deployment_bps,
        cooldown_seconds: policy.cooldown_seconds,
        last_deployed_at: policy.last_deployed_at,
        lifetime_deployed: policy.lifetime_deployed,
    }
}

fn make_policy(
    minimum_reserve_floor: u64,
    liquidity_floor_bps: u16,
    surplus_deployment_bps: u16,
    cooldown_seconds: i64,
    last_deployed_at: i64,
    lifetime_deployed: u64,
) -> ReservePolicy {
    ReservePolicy {
        version: 1,
        protocol: Pubkey::new_from_array([1u8; 32]),
        treasury: Pubkey::new_from_array([2u8; 32]),

        minimum_reserve_floor,
        liquidity_floor_bps,
        surplus_deployment_bps,
        cooldown_seconds,

        last_deployed_at,
        lifetime_deployed,

        bump: 255,
        reserved: [0; 31],
    }
}

fuzz_target!(|data: &[u8]| {
    let mut cursor = Cursor::new(data);

    let reserve_balance = cursor.u64();
    let liquidity_reference = cursor.u64();

    let minimum_reserve_floor = cursor.u64();

    /*
     * Preserve invalid values too. Rates above 10,000 must fail closed rather
     * than being silently normalized by the harness.
     */
    let liquidity_floor_bps = cursor.u16();
    let surplus_deployment_bps = cursor.u16();

    let cooldown_seconds = cursor.i64();
    let last_deployed_at = cursor.i64();
    let current_timestamp = cursor.i64();
    let lifetime_deployed = cursor.u64();

    let parameters_valid = ReservePolicy::validate_parameters(
        minimum_reserve_floor,
        liquidity_floor_bps,
        surplus_deployment_bps,
        cooldown_seconds,
    )
    .is_ok();

    let evaluation = evaluate_reserve_deployment(
        reserve_balance,
        liquidity_reference,
        ReserveDeploymentPolicy {
            minimum_reserve_floor,
            liquidity_floor_bps,
            surplus_deployment_bps,
        },
    );

    match evaluation {
        Ok(result) => {
            /*
             * A successful engine evaluation can never authorize more than
             * the current gross surplus.
             */
            assert!(
                result.deployable_amount <= result.gross_surplus,
                "deployable amount exceeded gross surplus"
            );

            /*
             * Accounting conservation:
             *
             * reserve before == deployed amount + reserve after
             */
            assert_eq!(
                result
                    .deployable_amount
                    .checked_add(result.remaining_reserve),
                Some(reserve_balance),
                "reserve evaluation did not conserve value"
            );

            match result.stage {
                ReserveDeploymentStage::Filling => {
                    assert!(
                        reserve_balance < result.reserve_floor,
                        "Filling stage used at or above the floor"
                    );

                    assert_eq!(
                        result.deployable_amount, 0,
                        "Filling reserve authorized deployment"
                    );

                    assert_eq!(
                        result.remaining_reserve, reserve_balance,
                        "Filling evaluation changed the reserve"
                    );
                }

                ReserveDeploymentStage::Healthy => {
                    assert_eq!(
                        reserve_balance, result.reserve_floor,
                        "Healthy stage did not equal the floor"
                    );

                    assert_eq!(
                        result.deployable_amount, 0,
                        "Healthy reserve authorized deployment"
                    );

                    assert_eq!(
                        result.remaining_reserve, reserve_balance,
                        "Healthy evaluation changed the reserve"
                    );
                }

                ReserveDeploymentStage::Surplus => {
                    assert!(
                        reserve_balance > result.reserve_floor,
                        "Surplus stage used without surplus"
                    );

                    assert_eq!(
                        result.gross_surplus,
                        reserve_balance - result.reserve_floor,
                        "gross surplus was calculated incorrectly"
                    );

                    assert!(
                        result.remaining_reserve >= result.reserve_floor,
                        "successful deployment breached the protected floor"
                    );
                }
            }

            /*
             * Valid policy rates must be compatible with a successful engine
             * evaluation. Invalid policy parameters may still include an
             * engine-valid zero floor; they must never be recorded on-chain.
             */
            if parameters_valid {
                let mut policy = make_policy(
                    minimum_reserve_floor,
                    liquidity_floor_bps,
                    surplus_deployment_bps,
                    cooldown_seconds,
                    last_deployed_at,
                    lifetime_deployed,
                );

                let before = snapshot(&policy);

                if result.deployable_amount == 0 {
                    /*
                     * The live Spillway rejects zero deployable surplus before
                     * policy mutation. Reproduce that fail-closed behavior.
                     */
                    assert_eq!(
                        snapshot(&policy),
                        before,
                        "zero-surplus path mutated Reserve Policy"
                    );

                    return;
                }

                let cooldown_result =
                    policy.cooldown_has_elapsed(current_timestamp);

                match cooldown_result {
                    Ok(true) => {
                        let expected_lifetime = lifetime_deployed
                            .checked_add(result.deployable_amount);

                        let record_result = policy.record_deployment(
                            result.deployable_amount,
                            current_timestamp,
                        );

                        match expected_lifetime {
                            Some(expected) => {
                                assert!(
                                    record_result.is_ok(),
                                    "eligible deployment unexpectedly failed"
                                );

                                assert_eq!(
                                    policy.lifetime_deployed, expected,
                                    "lifetime deployment accounting mismatch"
                                );

                                assert_eq!(
                                    policy.last_deployed_at,
                                    current_timestamp,
                                    "deployment timestamp was not recorded"
                                );

                                /*
                                 * Immutable policy parameters must never be
                                 * modified by a successful transition.
                                 */
                                assert_eq!(
                                    policy.minimum_reserve_floor,
                                    before.minimum_reserve_floor
                                );
                                assert_eq!(
                                    policy.liquidity_floor_bps,
                                    before.liquidity_floor_bps
                                );
                                assert_eq!(
                                    policy.surplus_deployment_bps,
                                    before.surplus_deployment_bps
                                );
                                assert_eq!(
                                    policy.cooldown_seconds,
                                    before.cooldown_seconds
                                );
                            }

                            None => {
                                assert!(
                                    record_result.is_err(),
                                    "lifetime overflow was accepted"
                                );

                                assert_eq!(
                                    snapshot(&policy),
                                    before,
                                    "overflow failure mutated Reserve Policy"
                                );
                            }
                        }
                    }

                    Ok(false) | Err(_) => {
                        let record_result = policy.record_deployment(
                            result.deployable_amount,
                            current_timestamp,
                        );

                        assert!(
                            record_result.is_err(),
                            "cooldown-protected deployment was accepted"
                        );

                        assert_eq!(
                            snapshot(&policy),
                            before,
                            "cooldown failure mutated Reserve Policy"
                        );
                    }
                }
            }
        }

        Err(_) => {
            /*
             * Invalid rates or impossible arithmetic must fail closed. There
             * is no evaluation result and therefore no deployable amount.
             */
        }
    }
});
RS

echo "Created: $FUZZ_FILE"

echo
echo "===== REGISTER TARGET IN FUZZ MANIFEST ====="

python3 <<'PY'
from pathlib import Path
import re

path = Path("fuzz/Cargo.toml")
text = path.read_text()

target = "spillway_invariants"

if re.search(
    rf'(?ms)^\[\[bin\]\]\s*.*?^name\s*=\s*"{re.escape(target)}"\s*$',
    text,
):
    raise RuntimeError(f"Fuzz target already registered: {target}")

entry = f'''
[[bin]]
name = "{target}"
path = "fuzz_targets/{target}.rs"
test = false
doc = false
bench = false
'''

path.write_text(text.rstrip() + "\n" + entry)

print(f"Registered fuzz target: {target}")
PY

echo
echo "===== VERIFY TARGET REGISTRATION ====="

grep -n -A 6 \
    'name = "spillway_invariants"' \
    "$FUZZ_MANIFEST"

echo
echo "===== FORMAT PROTOCOL ====="
cargo fmt --all

echo
echo "===== VERIFY STANDARD REGRESSION ====="

cargo test --workspace

echo
echo "===== STRICT CLIPPY ====="

cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- \
    -D warnings

echo
echo "===== VERIFY NIGHTLY FUZZ ENVIRONMENT ====="

if ! rustup toolchain list | grep -q '^nightly'; then
    rustup toolchain install nightly
fi

rustup component add \
    llvm-tools-preview \
    --toolchain nightly

if ! cargo +nightly fuzz --version >/dev/null 2>&1; then
    cargo +nightly install cargo-fuzz
fi

cargo +nightly fuzz --version

echo
echo "===== LIST FUZZ TARGETS ====="

cargo +nightly fuzz list

if ! cargo +nightly fuzz list | grep -qx "$TARGET"; then
    echo "ERROR: $TARGET is not listed by cargo-fuzz."
    exit 1
fi

echo
echo "===== COMPILE SPILLWAY FUZZ TARGET ====="

mkdir -p "fuzz/corpus/$TARGET"
mkdir -p "fuzz/artifacts/$TARGET"

cargo +nightly fuzz build "$TARGET"

echo
echo "===== SHORT SMOKE CAMPAIGN ====="

cargo +nightly fuzz run "$TARGET" \
    "fuzz/corpus/$TARGET" \
    -- \
    -runs=10000 \
    -timeout=10 \
    -rss_limit_mb=4096 \
    -max_len=128 \
    -print_final_stats=1

echo
echo "===== VERIFY NO SMOKE-TEST CRASH ====="

if find "fuzz/artifacts/$TARGET" \
    -maxdepth 1 \
    -type f \
    \( \
      -name 'crash-*' -o \
      -name 'timeout-*' -o \
      -name 'oom-*' \
    \) |
    grep -q .
then
    echo "ERROR: Smoke campaign generated a crash artifact."
    find "fuzz/artifacts/$TARGET" -maxdepth 1 -type f -print
    exit 1
fi

echo
echo "===== COMMIT VERIFIED FUZZ TARGET ====="

git add \
    "$FUZZ_MANIFEST" \
    "$FUZZ_FILE"

git diff --cached --check
git diff --cached --stat

if git diff --cached --quiet; then
    echo "ERROR: No fuzz target changes were staged."
    exit 1
fi

git commit -m \
    "security: add spillway invariant fuzz target"

if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Tag already exists: $TAG"
else
    git tag -a "$TAG" \
        -m "Spillway invariant fuzz harness compiled and smoke-tested"
fi

echo
echo "===== START EIGHT-HOUR BACKGROUND CAMPAIGN ====="

mkdir -p "$RUN_DIR"

nohup cargo +nightly fuzz run "$TARGET" \
    "fuzz/corpus/$TARGET" \
    -- \
    -max_total_time="$RUN_SECONDS" \
    -timeout=10 \
    -rss_limit_mb=4096 \
    -max_len=128 \
    -print_final_stats=1 \
    >"$LOG_FILE" 2>&1 &

FUZZ_PID="$!"
printf '%s\n' "$FUZZ_PID" > "$PID_FILE"

sleep 2

if ! kill -0 "$FUZZ_PID" 2>/dev/null; then
    echo "ERROR: Background fuzz process exited immediately."
    echo
    tail -n 100 "$LOG_FILE" || true
    exit 1
fi

echo
echo "===== FINAL STATUS ====="

git log --oneline --decorate -4

echo
echo "============================================================"
echo "RBVR SPILLWAY FUZZING STARTED"
echo "============================================================"
echo "Target:       $TARGET"
echo "PID:          $FUZZ_PID"
echo "Duration:     8 hours"
echo "Log:          $LOG_FILE"
echo "PID file:     $PID_FILE"
echo "Corpus:       fuzz/corpus/$TARGET"
echo "Artifacts:    fuzz/artifacts/$TARGET"
echo "Commit/tag:   $TAG"
echo
echo "Follow live:"
echo "tail -f $LOG_FILE"
echo
echo "Check process:"
echo "ps -p $FUZZ_PID -o pid,etime,%cpu,%mem,cmd"
echo
echo "Check worker:"
echo "pgrep -af '$TARGET'"
echo
echo "Check for failures:"
echo "find fuzz/artifacts/$TARGET -maxdepth 1 -type f -print"
echo
echo "Stop manually:"
echo "kill $FUZZ_PID"
echo "============================================================"

trap - EXIT
