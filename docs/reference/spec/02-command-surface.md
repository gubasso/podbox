# Command and API surface

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Normative. Defines every command, global flag, exit code, and output/prompting contract. This
> surface is **designed from** the CLI best-practices distilled in
> [`01-cli-design-research.md`](01-cli-design-research.md) — each choice below traces to that
> checklist — while preserving the fixed behaviors from
> [`00-goals-and-non-goals.md`](00-goals-and-non-goals.md).

One of the four **canon** documents. Names, flags, and exit codes defined here are authoritative;
every other document uses them verbatim.

## 1. CLI design research summary (see `01-…`)

The surface applies these distilled conventions (full sourcing in
[`01-cli-design-research.md`](01-cli-design-research.md)): noun-topic + verb-action grouping;
promotion of the single most-common workflow to a top-level verb; bypassable prompts; `--quiet` and
`--json` for scripting; strict stdout(data)/stderr(diagnostics) discipline; `--dry-run` for
mutating commands; confirmation or `--force`/`--yes` for destructive actions; `-h`/`--help` and
usage-on-error at every level; a documented, stable exit-code taxonomy; shell completion as a
first-class expectation.

## 2. Command-shape decision

- The single most-common workflow — **entering the sandbox to work** — is promoted to the top-level
  verb **`podbox shell`**, shown first in top-level help.
- Universal lifecycle verbs are top-level: **`init`**, **`doctor`**, **`status`**,
  **`version`/`help`**, and shell completion.
- Less-frequent resource operations live under noun groups: **`workspace`**, **`image`**,
  **`manifest`**, **`config`**, **`network`**.
- There is **no public `deploy` command.** Its former seed/reconcile behavior is folded into `init`
  (baseline) and redistributed to `config`/`manifest`/`image` (see §4.2).

Grammar: `podbox [global-flags] <top-verb | group <verb>> [args] [command-flags]`.

## 3. Global flags

Available on every command (a command MAY document additional local flags):

| Flag                     | Meaning                                                                    |
| ------------------------ | ------------------------------------------------------------------------- |
| `--config <path>`        | Explicit config path; highest precedence in resolution (see `03-…` §5).    |
| `--manifest <name>`      | Select a manifest by name from `manifests/`.                              |
| `--workspace <path>`     | Target a specific workspace instead of the current directory.             |
| `--json`                 | Emit stable machine-readable output on stdout (see `10-…`).               |
| `--quiet` / `-q`         | Suppress progress/non-essential output; script-friendly.                  |
| `--verbose` / `-v`       | Increase diagnostic detail on stderr.                                     |
| `--no-color`             | Disable ANSI color (also honored via environment convention).            |
| `--yes` / `--force`      | Bypass confirmation for destructive actions (only where a safeguard exists).|
| `--dry-run`              | Show what would change without mutating state (mutating commands).        |
| `--help` / `-h`          | Command/group/tool help with examples.                                    |
| `--version`              | Print version and exit.                                                   |

Prompts appear only on an interactive TTY; every prompt MUST be bypassable via a flag. Noninteractive
invocations that need missing input MUST fail with a clear message and a non-zero exit code rather
than block. Full output/prompting contract: [`10-errors-output-and-scriptability.md`](10-errors-output-and-scriptability.md).

## 4. Commands

For each command: purpose, synopsis, key flags, output/exit behavior, idempotence, destructive-action
safeguards, and any post-command `doctor` check. The executor MUST expand each into full prose when
authoring; the contract below is normative.

### 4.1 `shell` — the central UX

```text
podbox shell [--workspace <path>] [--manifest <name>] [--reconcile auto|always|never] [--] [command...]
```

- Ensure a sandbox exists for the current (or `--workspace`) workspace; reconcile per policy
  (`--reconcile`, default from `config.toml` `defaults.reconcile`, itself defaulting to `auto`);
  then either **enter an interactive login-capable shell** or, if `command...` is given after `--`,
  run that command **as an argv vector** in a login-capable context.
