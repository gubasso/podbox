# Rust implementation binding

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Reference (lookup). The compact binding layer for the technology-neutral spec in this directory. It
> maps podbox's capability classes to concrete Rust modules and crate categories. It is **not** a
> second spec: the *why* lives in [`../../decisions/`](../../decisions/) (ADR-0001…0013) and the *what*
> lives in the neutral docs (`00`–`12`). Crate **versions** are produced by `cargo add` (ADR-0012),
> not pinned here.

## Scope

This page binds the neutral contract to an idiomatic Rust CLI, per
`docs-n-notes/tech/languages/rust/cli-spec` and `docs-n-notes/tech/programming/cli-design`. The neutral
invariants N1–N3 keep this layer separate from the product contract: nothing here changes *what podbox
is*, only *how this repo implements it*.

## Crate shape

- Single binary crate `podbox` (the existing tracked crate, **expanded not re-initialized** —
  ADR-0002). `edition = "2024"`, `rust-version = "1.85"`.
- Workspace split deferred until a cli-spec `01` trigger fires (2nd binary sharing ≥30% code,
  independently-publishable subsystem, >~10s warm `cargo check`, independent adapter dep trees; hard
  look ~8k LOC).
- `Cargo.lock` committed. All dependency changes via `cargo add`/`cargo remove`/`cargo update`
  (ADR-0012, [`../../guides/contributing-rust.md`](../../guides/contributing-rust.md)).

## Module map (cli-spec `00-directory-tree.md`)

| Path | Responsibility | Not this |
| ---- | -------------- | -------- |
| `main.rs` | ≤120 LOC: parse → init logging → build `AppContext` → dispatch → map exit | no logic, no I/O |
| `cli/` | clap derive structs only (`Cli`, `Commands`, `GlobalArgs`, `<Verb>Args`) | no business logic |
| `commands/` | one free-fn `run(&AppContext, <Verb>Args) -> Result<(), AppError>` per verb | no parsing, no direct external I/O |
| `domain/` | pure types + newtypes (`manifest`, `image`, `digest`, `workspace`, `state`, `doctor`, `network`) | **zero I/O** |
| `services/` | multi-adapter orchestration (`compose`, `image_build`) | no arg parsing |
| `adapters/` | one trait + default impl per external system (`runtime`, `devcontainer`, `state_store`), each own error enum | no domain rules |
| `config/` | `figment` layered loader (per-key provenance) | no globals |
| `context.rs` | `AppContext`, built once in `main` | no statics/thread-locals |
| `error.rs` / `exit.rs` | `AppError` + `exit_code() -> u8` (`0`–`7`) | no catch-all `_ => 1` |
| `logging.rs` | one `tracing` subscriber + appender → `$XDG_STATE_HOME/podbox/podbox.log` | not stdout |
| `ui/` | **the only place** with print statements (grep-linted) | not domain/adapters |
| `util/` | small shared helpers | not a junk drawer |
| `tests/cmd_<name>.rs` | one integration test per subcommand | — |

Supporting files: `deny.toml`, `rust-toolchain.toml`, `justfile`.

## Command surface → `cli/` + `commands/` (four-edit; ADR-0004)

| Command / group | Pair | Notes |
| --------------- | ---- | ----- |
| `shell` | `cli/shell.rs` + `commands/shell.rs` | top-level, first in help (U1, Thesis A) |
| `init` | `cli/init.rs` + `commands/init.rs` | lean; runs doctor after (U2) |
| `doctor` | `cli/doctor.rs` + `commands/doctor.rs` | check catalog (U3) |
| `status` | `cli/status.rs` + `commands/status.rs` | surfaces relaxations (S6) |
| `completion` | `cli/completion.rs` + `commands/completion.rs` | `clap_complete` |
| `version` / `help` | clap built-ins + `cli/mod.rs` | `--version` = SHA + date |
| `workspace {up,reconcile,shell,exec,down,status}` | `cli/workspace.rs` + `commands/workspace.rs` (subcommand enum) | `reconcile` first-class/idempotent (I2, Thesis B) |
| `image {build,list,inspect,prune}` | `cli/image.rs` + `commands/image.rs` | build freshness (B1/B2) |
| `manifest {list,show,validate,compose}` | `cli/manifest.rs` + `commands/manifest.rs` | composition (P1–P5) |
| `config {paths,show,validate,get,set,unset}` | `cli/config.rs` + `commands/config.rs` | precedence (C8) |
| `network {show,allow}` | `cli/network.rs` + `commands/network.rs` | allowlist union (S4) |

