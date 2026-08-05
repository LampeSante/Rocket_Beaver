#!/usr/bin/env bash
set -uo pipefail

ROOT="$(pwd)"
SRC="programs/treasury-router/src"
REPORT="$ROOT/RBVR-SECURITY-AUDIT-LATEST.md"
LOGDIR="$ROOT/.rbvr-security-audit"
FUZZ_TARGET="process_release"

rm -rf "$LOGDIR"
mkdir -p "$LOGDIR"

PASS=0
WARN=0
FAIL=0

section() {
    printf '\n## %s\n\n' "$1" >> "$REPORT"
}

pass() {
    PASS=$((PASS + 1))
    printf -- '- ✅ **PASS:** %s\n' "$1" >> "$REPORT"
}

warn() {
    WARN=$((WARN + 1))
    printf -- '- ⚠️ **REVIEW:** %s\n' "$1" >> "$REPORT"
}

fail() {
    FAIL=$((FAIL + 1))
    printf -- '- ❌ **FAIL:** %s\n' "$1" >> "$REPORT"
}

run_logged() {
    local name="$1"
    shift
    "$@" >"$LOGDIR/$name.log" 2>&1
}

record_matches() {
    local title="$1"
    local pattern="$2"
    local output="$LOGDIR/$3.txt"

    section "$title"

    grep -RniE \
        --include='*.rs' \
        --exclude='*.before-*' \
        --exclude-dir=target \
        --exclude-dir=.git \
        "$pattern" "$SRC" >"$output" 2>/dev/null || true

    if [[ -s "$output" ]]; then
        printf '```text\n' >> "$REPORT"
        sed -n '1,240p' "$output" >> "$REPORT"
        printf '```\n' >> "$REPORT"
    else
        printf '_No matches found._\n' >> "$REPORT"
    fi
}

cat >"$REPORT" <<EOF
# Rocket Beaver Treasury Router — Deep Security Audit

- **Date:** $(date -Iseconds)
- **Branch:** $(git branch --show-current)
- **Commit:** $(git rev-parse HEAD)
- **Rust:** $(rustc --version)
- **Nightly:** $(rustc +nightly --version 2>/dev/null || echo unavailable)
- **Anchor:** $(anchor --version 2>/dev/null || echo unavailable)
- **Solana:** $(solana --version 2>/dev/null || echo unavailable)

This is an automated source and test review. It is not a substitute for an
independent professional audit.
EOF

section "Repository condition"

git status --short >"$LOGDIR/git-status.txt"

if [[ -s "$LOGDIR/git-status.txt" ]]; then
    warn "The working tree contains uncommitted changes. Audit results apply to the exact local working tree, not merely the current commit."
    printf '\n```text\n' >>"$REPORT"
    cat "$LOGDIR/git-status.txt" >>"$REPORT"
    printf '```\n' >>"$REPORT"
else
    pass "Working tree is clean."
fi

if git diff --check >"$LOGDIR/git-diff-check.log" 2>&1; then
    pass "No whitespace or conflict-marker errors detected by git diff --check."
else
    fail "git diff --check found malformed changes."
    printf '\n```text\n' >>"$REPORT"
    cat "$LOGDIR/git-diff-check.log" >>"$REPORT"
    printf '```\n' >>"$REPORT"
fi

section "Formatting and compiler checks"

if run_logged fmt cargo fmt --all -- --check; then
    pass "Rust formatting check passed."
else
    fail "Rust formatting check failed."
fi

if run_logged clippy cargo clippy \
    --workspace \
    --all-targets \
    --all-features \
    -- \
    -D warnings \
    -A unexpected_cfgs; then
    pass "Strict Clippy passed, excluding known Anchor cfg noise."
else
    fail "Strict Clippy failed."
    printf '\n```text\n' >>"$REPORT"
    tail -n 160 "$LOGDIR/clippy.log" >>"$REPORT"
    printf '```\n' >>"$REPORT"
fi

section "Production Rust tests"

if run_logged rust-tests cargo test --workspace --all-features; then
    pass "All Rust production tests passed."
    printf '\n```text\n' >>"$REPORT"
    tail -n 35 "$LOGDIR/rust-tests.log" >>"$REPORT"
    printf '```\n' >>"$REPORT"
else
    fail "Rust production tests failed."
    printf '\n```text\n' >>"$REPORT"
    tail -n 200 "$LOGDIR/rust-tests.log" >>"$REPORT"
    printf '```\n' >>"$REPORT"
fi

section "Release-engine fuzz build"

if run_logged fuzz-build cargo +nightly fuzz build "$FUZZ_TARGET"; then
    pass "The process_release fuzz target builds under nightly with sanitizer instrumentation."
else
    fail "The process_release fuzz target failed to build."
    printf '\n```text\n' >>"$REPORT"
    tail -n 160 "$LOGDIR/fuzz-build.log" >>"$REPORT"
    printf '```\n' >>"$REPORT"
