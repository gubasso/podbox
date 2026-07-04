# ADR-0007: Layered config via `figment`; preserve unknown keys (no `deny_unknown_fields`)

## Context and Problem Statement

podbox resolves configuration from several sources with a fixed precedence (C8) across four XDG roots
(C1–C6). The neutral `03` requires unknown `config.toml` keys be **preserved and ignored** for
forward-compatibility. cli-spec `05-config.md`'s `Config` uses `#[serde(default, deny_unknown_fields)]`,
which would reject them.

## Considered Options

- Hand-rolled precedence merging.
- The `config` / `confy` crates.
- `figment` layered providers with per-key provenance.

## Decision Outcome

Chosen option: **figment** — layered providers implement the C8 chain exactly (CLI → env → project
registry → local workspace file → work-clone discovery, else exit `2`), with `serde` + `toml` and
`directories` / `camino` for paths. **Do NOT set `deny_unknown_fields`** on the top-level
`config.toml` `Config`: podbox `03`'s forward-compat rule wins over cli-spec `05`. Map the four XDG
roots plus `PODBOX_HOME` / `$PODBOX_CONFIG_HOME`.

## Consequences

- Good: C1–C8 honored; per-key provenance powers `config show`; a newer `config.toml` still loads on
  an older binary (forward-compat).
- Bad: a deliberate divergence from the cli-spec chapter default — recorded here and flagged to
  docs-n-notes (Phase 7) as a trade-off note.
- Bad: unknown keys are silently ignored rather than erroring; acceptable per the neutral contract, a
  strict mode is a future option.
- Enacted by: `src/config/`, `src/context.rs`;
  `docs/reference/spec/03-config-and-xdg-layout.md`.

## Status

Accepted