## Parse-shape → runtime-shape examples

- `shell`: `ShellArgs` → `Request::from_cli` resolves workspace identity (I1) → `state_store` lookup
  → `runtime.exec_shell` as an argv vector (X1).
- `workspace reconcile`: `ReconcileArgs` → recompute compose digest (P5) → `compose` protecting the
  leaf layer (P2) → idempotent apply (I2).
- `image build`: `BuildArgs` → source-graph digest (B1) → `image_build` → rebuild-or-fail (B2).
- `doctor`: `DoctorArgs` → run the check catalog (`domain/doctor.rs`) → structured report (U3),
  report-vs-fail policy.
- `manifest compose`: `ComposeArgs` → deterministic merge (P3) → write derived cache artifact (C7).
- `network allow`: `AllowArgs` → union into the allowlist (P3 `network.allow`) → recompose (S4).

## Capability class → module (see [`RUST-BINDING-TRACEABILITY.md`](RUST-BINDING-TRACEABILITY.md))

| Capability class | Owning module(s) |
| ---------------- | ---------------- |
| microVM isolation + hardening (S1–S3, S5) | `adapters/runtime.rs` |
| default-deny egress + allowlist (S4) | `domain/network.rs`, `adapters/runtime.rs` |
| native devcontainer interpret + argv lifecycle (X1, X2) | `adapters/devcontainer.rs`, `domain/manifest.rs` |
| deterministic composition + freshness (P1–P5, C7) | `services/compose.rs`, `domain/manifest.rs`, `domain/digest.rs` |
| image build change-detection (B1, B2) | `services/image_build.rs`, `domain/image.rs`, `domain/digest.rs` |
| TOML config + XDG precedence (C1–C8) | `config/`, `domain/` typed config |
| workspace identity + serialized state (I1, I3) | `domain/workspace.rs`, `adapters/state_store.rs` |
| diagnostics (U3, S6) | `commands/doctor.rs`, `domain/doctor.rs` |

## Dependency categories (versions come from `cargo add` — no hand-pinned versions)

| Category | Crates |
| -------- | ------ |
| CLI | `clap` (derive, env, wrap_help), `clap_complete`, `clap_mangen` |
| Errors | `thiserror` (2.x), `anyhow` |
| Logging | `tracing`, `tracing-subscriber` (env-filter, fmt), `tracing-appender` |
| Serialization / config | `serde` (derive), `serde_json`, `toml` (1.x), `figment` (env, toml) |
| Paths | `directories`, `camino` |
| Color output | `anstream` (1.0), `anstyle`, `owo-colors`, `supports-color` |
| Async runtime | `tokio` (rt, macros) — via `ctx.runtime.block_on`, never per-command |
| Dev-dependencies | `assert_cmd`, `predicates`, `insta`, `tempfile`, `proptest`, `trybuild` |
| CLI quality tools (`cargo install`) | `cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-machete`, `cargo-mutants`, `cargo-bloat` |
| Considered, not mandated | `miette` (rich user diagnostics), `etcetera` (`choose_app_strategy`) — see [`../external-facts.md`](../external-facts.md) |

Perishable version facts (thiserror 2, toml 1, anstream 1.0, …) are tracked with revalidation dates in
[`../external-facts.md`](../external-facts.md).
