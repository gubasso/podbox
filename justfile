# Justfile — podbox task runner.
#
# Recipes are the single source of truth for the ADR-0013 quality stack; CI
# (bootstrap-ci) reuses these recipe names rather than re-encoding commands.
# Every recipe runs through the pinned Nix devShell (flake.nix) so local and CI
# runs share one toolchain. Dependencies are managed via `cargo add`/`remove`
# only (ADR-0012) — never hand-edited into Cargo.toml.

# Command prefix: run inside the pinned devShell so tools are on PATH.
nix := "nix develop --command"

# List available recipes.
default:
    @just --list

# --- Build / run (inner loop) ---

# Build a release binary.
build:
    {{nix}} cargo build --release

# Run the binary, forwarding args (e.g. `just run -- self help`).
run *ARGS:
    {{nix}} cargo run -- {{ARGS}}

# Remove Cargo build artifacts.
clean:
    {{nix}} cargo clean

# --- Formatting ---

# Format all Rust sources in place.
fmt:
    {{nix}} cargo fmt --all

# Auto-fix formatting and clippy lints.
fix:
    {{nix}} cargo fmt --all
    {{nix}} cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features

# --- Tests (cargo-nextest, ADR-0013) ---

# Run the test suite via cargo-nextest (includes the exit-code matrix, ADR-0005).
test:
    {{nix}} cargo nextest run --all-features

# Review and accept pending insta snapshots (ADR-0013).
snapshot-review:
    {{nix}} cargo insta review

# Mutation testing on critical modules (compose, digest, exit-map, config).
mutants:
    {{nix}} cargo mutants

# --- Lint / static analysis (ADR-0013) ---

# clippy as a hard gate (warnings are errors).
clippy:
    {{nix}} cargo clippy --all-targets --all-features -- -D warnings

# fmt --check + clippy + the stdout/stderr grep-lint.
lint: clippy lint-print
    {{nix}} cargo fmt --all --check

# Forbid println!/eprintln!/print!/eprint! outside src/ui/ and src/main.rs
# (ADR-0013: no stdout pollution). Exits non-zero on any match.
lint-print:
    #!/usr/bin/env bash
    set -euo pipefail
    if {{nix}} rg -n '\b(println!|print!|eprintln!|eprint!)' \
        --glob '!src/ui/**' --glob '!src/main.rs' src; then
        echo "error: println!/eprintln! found outside src/ui/ and src/main.rs" >&2
        exit 1
    fi

# --- Supply-chain hygiene (ADR-0013) ---

# cargo-deny: licenses, advisories, bans, sources.
deny:
    {{nix}} cargo deny check

# cargo-audit: RUSTSEC advisory scan.
audit:
    {{nix}} cargo audit

# cargo-machete: detect unused dependencies.
machete:
    {{nix}} cargo machete

# cargo-bloat: binary size baseline.
bloat:
    {{nix}} cargo bloat --release

# --- Aggregate gate ---

# What contributors (and CI) run before pushing: lint + test + supply-chain.
check: lint test deny audit machete
