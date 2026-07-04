# Adding a subcommand

> Guide (task). The mechanical steps to add a podbox subcommand. This enacts the four-edit rule from
> [ADR-0004](../decisions/ADR-0004-four-edit-subcommand-pattern.md); it links to the module map in
> [`../reference/spec/13-rust-implementation-binding.md`](../reference/spec/13-rust-implementation-binding.md)
> rather than repeating it.

## The four edits (and only these four)

Adding a subcommand `<name>` touches exactly four files. If you find yourself editing a fifth to make
it appear, something is wrong.

1. **`src/cli/<name>.rs`** — define `<Name>Args` as a clap derive struct. Doc comments on fields
   become `--help` text (write them for the user). Parse-shape only: no logic.
2. **`src/cli/mod.rs`** — add a `<Name>` variant to the `Commands` enum wired to `<Name>Args`.
3. **`src/commands/<name>.rs`** — implement
   `pub fn run(ctx: &AppContext, args: <Name>Args) -> Result<(), AppError>`. First line projects args
   into a domain request: `let req = <Name>Request::from_cli(args)?;` (parse-don't-validate). `run` is
   sync; reach async via `ctx.runtime.block_on(...)`.
4. **`src/main.rs`** — add the dispatch arm `Commands::<Name>(args) => commands::<name>::run(&ctx, args)`.

The `Commands` enum is matched exhaustively (no `Box<dyn Command>` registry), so the compiler tells
you if the dispatch arm is missing.

## Then, before it ships

- Add **`tests/cmd_<name>.rs`** — one integration test per subcommand
  ([ADR-0013](../decisions/ADR-0013-testing-and-quality-stack.md)), run in an isolated tempdir with a
  cleared environment.
- Snapshot the `--help` output and any `--json` output with `insta`.
- If `<name>` fails in a new way, add an `AppError` variant with an explicit exit code in the `0`–`7`
  taxonomy ([ADR-0005](../decisions/ADR-0005-error-taxonomy-and-exit-codes.md)) — never a catch-all
  `_ => 1` — and extend the exit-code matrix test.
- Print only through `ui/`. The CI grep-lint rejects `println!`/`eprintln!` elsewhere.

## Conventions

- Naming: `<Name>Args` (parse-shape), `<Name>Request` (runtime-shape), `<Layer>Error` (adapters). No
  `Manager`/`Helper`/`Handler`/`Wrapper` suffixes.
- If `<name>` belongs to a noun group (e.g. `image build`), it becomes a variant of that group's
  subcommand enum in `cli/image.rs` + `commands/image.rs`, following the same four-edit shape one
  level down.
- If `<name>` wraps an external tool, build its argv with a typed builder in `adapters/` and add a
  golden-argv test ([ADR-0008](../decisions/ADR-0008-cli-wrapper-architecture.md)); never format a
  command string by hand.
