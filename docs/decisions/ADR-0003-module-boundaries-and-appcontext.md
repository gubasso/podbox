# ADR-0003: Parse-shape vs runtime-shape modules; one `AppContext`

## Context and Problem Statement

A CLI mixes argument parsing, orchestration, pure domain logic, and I/O. Left untangled these defeat
testing and the parse-don't-validate discipline. cli-design `00-architecture.md` prescribes a
parse-shape vs runtime-shape split and a single `AppContext`.

## Considered Options

- Ad-hoc modules; handlers read args and perform I/O inline.
- A global / static / thread-local context.
- Layered module boundaries plus one `AppContext` built once in `main`.

## Decision Outcome

Chosen option: **layered boundaries** — `cli/` holds clap derive structs only (parse-shape);
`commands/` holds free-fn handlers `run(&AppContext, <Verb>Args) -> Result<(), AppError>`; `domain/`
is pure types + newtypes with **zero I/O**; `adapters/` is the sole place that talks to external
systems (each with its own error enum); `services/` orchestrates adapters. One `AppContext` is built
once in `main` and passed by reference — no globals, statics, or thread-locals. Handlers first project
args into a domain `Request` via `Request::from_cli(args)?` (parse-don't-validate).

## Consequences

- Good: `domain/` is unit-testable without I/O; adapters are swappable/mockable (golden-argv tests,
  ADR-0008); per-workspace state stays serialized behind `state_store` (I3); config is resolved once
  (C8); output paths stay deterministic (U4).
- Bad: more files and one indirection from raw args to domain requests.
- Enacted by: `src/context.rs`, `src/domain/`, `src/adapters/`, `src/services/`; reference
  `docs/reference/spec/13-rust-implementation-binding.md`.

## Status

Accepted
