# Rust binding traceability

> Reference (lookup). Binds every normative invariant from
> [`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md) — plus the implementation
> invariant **R1** — to the ADR(s) in [`../../decisions/`](../../decisions/) and the Rust module(s)
> that enact it. Completeness gate: **every invariant has ≥ 1 binding row.** The neutral contract is
> unchanged; this matrix is the binding layer's coverage proof (invariants N1–N3 keep the two layers
> distinct).

## Configuration

| Invariant | ADR(s) | Owning module(s) |
| --------- | ------ | ---------------- |
| C1 `devcontainer/` = layer dirs only | ADR-0007 | `config/`, `domain/` typed config (validation) |
| C2 `manifests/` = manifests only | ADR-0007 | `config/`, `domain/manifest.rs` |
| C3 `config.toml` (no `projects.yaml`) | ADR-0007 | `config/` |
| C4 no `default/devcontainer.json` level | ADR-0007 | `config/` (resolution excludes it) |
| C5 full XDG across config/cache/state/data | ADR-0006, ADR-0007 | `logging.rs` (state sink), `config/` (`directories`) |
| C6 data-root seed-only | ADR-0007 | `config/` (root resolution) |
| C7 composed `devcontainer.json` is derived cache | ADR-0009 | `services/compose.rs` |
| C8 precedence chain; exit `2` on none | ADR-0007, ADR-0005 | `config/`, `exit.rs` |

## Composition

| Invariant | ADR(s) | Owning module(s) |
| --------- | ------ | ---------------- |
| P1 ordered `layers` = selection API | ADR-0009 | `domain/manifest.rs` |
| P2 last layer = protected leaf | ADR-0009 | `services/compose.rs`, `domain/manifest.rs` |
| P3 deterministic merge | ADR-0009 | `domain/manifest.rs` |
| P4 auto-composition | ADR-0009 | `services/compose.rs` |
| P5 digest-keyed freshness (fail closed) | ADR-0009 | `domain/digest.rs` |

## Isolation and security

| Invariant | ADR(s) | Owning module(s) |
| --------- | ------ | ---------------- |
| S1 microVM primary boundary | ADR-0008 | `adapters/runtime.rs` |
| S2 rootless-operable | ADR-0008 | `adapters/runtime.rs` |
| S3 drop-caps / no-new-privs / tmpfs `/tmp` | ADR-0008 | `adapters/runtime.rs` (hardening argv) |
| S4 default-deny egress + allowlist | ADR-0008 | `domain/network.rs`, `adapters/runtime.rs` |
| S5 no cred / control-socket live-mount | ADR-0008 | `adapters/runtime.rs` (mount policy) |
| S6 relaxations explicit + visible | ADR-0011 | `domain/doctor.rs`, `commands/status.rs` |

## Execution

| Invariant | ADR(s) | Owning module(s) |
| --------- | ------ | ---------------- |
| X1 argv-vector hooks/`exec` (no `sh -c`) | ADR-0008 | `adapters/runtime.rs`, `adapters/devcontainer.rs` |
| X2 native parse/compose, drive lifecycle | ADR-0008 | `adapters/devcontainer.rs`, `domain/manifest.rs` |

## Build

| Invariant | ADR(s) | Owning module(s) |
| --------- | ------ | ---------------- |
| B1 detect source changes by default | ADR-0010 | `domain/image.rs`, `domain/digest.rs` |
| B2 no silent stale build | ADR-0010 | `services/image_build.rs` |

## Identity and lifecycle

| Invariant | ADR(s) | Owning module(s) |
| --------- | ------ | ---------------- |
| I1 stable workspace identity | ADR-0003 | `domain/workspace.rs` |
| I2 `workspace reconcile` first-class/idempotent | ADR-0004, ADR-0009 | `commands/workspace.rs` (reconcile), `services/compose.rs` |
| I3 per-workspace state serialized | ADR-0003 | `adapters/state_store.rs`, `domain/state.rs` |

## UX

| Invariant | ADR(s) | Owning module(s) |
| --------- | ------ | ---------------- |
| U1 `shell` central, first in help | ADR-0004 | `cli/shell.rs`, `commands/shell.rs` |
| U2 `init` lean + runs doctor after | ADR-0011 | `commands/init.rs`, `commands/doctor.rs` |
| U3 doctor specific/actionable | ADR-0011 | `commands/doctor.rs`, `domain/doctor.rs` |
| U4 stream discipline + `--json` + quiet/verbose/no-color | ADR-0003, ADR-0005, ADR-0006 | `ui/`, `error.rs`, `logging.rs` |
| U5 destructive gated (`--yes`/`--force`) + `--dry-run` | ADR-0005 | `cli/mod.rs` (GlobalArgs), `error.rs` (exit `7`) |

## Neutrality

| Invariant | ADR(s) | Owning module(s) |
| --------- | ------ | ---------------- |
| N1 no mandated language/framework/layout | ADR-0001 | binding layer kept separate from `reference/spec/` |
| N2 no single tool is the only impl | ADR-0001, ADR-0008 | `adapters/` trait boundary (swappable) |
| N3 formats-as-contracts, not the reader | ADR-0001 | `domain/` types mirror the neutral formats |

## Implementation / contribution

| Invariant | ADR(s) | Owning module(s) |
| --------- | ------ | ---------------- |
| R1 cargo-CLI-only dependency management | ADR-0012 | `Cargo.toml`/`Cargo.lock` via `cargo add`; `guides/contributing-rust.md` |

**Coverage:** 33 neutral invariants + R1 = 34 rows, each bound to ≥ 1 ADR and ≥ 1 module. No unbound
invariant.
