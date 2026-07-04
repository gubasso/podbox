# ADR-0005: Error taxonomy — thiserror inside, anyhow at edge, podbox `0`–`7` exit codes

## Context and Problem Statement

podbox needs one coherent error path and a machine-stable exit-code contract. The neutral spec
(`02`/`10`/`11`) fixes a `0`–`7` exit taxonomy and stream discipline (U4, C8, U5, B2). cli-spec
`03-error-handling.md`'s worked examples use raw BSD sysexits (64/65/66/69/70/74/77/78), which clash.

## Considered Options

- Adopt BSD sysexits verbatim (the cli-spec `03` example values).
- A catch-all `_ => 1` fallback mapping.
- One `AppError` mapping to podbox's own `0`–`7`, with sysexits as design influence only.

## Decision Outcome

Chosen option: **podbox `0`–`7` governs** — `thiserror` per layer, `anyhow` only at the binary
boundary, one `AppError::exit_code() -> u8`. BSD sysexits are influence, not the contract. The map:
success `0`; general `1`; usage/config-syntax `2`; unmet host/runtime `3`; validation `4`;
workspace-not-found/not-running `5`; build `6`; destructive-refused `7`. **No catch-all `_ => 1`** —
every variant maps explicitly. Adopt stable `err.kind` keys (cli-design `02`) reusing podbox's
existing stable IDs (doctor check IDs, workspace identity, digests).

## Consequences

- Good: honors C8 (exit `2` on unresolved config), U4/U5, B2; scriptable per `10`.
- Good: the mapping is a unit-tested matrix (ADR-0013), so drift is caught in CI.
- Bad: diverges from the cli-spec chapter's example exit values — flagged back to docs-n-notes
  (Phase 7) as a trade-off note.
- Enacted by: `src/error.rs`, `src/exit.rs`;
  `docs/reference/spec/10-errors-output-and-scriptability.md`.

## Status

Accepted
