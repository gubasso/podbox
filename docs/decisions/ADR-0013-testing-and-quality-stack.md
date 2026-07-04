# ADR-0013: Testing and quality stack

## Context and Problem Statement

podbox's guarantees (deterministic composition, no stale builds, correct exit codes, hardened wrapper
argv, no stdout pollution) need automated enforcement, or they regress silently.

## Considered Options

- Unit tests only.
- Manual QA.
- A layered stack: integration + snapshot + property + mutation + supply-chain, wired as CI gates.

## Decision Outcome

Chosen option: **layered stack** — `assert_cmd` + `predicates` with one `tests/cmd_<name>.rs` per
subcommand; `insta` snapshots for `--help` and JSON output; `cargo-nextest` as the runner; `proptest`
for parsers, the digest, and the state machine; `cargo-mutants` (≥60% on critical modules: compose,
digest, exit-map, config precedence); a `cargo-bloat` baseline;
`cargo-deny` / `cargo-audit` / `cargo-machete` for supply-chain hygiene; golden-argv tests against a
stub child for the wrapper (ADR-0008). Two CI gates are mandatory: the exit-code matrix unit test
(ADR-0005) and a grep-lint forbidding `println!` / `eprintln!` outside `ui/` and `main.rs`.

## Consequences

- Good: honors U4 and the cli-design `99-checklist` testing block; the wrapper's hardening argv
  (S3/S4/S5) is snapshot-locked; the exit taxonomy (`0`–`7`) is a tested contract.
- Bad: mutation testing and snapshots add CI time and snapshot-review overhead — accepted for
  correctness.
- Enacted by: `tests/`, `deny.toml`, the CI config; `docs/reference/rust-checklist.md`; dev-deps
  added via `cargo add --dev` (ADR-0012).

## Status

Accepted
