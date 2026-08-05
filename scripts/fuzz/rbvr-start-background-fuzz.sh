#!/usr/bin/env bash
set -euo pipefail

cd ~/rocket-beaver-contract

TARGET="process_release"
SECONDS_TO_RUN=28800
LOG_DIR=".rbvr-fuzz-runs"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="$LOG_DIR/${TARGET}-${TIMESTAMP}.log"
PID_FILE="$LOG_DIR/${TARGET}.pid"

mkdir -p "$LOG_DIR"
mkdir -p "fuzz/corpus/$TARGET"
mkdir -p "fuzz/artifacts/$TARGET"

if ! rustup toolchain list | grep -q '^nightly'; then
    echo "Installing Rust nightly..."
    rustup toolchain install nightly
fi

rustup component add llvm-tools-preview --toolchain nightly >/dev/null

if [[ -f "$PID_FILE" ]]; then
    OLD_PID="$(cat "$PID_FILE" 2>/dev/null || true)"

    if [[ -n "$OLD_PID" ]] && kill -0 "$OLD_PID" 2>/dev/null; then
        echo "A $TARGET fuzz run is already active."
        echo "PID: $OLD_PID"
        echo "Log:"
        ls -1t "$LOG_DIR"/"${TARGET}"-*.log | head -n 1
        exit 0
    fi

    rm -f "$PID_FILE"
fi

echo "Starting $TARGET fuzzing for $SECONDS_TO_RUN seconds..."

nohup cargo +nightly fuzz run "$TARGET" \
    "fuzz/corpus/$TARGET" \
    -- \
    -max_total_time="$SECONDS_TO_RUN" \
    -timeout=10 \
    -rss_limit_mb=4096 \
    -max_len=512 \
    -print_final_stats=1 \
    >"$LOG_FILE" 2>&1 &

FUZZ_PID=$!
echo "$FUZZ_PID" > "$PID_FILE"

sleep 2

if ! kill -0 "$FUZZ_PID" 2>/dev/null; then
    echo "The fuzzer stopped during startup."
    echo
    cat "$LOG_FILE"
    rm -f "$PID_FILE"
    exit 1
fi

echo
echo "============================================================"
echo "RBVR BACKGROUND FUZZING STARTED"
echo "============================================================"
echo "Target:   $TARGET"
echo "PID:      $FUZZ_PID"
echo "Duration: 8 hours"
echo "Log:      $LOG_FILE"
echo
echo "Follow live:"
echo "tail -f $LOG_FILE"
echo
echo "Check whether it is running:"
echo "ps -p $FUZZ_PID -o pid,etime,%cpu,%mem,cmd"
echo
echo "Stop it:"
echo "kill $FUZZ_PID"
echo
echo "Crash artifacts:"
echo "find fuzz/artifacts/$TARGET -maxdepth 1 -type f -print"
echo "============================================================"
