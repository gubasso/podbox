# ADR-0006: Logging via `tracing` with an XDG state-file sink

## Context and Problem Statement

podbox needs structured, filterable diagnostics that never pollute stdout data (U4) and that persist
for debugging, while staying XDG-compliant (C5).

## Considered Options

- `log` + `env_logger`.
- Ad-hoc `println!` / `eprintln!`.
- `tracing` + `tracing-subscriber` + `tracing-appender`.

## Decision Outcome

Chosen option: **the tracing stack** — one subscriber built in `main`; honor `RUST_LOG` (do **not**
invent `PODBOX_LOG`); default file sink `$XDG_STATE_HOME/podbox/podbox.log` resolved via
`directories`; hold the `WorkerGuard` for the program lifetime; the JSON layer sets
`.with_ansi(false)`. All human-facing output goes through `ui/` (never `println!` elsewhere), keeping
stdout = data / stderr = diagnostics (U4).

## Consequences

- Good: C5 (state root) and U4 satisfied; per-key spans aid doctor and wrapper debugging.
- Good: `--quiet` / `--verbose` / `--no-color` map to subscriber filters without touching stdout
  data or the exit code.
- Bad: a small startup cost and the guard-lifetime footgun — mitigated by holding the `WorkerGuard`
  in `AppContext`.
- Enacted by: `src/logging.rs`, `src/ui/`; reference
  `docs/reference/spec/13-rust-implementation-binding.md`. The no-print-outside-`ui/` grep-lint is a
  CI gate (ADR-0013).

## Status

Accepted
