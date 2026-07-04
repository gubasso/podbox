# ADR-0004: Four-edit subcommand pattern and command-surface mapping

## Context and Problem Statement

The `02` command surface is broad (top verbs plus five noun groups). Adding or changing a subcommand
must be mechanical and hard to get subtly wrong, and `shell` / `workspace reconcile` must stay
prominent (U1 / I2, both theses).

## Considered Options

- A `Box<dyn Command>` runtime registry.
- A macro-generated command table.
- The cli-spec `02-subcommand-pattern.md` four-edit rule with an exhaustive `Commands` enum.

## Decision Outcome

Chosen option: **four-edit rule** — adding a subcommand touches exactly (1) `src/cli/<name>.rs`
(`<Verb>Args`), (2) `src/cli/mod.rs` (`Commands` variant), (3) `src/commands/<name>.rs` (`run`
handler), (4) `src/main.rs` (dispatch arm). **No `Box<dyn Command>` registry** — an exhaustive match
preserves compile-time completeness and clap help. `run` is sync; async goes through
`ctx.runtime.block_on(...)`, never a per-command runtime. `shell` is top-level and shown first in help
(U1, Thesis A); `workspace reconcile` is first-class and idempotent (I2, Thesis B).

## Consequences

- Good: every `02` verb maps to one `cli`/`commands` pair; the compiler flags a missing dispatch arm;
  help output and the neutral contract stay in lockstep.
- Bad: four edit sites per command — accepted; they are mechanical and covered by the guide.
- Enacted by: the command→pair table in `13-rust-implementation-binding.md` covering `shell`, `init`,
  `doctor`, `status`, `workspace {up,reconcile,shell,exec,down,status}`, `image
  {build,list,inspect,prune}`, `manifest {list,show,validate,compose}`, `config
  {paths,show,validate,get,set,unset}`, `network {show,allow}`, `completion`, `version`/`help`; guide
  `docs/guides/adding-a-subcommand.md`.

## Status

Accepted
