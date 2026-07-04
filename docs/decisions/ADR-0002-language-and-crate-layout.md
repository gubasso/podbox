# ADR-0002: Rust, single binary crate, edition 2024

## Context and Problem Statement

podbox needs an implementation language and crate topology. The neutral spec (N1) mandates none; this
binding chooses one. A minimal `Cargo.toml` + `src/main.rs` crate already exists and is git-tracked.
cli-spec `01-crate-layout.md` defines the triggers for splitting into a workspace.

## Considered Options

- A multi-crate Cargo workspace up front (core lib + binary + adapters).
- A single binary crate, split later when a trigger fires.
- A non-Rust language (out of scope; the operator chose Rust).

## Decision Outcome

Chosen option: **single binary crate** — the cli-spec `01` workspace triggers (a 2nd binary sharing
≥30% code, an independently-publishable subsystem, >~10s warm `cargo check`, independent adapter
dependency trees; hard-look ~8k LOC) are not yet met. **Retain and expand the existing crate — never
`cargo init`.** Set `edition = "2024"`, `rust-version = "1.85"`. Adopt the canonical `src/` tree from
cli-spec `00-directory-tree.md`.

## Consequences

- Good: least ceremony; fast builds; matches idiomatic small-CLI layout (fd/bat/ouch).
- Good: the existing tracked crate and its history are preserved.
- Bad: a future workspace split is a migration cost — deferred deliberately behind a named trigger,
  to be recorded in its own ADR when a trigger fires.
- Enacted by: `Cargo.toml` `[package]` metadata, `src/` per ADR-0003, the optional Phase 8 skeleton.
  All dependency changes go through ADR-0012 (`cargo add`).

## Status

Accepted
