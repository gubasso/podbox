# Contributing: managing Rust dependencies

> Guide (task). How to add, change, or remove a crate in podbox. This is the enacting workflow for
> invariant **R1** ([`../reference/spec/11-invariants-and-guarantees.md`](../reference/spec/11-invariants-and-guarantees.md)
> §10) and [ADR-0012](../decisions/ADR-0012-cargo-cli-only-dependencies.md). For the exact crate set,
> see [`../reference/spec/13-rust-implementation-binding.md`](../reference/spec/13-rust-implementation-binding.md).

## The rule

**Never hand-edit dependency lines in `Cargo.toml`.** Every dependency add, upgrade, removal, or
feature change goes through a `cargo` command that fetches a SemVer-compatible version, resolves the
whole graph, and updates `Cargo.lock`. This applies to humans and to coding agents equally.

`Cargo.lock` is committed (podbox ships a binary). Editing non-dependency `[package]` metadata (e.g.
`rust-version`, `description`) by hand is fine — the rule is about **dependencies**, not the whole
manifest.

## Approved commands

```bash
# add a runtime dependency (latest compatible version, graph resolved, lockfile updated)
cargo add clap --features derive,env,wrap_help

# add several at once
cargo add anyhow thiserror tracing serde serde_json toml directories camino

# add a dev-dependency
cargo add --dev assert_cmd predicates insta tempfile

# change features on an existing dependency
cargo add tracing-subscriber --features env-filter,fmt

# remove a dependency
cargo remove some-crate

# upgrade within the allowed SemVer range and refresh the lockfile
cargo update -p clap
```

Install CLI quality tools with `cargo install` (they are not project dependencies):

```bash
cargo install cargo-nextest cargo-deny cargo-machete cargo-mutants cargo-bloat
```

## Forbidden

- ❌ Opening `Cargo.toml` and typing `clap = "4"` (or any `name = "version"`) by hand.
- ❌ Adding or editing a `features = [...]` list on a dependency by hand.
- ❌ Guessing a version number from memory instead of letting `cargo add` resolve it.
- ❌ Committing a `Cargo.toml` dependency change without the matching `Cargo.lock` update.

## Why

`cargo add` resolves against the live index, so the graph stays coherent and current; hand-editing
risks stale or incompatible pins and a `Cargo.lock` that no longer matches the resolver. See
[ADR-0012](../decisions/ADR-0012-cargo-cli-only-dependencies.md) for the full rationale and
consequences.
