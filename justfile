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

# Mutation testing scoped to ADR-0013 critical modules.
mutants-critical:
    #!/usr/bin/env bash
    set -euo pipefail
    set +e
    output="$({{nix}} cargo mutants --package podbox \
        --file src/services/compose.rs \
        --file src/domain/digest.rs \
        --file src/exit.rs \
        --file src/config/loader.rs 2>&1)"
    status=$?
    set -e
    echo "$output"
    summary="$(grep -E '[0-9]+ mutants tested .*: [0-9]+ missed, [0-9]+ caught, [0-9]+ unviable' <<<"$output" | tail -1 || true)"
    if [[ -z "$summary" ]]; then
        exit "$status"
    fi
    missed="$(sed -E 's/.*: ([0-9]+) missed, ([0-9]+) caught, ([0-9]+) unviable.*/\1/' <<<"$summary")"
    caught="$(sed -E 's/.*: ([0-9]+) missed, ([0-9]+) caught, ([0-9]+) unviable.*/\2/' <<<"$summary")"
    viable=$((missed + caught))
    percent=$((caught * 100 / viable))
    if (( percent < 60 )); then
        echo "mutation score ${percent}% is below required 60%" >&2
        exit 2
    fi
    echo "mutation score ${percent}% meets required 60%"

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
    {{nix}} cargo bloat --release --bin podbox

# Verify the published crate payload stays lean (CLAUDE.md).
package-check:
    {{nix}} cargo package --list --allow-dirty

# Generate release distribution assets without adding source-tree artifacts.
dist-assets:
    mkdir -p target/dist/completions target/dist/man
    {{nix}} cargo run --bin podbox -- completion bash > target/dist/completions/podbox.bash
    {{nix}} cargo run --bin podbox -- completion zsh > target/dist/completions/_podbox
    {{nix}} cargo run --bin podbox -- completion fish > target/dist/completions/podbox.fish
    {{nix}} cargo run --bin podbox -- completion powershell > target/dist/completions/podbox.ps1
    {{nix}} cargo run --bin podbox -- completion elvish > target/dist/completions/podbox.elv
    {{nix}} cargo run --bin podbox -- manpage > target/dist/man/podbox.1

# --- Release / publish (bootstrap-cargo-publish) ---
#
# Publishing logic lives in the deployed scripts/ helpers, never inlined here.
# The auth gate lives only in ./scripts/publish. CI (release-plz, Trusted
# Publishing via OIDC) is the normal release path; these recipes are the local
# operator surface. See PUBLISHING.md.

# Dry-run crates.io readiness (no token required).
publish-dry:
    {{nix}} ./scripts/publish-dry

# Publish to crates.io (auth-gated; the first publish is manual).
publish:
    {{nix}} ./scripts/publish

# Update versions + changelog locally (release-plz update).
release-update:
    {{nix}} ./scripts/release release-plz-update

# Open/refresh the release PR (release-plz release-pr).
release-pr:
    {{nix}} ./scripts/release release-plz-pr

# Check public API semver compatibility (cargo semver-checks).
semver-check:
    {{nix}} ./scripts/release semver-check

# --- Aggregate gate ---

# What contributors (and CI) run before pushing: lint + test + supply-chain.
check: lint test deny audit machete