- The workspace is matched by a stable **workspace label** so work-clones keep separate identity.
- `shell` MUST bring the workspace up if it is absent (state this explicitly to the user).
- Documented first-class scenario: an external launcher composing multiple
  `podbox shell <agent-command>` invocations into side-by-side panes (the terminal pairing use case).
- Also available as `workspace shell` (grouped equivalent) for discoverability.
- Never emits a multi-line `sh -c` entrypoint (argv-vector only; see [`11-…`](11-invariants-and-guarantees.md)).

### 4.2 `init` — lean baseline (folds in `deploy`)

```text
podbox init [--manifest <name>] [--starter minimal|none] [--force] [--dry-run] [--json]
```

- Create or reconcile a **sane, lean, clean working baseline**: a minimal starter (one small example
  manifest + the layer directories it references + a starter `config.toml`) the user *may or may not*
  adopt as a reference. It MUST NOT install a bloated template set.
- Seeds from the installed **data-root seed sources** into user config. **Idempotent** and
  **non-destructive to user-modified leaf layers**: shared layers are reconciled, the leaf is
  protected.
- Never writes a workspace-local `.devcontainer/`; only user config under `$PODBOX_CONFIG_HOME`.
- **Runs `doctor` afterward** (unless `--dry-run`) and surfaces its diagnostics.
- The standalone `deploy` command is **removed**; its reconcile/seed behavior lives here (and in
  `config`/`manifest`/`image` for targeted operations).

### 4.3 `doctor` — first-class diagnostics

```text
podbox doctor [--scope all|host|runtime|config|workspace|images|network] [--json] [--quiet]
```

- Runs a full check of **all** program requirements with actionable, specific remediation (full
  catalog: [`08-doctor-and-diagnostics.md`](08-doctor-and-diagnostics.md)). Generic "unavailable"
  messages are non-compliant.
- Default human-readable grouped report; `--json` emits stable structured diagnostics (stable check
  IDs, status, message, evidence, remediation, docs reference).
- **Exit** non-zero (`3` for unmet host/runtime requirement, or the most specific applicable code)
  if any *hard* check fails; warnings do not fail unless a strict mode is requested.
- Runs as a **post-step** from `init` and from any command whose success depends on host readiness
  (first `workspace up`, `image build`). Post-step failure is *reported* without necessarily failing
  a primary command that already succeeded (report-vs-fail policy in [`08-…`](08-doctor-and-diagnostics.md)).

### 4.4 `status`

```text
podbox status [--workspace <path>] [--json]
```

Show the workspace's sandbox state, resolved config, image freshness, and **drift** state (see
[`05-…`](05-workspace-lifecycle-and-shell-ux.md)).

### 4.5 `workspace` group

```text
podbox workspace up        [--workspace <path>] [--manifest <name>]
podbox workspace reconcile [--workspace <path>] [--dry-run] [--yes]
podbox workspace shell     [ ... same as top-level `shell` ... ]
podbox workspace exec      [--workspace <path>] -- <argv...>
podbox workspace down      [--workspace <path>] [--force] [--dry-run]
podbox workspace status    [--workspace <path>] [--json]
```

- `up` — start a sandbox if absent from the resolved config; do **not** recreate an existing healthy
  sandbox unless explicitly requested. On detected drift of a running sandbox, report drift and point
  to `reconcile`.
- **`reconcile`** — the first-class recreate/reconcile path after any config, manifest, image, mount,
  or runtime-policy change: recompose, ensure image freshness, safely stop/remove the old sandbox,
  create a new one, run lifecycle hooks (argv-vector), report a summary. **Idempotent**, and **the
  recommended path after any config edit** — the formal replacement for the legacy `ws reup` habit.
  Cheap/fast enough to be routine.
- `exec` — run a noninteractive command **by argv vector** (`-- <argv...>`), never a shell blob.
- `down` — stop/remove the sandbox; **destructive**, gated by confirmation unless `--yes`/`--force`,
  supports `--dry-run`.

### 4.6 `image` group

```text
podbox image build   [<name>...] [--all] [--full-rebuild] [--pull-policy missing|newer|always|never] [--dry-run] [--json]
podbox image list    [--json]
podbox image inspect <name> [--json]
podbox image prune   [--dry-run] [--force]
```