fi

section "Bounded release-engine fuzz campaign"

if run_logged fuzz-campaign \
    cargo +nightly fuzz run "$FUZZ_TARGET" -- \
    -runs=1000000 \
    -max_len=160 \
    -timeout=10 \
    -print_final_stats=1; then
    pass "The bounded 1,000,000-run process_release campaign completed without a crash."
else
    fail "The bounded process_release campaign terminated abnormally."
    printf '\n```text\n' >>"$REPORT"
    tail -n 200 "$LOGDIR/fuzz-campaign.log" >>"$REPORT"
    printf '```\n' >>"$REPORT"
fi

section "Previously saved slow-unit reproduction"

SLOW_UNIT="$(
    find "fuzz/artifacts/$FUZZ_TARGET" \
        -maxdepth 1 \
        -type f \
        -name 'slow-unit-*' \
        -printf '%T@ %p\n' 2>/dev/null |
    sort -nr |
    head -n1 |
    cut -d' ' -f2-
)"

if [[ -n "${SLOW_UNIT:-}" && -f "$SLOW_UNIT" ]]; then
    printf -- '- Artifact: `%s`\n' "$SLOW_UNIT" >>"$REPORT"

    if timeout 20s \
        cargo +nightly fuzz run "$FUZZ_TARGET" "$SLOW_UNIT" -- \
        -runs=1 \
        -timeout=10 \
        >"$LOGDIR/slow-unit.log" 2>&1; then
        pass "The saved slow unit reproduced and terminated within the audit timeout."
    else
        STATUS=$?
        if [[ "$STATUS" -eq 124 ]]; then
            fail "The saved slow unit exceeded 20 seconds. This indicates a potential fuzz-harness complexity or denial-of-service issue requiring investigation."
        else
            fail "The saved slow unit caused an abnormal fuzzer result."
        fi

        printf '\n```text\n' >>"$REPORT"
        tail -n 160 "$LOGDIR/slow-unit.log" >>"$REPORT"
        printf '```\n' >>"$REPORT"
    fi
else
    warn "No saved slow-unit artifact was found."
fi

record_matches \
    "Unchecked and saturating arithmetic inventory" \
    'saturating_(add|sub|mul)|wrapping_(add|sub|mul)|unchecked_|[^a-zA-Z_]as[[:space:]]+(u8|u16|u32|u64|usize|i8|i16|i32|i64)' \
    arithmetic-review

if grep -RniE \
    --include='*.rs' \
    --exclude='*.before-*' \
    --exclude-dir=target \
    'saturating_(add|sub|mul)|wrapping_(add|sub|mul)' \
    "$SRC" >"$LOGDIR/saturating.txt" 2>/dev/null; then
    warn "Saturating or wrapping arithmetic remains in production source. Each match must be justified as fail-closed or replaced with checked arithmetic."
else
    pass "No saturating or wrapping arithmetic found in production Rust source."
fi

record_matches \
    "Explicit panic and assertion inventory" \
    'unwrap\(|expect\(|panic!\(|assert!\(|assert_eq!\(|unreachable!\(|todo!\(|unimplemented!\(' \
    panic-review

if grep -RniE \
    --include='*.rs' \
    --exclude='*.before-*' \
    --exclude-dir=target \
    --exclude='*test*' \
    'unwrap\(|expect\(|panic!\(|unreachable!\(|todo!\(|unimplemented!\(' \
    "$SRC" >"$LOGDIR/production-panics.txt" 2>/dev/null; then
    warn "Potential panic-producing calls exist. Review the listed locations and distinguish test-only modules from reachable production code."
else
    pass "No obvious unwrap, expect, panic, todo, or unimplemented calls found outside test-named files."
fi

record_matches \
    "Authority and privileged-control inventory" \
    'authority|admin|owner|signer|Signer<' \
    authority-review

record_matches \
    "PDA and account-linkage inventory" \
    'seeds[[:space:]]*=|bump[[:space:]]*=|has_one[[:space:]]*=|constraint[[:space:]]*=|address[[:space:]]*=' \
    pda-review

record_matches \
    "CPI and external-program inventory" \
    'invoke\(|invoke_signed\(|CpiContext|token::transfer|transfer_checked|burn|mint_to|set_authority|close_account' \
    cpi-review

if grep -RniE \
    --include='*.rs' \
    --exclude='*.before-*' \
    --exclude-dir=target \
    'invoke\(|invoke_signed\(|CpiContext|token::transfer|transfer_checked|burn|mint_to|set_authority|close_account' \
    "$SRC" >"$LOGDIR/cpi-matches.txt" 2>/dev/null; then
    warn "CPI or token-authority operations exist and require manual account-program, mint, owner, signer-seed, and destination review."
else
    pass "No direct CPI or token authority calls were found in the scanned source."
