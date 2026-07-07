# Idiomatic Rust / CLI checklist

> Reference (lookup). The shippability gate for the podbox Rust binding, derived from cli-design
> `99-checklist.md` (+ cli-spec `06-testing-and-quality/`) and tailored to podbox's invariants. Every
> box is either **satisfied-by-ADR-N** (the binding decides it) or **waived-by-ADR-N** (podbox
> deliberately diverges from the reference default). No silent gaps. The binary is now implemented:
> a **checked** box has been verified against the current tree (the named module/test backs it); an
> **unchecked** box is a genuine remaining gap (e.g. rides the deferred
> [live guest transport](implementation-status.md), or is not yet wired) — not merely "unbuilt". The
> two **WAIVED** boxes stay unchecked by policy. Spot-check any checked box against the cited path.

## Architecture

- [x] `main` ≤ 120 LOC (parse → init logging → `AppContext` → dispatch → exit-map) — ADR-0002/0003.
- [x] Parse-shape vs runtime-shape are different types; projection at handler top — ADR-0003.
- [x] One `AppContext`, built once, no globals/thread-locals — ADR-0003.
- [x] Each subcommand is its own file on both sides (`cli/<name>` + `commands/<name>`) — ADR-0004.
- [x] No print statements outside `ui/`; grep-lint enforces it — ADR-0006, ADR-0013.
- [x] `domain/` has zero I/O; `adapters/` is the only outside-talk — ADR-0003.
- [x] No `lib.rs` public surface without a real second consumer — ADR-0002 (single binary crate).

## Logging & output

- [x] Human-UX: stdout = results, stderr = prompts/progress/warnings/errors — ADR-0006 (U4).
- [x] Color respects `NO_COLOR`/`FORCE_COLOR`/`--no-color` — ADR-0006 (U4); `anstream` backbone.
- [x] `--json` for machine output on pipeable commands — ADR-0005/0006 (U4).
- [x] Log sink `$XDG_STATE_HOME/podbox/podbox.log`; rotation/appender configured — ADR-0006 (C5).
- [x] Log records single-line, structured, no ANSI (`fmt().json().with_ansi(false)`); stable field names incl. `err.kind` — ADR-0006.
- [x] Log level reuses `RUST_LOG` (no `PODBOX_LOG`) — ADR-0006.

## Error messages

- [x] Each error has a stable `err.kind` — ADR-0005 (reuses doctor IDs / workspace identity / digest).
- [ ] **WAIVED — ADR-0005:** the reference says "every variant maps to a BSD sysexits code." podbox's
      canon `0`–`7` taxonomy (`02`/`10`/`11`) governs; sysexits are design influence only. Still: **no
      catch-all `_ => 1`**, and the exit-code matrix is unit-tested.
- [ ] User-facing errors carry what/where/why/hint — ADR-0005 (cli-design `02`).
- [ ] No `panic`/`unwrap`/`expect` outside `main`/tests/build/once-init — ADR-0013 (`09` lints).

## Configuration

- [x] Precedence `CLI > env > project > local workspace > (no user default)`, same for every key —
      ADR-0007 (C8). Note: podbox has **no user-default level** (C4) — a deliberate departure from the
      reference `user file` layer.
- [x] User config under `$XDG_CONFIG_HOME/podbox/` — ADR-0007 (C5).
- [x] Per-key source provenance — ADR-0007 (`figment`).
- [ ] **WAIVED — ADR-0007:** the reference says "unknown keys fail loudly." podbox `03` requires
      unknown `config.toml` keys be **preserved and ignored** (forward-compat); no `deny_unknown_fields`
      on the top-level Config. Flagged to docs-n-notes.
- [x] Env vars use `PODBOX_*` prefix — ADR-0007.
- [x] `config show` shows resolved values + sources — ADR-0007 (`02` command surface).

## Coding style