- `build` — **reliable-by-default change detection**: MUST rebuild affected images whenever their
  declared source graph changed and MAY reuse a cached result only when freshness is provably
  unchanged; it MUST NOT serve stale output silently (full contract:
  [`06-image-builds-and-change-detection.md`](06-image-builds-and-change-detection.md)).
  `--full-rebuild` is an explicit force/no-cache escape hatch, **not** required for correctness.
  `--dry-run [--json]` reports what would rebuild and why.
- `prune` — **destructive**; confirmation unless `--force`; supports `--dry-run`.

### 4.7 `manifest` and `config` groups

```text
podbox manifest list | show <name> | validate <name> | compose <name>
podbox config   paths | show | validate | get <key> | set <key> <value> | unset <key>
```

- `manifest compose` renders the composed `devcontainer.json` for a manifest into the cache.
- Runtime operations read only user config (invariant); installed data-root files are seed-only.

### 4.8 `network` group

```text
podbox network show  [--workspace <path>] [--json]
podbox network allow <entry> [--workspace <path>]
```

- `show` — display the effective composed egress allowlist for a workspace (manifest `network.allow`
  entries unioned per [`04-…`](04-manifest-and-composition-model.md)); default egress posture is deny
  (invariant **S4**).
- `allow` — add an allowlist entry to the applicable manifest/config; a subsequent
  `workspace reconcile` applies it. This group only *edits and displays* policy; enforcement is
  in-guest (see [`07-…`](07-runtime-and-infrastructure.md)).

### 4.9 `version`, `help`, completion

```text
podbox version | --version
podbox help [command]
podbox completion <shell>
```

Shell completion is a first-class expectation; `help`/`version`/no-arg short-circuit before any group
loads.

## 5. Exit-code taxonomy (normative)

Stable and documented; restated as the canonical reference in
[`10-errors-output-and-scriptability.md`](10-errors-output-and-scriptability.md):

| Code | Meaning                                                        |
| ---- | ------------------------------------------------------------- |
| `0`  | Success.                                                       |
| `1`  | General/unexpected failure.                                   |
| `2`  | Usage or config **syntax** error (bad flags, unparsable config). |
| `3`  | Unmet **host/runtime requirement** (doctor hard-check failed).|
| `4`  | **Validation** failed (schema/semantic: manifest, layer refs, composed result). |
| `5`  | Workspace **not found / not running** when required.          |
| `6`  | **Build** failed.                                             |
| `7`  | Unsafe/**destructive action refused** (needs `--yes`/`--force`). |

## 6. Fixed-behavior checklist (traceability anchor)

Every item below MUST be observable in this surface; verified in
[`TRACEABILITY.md`](TRACEABILITY.md):

- [ ] No public `deploy`; its baseline behavior is in `init`.
- [ ] `shell` is the promoted, first-shown central UX.
- [ ] `init` produces a lean clean baseline and runs `doctor` afterward.
- [ ] `doctor` is first-class (standalone) and a post-step of relevant commands.
- [ ] `workspace reconcile` is the first-class common path (replacing `reup`).
- [ ] `exec` mandates argv-vector (`-- <argv...>`), never a shell blob.
- [ ] `image build` is reliable-by-default; `--full-rebuild` is only an escape hatch.
- [ ] Every interactive command has noninteractive flags; destructive verbs have safeguards.
- [ ] A stable, documented exit-code taxonomy exists.

## Reference Evidence (non-normative)

- Legacy command groups `ws`, `image`, `init`, `deploy`, `config`, `test`, `doctor`, `net`
  (`lib/dctl/commands/`); `deploy` verbs `all|apply|list|plan|reset`; `ws` verbs
  `up|reup|shell|exec|run|down|status`; `image` verbs `build|list` with the `--full-rebuild` gap
  (`lib/dctl/commands/image/build.sh`).
- Real usage centers on `dctl ws shell`, launched in paired panes by
  `~/.dotfiles/kitty/.local/bin/kitty-dctl-pair`.
