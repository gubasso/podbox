# ADR-0012: Manage dependencies only through cargo CLI commands

## Context and Problem Statement

Dependency drift and incoherent version graphs arise when contributors or agents hand-edit
`Cargo.toml` with guessed names/versions. The operator mandates that dependency changes go only
through the package manager's resolve-and-lock path. cli-spec `07-dependencies.md` is currently silent
on this and its examples hand-edit `Cargo.toml`.

## Considered Options

- Hand-edit `Cargo.toml` dependency lines (the current cli-spec examples).
- A committee-approved pinned-version list edited by hand.
- Cargo CLI only: `cargo add` / `cargo remove` / `cargo update`.

## Decision Outcome

Chosen option: **cargo CLI only** — all dependency add/upgrade/removal/feature changes MUST go through
`cargo add`, `cargo remove`, `cargo update`. Coding agents and contributors MUST NOT hand-edit
dependency names, versions, or features in `Cargo.toml`. Rationale: `cargo add` selects the latest
SemVer-compatible version, resolves the whole graph, and updates `Cargo.lock` atomically. `Cargo.lock`
is committed for the binary crate. Editing non-dependency `[package]` metadata (e.g. `rust-version`)
by hand remains allowed.

## Consequences

- Good: version graphs stay coherent and current; no stale hand-pinned strings; reproducible via the
  committed lockfile.
- Good: enforced as podbox invariant **R1** (`11-…`) plus a contributor guide, and mirrored into
  docs-n-notes (Phase 7).
- Bad: requires the toolchain present to change deps (no offline hand-edit) — accepted.
- Enacted by: `docs/reference/spec/11-invariants-and-guarantees.md` (R1),
  `docs/guides/contributing-rust.md`, the Phase 8 skeleton's `cargo add` history; mirrored in
  `docs-n-notes/.../cli-spec/07-dependencies.md` + `cli-spec/adr/0001-cargo-cli-only-dependencies.md`.

## Status

Accepted