fi

record_matches \
    "Initialization and reinitialization inventory" \
    '#\[account\([^]]*(init|init_if_needed)|pub fn initialize|initialized_at|version[[:space:]]*=' \
    initialization-review

record_matches \
    "Pause, configuration and upgrade controls" \
    'paused|configuration_updates_enabled|upgrade|set_authority|close|realloc' \
    mutability-review

section "Protocol authority architecture"

if grep -RniE \
    --include='*.rs' \
    --exclude='*.before-*' \
    --exclude-dir=target \
    'protocol_state\.authority|authority[[:space:]]*=' \
    "$SRC" >"$LOGDIR/protocol-authority.txt" 2>/dev/null; then
    warn "Protocol authority fields or assignments remain present. Confirm whether they are initialization-only, whether any privileged instruction consumes them, and how final authority revocation is enforced."
    printf '\n```text\n' >>"$REPORT"
    sed -n '1,180p' "$LOGDIR/protocol-authority.txt" >>"$REPORT"
    printf '```\n' >>"$REPORT"
else
    pass "No protocol authority field use was detected."
fi

section "Release instruction consistency"

for bucket in reserve liquidity company founder buyback; do
    file="$SRC/instructions/$bucket.rs"

    if [[ ! -f "$file" ]]; then
        fail "Missing release instruction file: $file"
        continue
    fi

    if grep -q 'process_release' "$file"; then
        pass "$bucket delegates accounting mutation to the centralized process_release engine."
    else
        fail "$bucket does not appear to call the centralized process_release engine."
    fi

    if grep -qE 'transfer|transfer_checked' "$file"; then
        pass "$bucket contains a token transfer operation."
    else
        warn "$bucket contains no obvious token transfer call; verify that value movement occurs through a shared helper or CPI wrapper."
    fi
done

section "Dependency vulnerability checks"

if command -v cargo-audit >/dev/null 2>&1; then
    if run_logged cargo-audit cargo audit; then
        pass "cargo audit reported no known vulnerable Rust dependencies."
    else
        warn "cargo audit reported advisories or could not complete."
        printf '\n```text\n' >>"$REPORT"
        tail -n 180 "$LOGDIR/cargo-audit.log" >>"$REPORT"
        printf '```\n' >>"$REPORT"
    fi
else
    warn "cargo-audit is not installed. Install with: cargo install cargo-audit --locked"
fi

if [[ -f package-lock.json ]]; then
    if run_logged npm-audit npm audit --omit=dev; then
        pass "npm production-dependency audit passed."
    else
        warn "npm reported production dependency vulnerabilities or audit errors."
        printf '\n```text\n' >>"$REPORT"
        tail -n 180 "$LOGDIR/npm-audit.log" >>"$REPORT"
        printf '```\n' >>"$REPORT"
    fi
else
    warn "No package-lock.json found; npm dependency audit was skipped."
fi

section "Secret and private-key filename scan"

find . \
    -path './.git' -prune -o \
    -path './target' -prune -o \
    -path './node_modules' -prune -o \
    -type f \
    \( -iname '*.json' -o -iname '*.pem' -o -iname '*.key' -o -iname '*secret*' -o -iname '*wallet*' \) \
    -print >"$LOGDIR/key-filenames.txt"

if grep -Ev \
    'package(-lock)?\.json$|tsconfig\.json$|beavernomics|metadata|deployment|authority-architecture|idl|target/' \
    "$LOGDIR/key-filenames.txt" >"$LOGDIR/suspicious-key-filenames.txt"; then
    warn "Potentially sensitive filenames were detected. Confirm none contain deployer, treasury, mint, upgrade-authority, or wallet secret bytes."
    printf '\n```text\n' >>"$REPORT"
    sed -n '1,160p' "$LOGDIR/suspicious-key-filenames.txt" >>"$REPORT"
    printf '```\n' >>"$REPORT"
else
    pass "No obviously suspicious private-key filenames were detected outside excluded paths."
fi

section "Final automated assessment"

printf -- '- **Passes:** %d\n' "$PASS" >>"$REPORT"
printf -- '- **Manual reviews/warnings:** %d\n' "$WARN" >>"$REPORT"
printf -- '- **Failures:** %d\n' "$FAIL" >>"$REPORT"

if [[ "$FAIL" -eq 0 ]]; then
    printf '\n**AUTOMATED RESULT: PASS WITH MANUAL REVIEW ITEMS**\n' >>"$REPORT"
else
    printf '\n**AUTOMATED RESULT: FAIL — DO NOT DEPLOY**\n' >>"$REPORT"
fi

printf '\nReport written to:\n%s\n\n' "$REPORT"
printf 'Passes: %d | Reviews: %d | Failures: %d\n' "$PASS" "$WARN" "$FAIL"

if [[ "$FAIL" -gt 0 ]]; then
    exit 1
fi
