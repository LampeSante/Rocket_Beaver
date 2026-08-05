#!/usr/bin/env bash
set -euo pipefail

cd ~/rocket-beaver-contract

echo "============================================================"
echo "RBVR RELEASE ENGINE — COVERAGE-GUIDED FUZZ PASS"
echo "============================================================"

echo
echo "===== VERIFY FUZZ FILES ====="

test -f fuzz/Cargo.toml || {
    echo "Missing fuzz/Cargo.toml"
    exit 1
}

test -f fuzz/fuzz_targets/process_release.rs || {
    echo "Missing fuzz/fuzz_targets/process_release.rs"
    exit 1
}

sed -n '1,240p' fuzz/fuzz_targets/process_release.rs

echo
echo "===== INSTALL CARGO-FUZZ IF REQUIRED ====="

if ! cargo fuzz --help >/dev/null 2>&1; then
    cargo install cargo-fuzz --locked
fi

echo
echo "===== AVAILABLE FUZZ TARGETS ====="

FUZZ_TARGETS="$(cargo fuzz list)"
printf '%s\n' "$FUZZ_TARGETS"

if ! printf '%s\n' "$FUZZ_TARGETS" | grep -Fxq "process_release"; then
    echo
    echo "The process_release fuzz target is not registered in fuzz/Cargo.toml."
    exit 1
fi

echo
echo "===== FORMAT AND COMPILE BASELINE ====="

cargo fmt --all
cargo test --workspace
cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- \
    -D warnings

echo
echo "===== CREATE CORPUS DIRECTORIES ====="

mkdir -p fuzz/corpus/process_release
mkdir -p fuzz/artifacts/process_release

# Deterministic starter inputs. LibFuzzer will mutate these.
printf '\x00' > fuzz/corpus/process_release/zero
printf '\x01' > fuzz/corpus/process_release/one
printf '\xff' > fuzz/corpus/process_release/max_byte

python3 - <<'PY'
from pathlib import Path

corpus = Path("fuzz/corpus/process_release")
corpus.mkdir(parents=True, exist_ok=True)

seeds = {
    "all_zero_64": bytes(64),
    "all_ff_64": bytes([0xFF]) * 64,
    "ascending_64": bytes(range(64)),
    "alternating_64": bytes([0x00, 0xFF]) * 32,
    "small_values": bytes([
        1, 0, 0, 0, 0, 0, 0, 0,
        2, 0, 0, 0, 0, 0, 0, 0,
        3, 0, 0, 0, 0, 0, 0, 0,
        4, 0, 0, 0, 0, 0, 0, 0,
        5, 0, 0, 0, 0, 0, 0, 0,
    ]),
}

for name, data in seeds.items():
    (corpus / name).write_bytes(data)

print(f"Corpus seeds: {len(list(corpus.iterdir()))}")
PY

echo
echo "===== CLEAR OLD CRASH ARTIFACTS ====="

find fuzz/artifacts/process_release \
    -maxdepth 1 \
    -type f \
    \( -name 'crash-*' -o -name 'leak-*' -o -name 'timeout-*' -o -name 'oom-*' \) \
    -print \
    -delete 2>/dev/null || true

echo
echo "===== FUZZ RELEASE ACCOUNTING ====="

cargo fuzz run process_release \
    fuzz/corpus/process_release \
    -- \
    -max_total_time=180 \
    -timeout=10 \
    -rss_limit_mb=4096 \
    -max_len=512 \
    -print_final_stats=1

echo
echo "===== CHECK FOR FAILURES ====="

FAILURES="$(
    find fuzz/artifacts/process_release \
        -maxdepth 1 \
        -type f \
        \( -name 'crash-*' -o -name 'leak-*' -o -name 'timeout-*' -o -name 'oom-*' \) \
        -print 2>/dev/null || true
)"

if [[ -n "$FAILURES" ]]; then
    echo "Fuzzing produced failure artifacts:"
    printf '%s\n' "$FAILURES"
    exit 1
fi

echo "No crash, leak, timeout, or out-of-memory artifacts found."

echo
echo "===== CORPUS SUMMARY ====="

CORPUS_COUNT="$(
    find fuzz/corpus/process_release -maxdepth 1 -type f | wc -l
)"

CORPUS_BYTES="$(
    find fuzz/corpus/process_release \
        -maxdepth 1 \
        -type f \
        -printf '%s\n' |
    awk '{ total += $1 } END { print total + 0 }'
)"

echo "Corpus files: $CORPUS_COUNT"
echo "Corpus bytes: $CORPUS_BYTES"

echo
echo "===== RE-RUN NORMAL VALIDATION ====="

cargo test --workspace

echo
echo "===== STATUS ====="

git status --short
git diff --stat

echo
echo "============================================================"
echo "RBVR RELEASE FUZZ PASS COMPLETE"
echo "============================================================"
echo "Target: process_release"
echo "Failure artifacts: none"
echo "Normal Rust suite: passed"
echo "============================================================"
