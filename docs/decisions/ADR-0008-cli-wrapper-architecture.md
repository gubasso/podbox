# ADR-0008: CLI-wrapper architecture over the container runtime and `devcontainer` CLI

## Context and Problem Statement

podbox drives external tools — a rootless microVM container runtime (e.g. `podman run --runtime
krun`) and the `devcontainer` CLI subset — while owning lifecycle itself (X2) and emitting only argv
vectors, never `sh -c` (X1). It must forward signals, map exit codes, and apply security hardening
(S3/S4/S5) on the generated command line.

## Considered Options

- Shell out with formatted strings / `sh -c`.
- Delegate lifecycle to a heavyweight orchestrator.
- Typed builders behind a `Spawner` / `Executable` port (cli-design `06-cli-wrapper-design/`).

## Decision Outcome

Chosen option: **typed builders + a Spawner port** — each external tool has a typed builder with a
`to_args()` / `into_command()` serialization boundary in `adapters/`. Argv uses the three-zone layout
`[WRAPPER-OPTS] <verb> [--] [CHILD-ARGS...]` with `--` verbatim passthrough and namespaced
`--podbox-*` wrapper flags; default to verbatim pass-through. Spawn (not exec) when signals must
forward or exit codes map; propagate child N→N and killed-by-signal→128+N. Resolve the inner binary
`$PODBOX_<TOOL>_BIN → config → PATH → bundled` (missing→127, not-exec→126). Honors X1, X2, S3/S4/S5;
cites backend `50`/`60`. krun historically lacks `podman exec`, so podbox drives exec via argv itself.

## Consequences

- Good: the exact child argv (including hardening flags) is snapshot-locked by golden-argv tests
  against a stub child (ADR-0013), so S3/S4/S5 stay reviewable.
- Bad: a typed builder per tool is more code than string interpolation — the safety is the point.
- Enacted by: `src/adapters/runtime.rs`, `src/adapters/devcontainer.rs`; reference
  `docs/reference/spec/13-rust-implementation-binding.md`.

## Status

Accepted