- [ ] Parse-don't-validate at every boundary — ADR-0003.
- [x] Newtypes for domain primitives (workspace id, digest, layer name, paths) — ADR-0003 (`08` naming).
- [ ] Static dispatch by default; dynamic only with justification — ADR-0003 (no `Box<dyn Command>`).
- [ ] Files ≤ ~400 LOC — cli-spec `09` (project convention).
- [ ] Module headers state purpose and non-purpose — cli-spec `00`/`08`.
- [ ] Strict lints project-level (`unsafe_code = forbid`, `unwrap_used`/`expect_used` warn, clippy
      pedantic/nursery) — ADR-0013 (cli-spec `09`).

## Designing for LLM agents

- [ ] `--help` is complete documentation for every flag/subcommand — ADR-0004.
- [ ] Output deterministic for the same input; no incidental timestamps — ADR-0009 (digest), U4.
- [x] `doctor` emits a structured health report — ADR-0011 (U3).
- [x] `init` reuses doctor checks as source of truth — ADR-0011 (U2).
- [x] Shell completions shipped (`clap_complete`) — ADR-0013 / reference page.
- [x] Man pages shipped via a subcommand (`clap_mangen`) — reference page.
- [x] `--help` and JSON snapshot-tested — ADR-0013.
- [x] Stable `err.kind` an agent can match — ADR-0005.

## Naming & docs

- [x] Least-public visibility default (`pub(crate)`) — cli-spec `08`.
- [x] `<Verb>Args` / `<Verb>Request` / `<Layer>Error` naming — cli-spec `08` (ADR-0003/0004).
- [ ] Every public / crate-public item has a doc comment — cli-spec `08`.
- [ ] Crate root has a module map linking to the spec — reference page `13`.
- [x] No `Manager`/`Helper`/`Utils`/`Handler`/`Wrapper` suffix soup — cli-spec `08`.

## Testing

- [x] One integration test file per subcommand (`tests/cmd_<name>.rs`) — ADR-0013.
- [x] Tests run in isolated tempdirs with cleared env — ADR-0013 (`tempfile`, `env_clear`).
- [x] Newtype constructors + parse→runtime projections unit-tested — ADR-0013.
- [x] Exit-code matrix locked by tests — ADR-0005, ADR-0013.
- [x] Structured output + `--help` snapshot-tested (`insta`) — ADR-0013.
- [x] Parallel-default runner (`cargo-nextest`) — ADR-0013.
- [ ] Property tests for parsers/digest/state machine (`proptest`) — ADR-0013. Partial: `domain/digest.rs`
      is property-tested; parsers and the state machine are not yet.
- [ ] Mutation score ≥ 60% on critical modules (`cargo-mutants`) — ADR-0013.

## Regression safeguards

- [ ] Restriction lints (`todo`, `dbg_macro`, `unwrap_used`, `panic`) enabled — ADR-0013 (`09`).
- [ ] Unused-dependency detection (`cargo-machete`) — ADR-0013.
- [ ] Binary-size baseline; CI flags growth (`cargo-bloat`) — ADR-0013.
- [ ] Snapshot updates require explicit review — ADR-0013.
- [x] Architectural boundary grep-lints (domain must not import adapters; no print outside `ui/`) —
      ADR-0003, ADR-0013.

## CI / shipping

- [x] Format-check, lint, test gate the PR — ADR-0013.
- [x] Toolchain pinned (`rust-toolchain.toml`) — ADR-0002 (Phase 8).
- [x] Lock file committed (`Cargo.lock`) — ADR-0012.
- [ ] Release artifacts include completions + man page — reference page.
- [x] `--version` includes git SHA + build date — `02` command surface.
- [ ] **Dependencies changed only via `cargo add`/`remove`/`update`** — ADR-0012 (R1); no hand-edited
      dependency lines.

## CLI-wrapper specifics

- [x] Typed command builder; no stringly-typed args — ADR-0008.
- [x] Args unit-testable as golden-argv snapshots before any subprocess — ADR-0008, ADR-0013.
- [ ] Signal forwarding (SIGINT/SIGTERM) to the child — ADR-0008.
- [x] Exit-code passthrough (child N→N; killed-by-signal→128+N) — ADR-0008.
- [ ] `--` sentinel passes through verbatim — ADR-0008.
- [ ] Inner binary resolution `$PODBOX_<TOOL>_BIN → config → PATH → bundled` (missing→127,
      not-exec→126) — ADR-0008.
