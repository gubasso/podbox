# CLI design research

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> This is the **binding research document**: podbox's command surface
> ([`02-command-surface.md`](02-command-surface.md)) is designed *from* the conventions distilled here.
> Each sub-section distils state-of-the-art CLI practice, then states a **normative recommendation for
> podbox** (RFC-2119 keywords binding) with rationale and exemplar tools. The **sourcing** in §1 and
> the citations throughout are **Reference Evidence** (non-normative); the recommendations they justify
> are normative. The closing checklist (§10) is the contract `02-…` satisfies.

## 1. Sources studied

Primary style guides and reference manuals consulted while deriving podbox's surface. Web access was
available at authoring time; the four seed guides below were fetched and verified on the access date
shown. The broader tool references are cited to each tool's official CLI reference.

| # | Source                                             | Kind                | URL                                                        | Accessed   |
| - | -------------------------------------------------- | ------------------- | --------------------------------------------------------- | ---------- |
| 1 | Command Line Interface Guidelines (clig.dev)       | cross-tool guide    | `https://clig.dev/`                                        | 2026-07-04 |
| 2 | gcloud CLI reference & overview                    | vendor CLI manual   | `https://docs.cloud.google.com/sdk/gcloud`                | 2026-07-04 |
| 3 | Heroku CLI Style Guide                             | vendor style guide  | `https://devcenter.heroku.com/articles/cli-style-guide`   | 2026-07-04 |
| 4 | Fuchsia CLI tools guidelines (rubric)              | platform guide      | `https://fuchsia.dev/fuchsia-src/development/api/cli`      | 2026-07-04 |
| 5 | Git reference (`git help`, `git(1)`)               | tool CLI reference  | `https://git-scm.com/docs`                                | 2026-07-04 |
| 6 | Docker / Podman CLI reference                      | tool CLI reference  | `https://docs.docker.com/reference/cli/docker/`, `https://docs.podman.io/en/latest/Commands.html` | 2026-07-04 |
| 7 | kubectl reference                                  | tool CLI reference  | `https://kubernetes.io/docs/reference/kubectl/`           | 2026-07-04 |
| 8 | GitHub CLI (`gh`) manual                           | tool CLI reference  | `https://cli.github.com/manual/`                           | 2026-07-04 |
| 9 | Cargo & rustup books                               | tool CLI reference  | `https://doc.rust-lang.org/cargo/commands/`, `https://rust-lang.github.io/rustup/` | 2026-07-04 |

**Limitation note.** Where a guideline is a living document, the citations reflect the state observed on
the access date above; the *conventions* distilled are long-stable across editions, and the normative
recommendations do not depend on any single source's current wording.

## 2. Structure — noun-topic + verb-action, with the hot path promoted

**Recommendation.** podbox MUST use a consistent `podbox [global-flags] <noun-group> <verb> [args]`
grammar for resource operations, and SHOULD promote the single most-common workflow to a **top-level
verb** rather than burying it under a group. Groups are **nouns**; the operations under them are
**verbs**. Tool names MUST NOT be hyphenated compounds where a group+verb reads better (`workspace
down`, not `workspace-down`). A promoted top-level verb SHOULD also be reachable under its natural group
for discoverability.

**Rationale.** Noun-then-verb grouping keeps the mental model small and lets new operations slot in
without colliding, but a strict grammar taxes the *most-frequent* action; promoting it is the
established remedy. podbox's most-frequent action is entering the sandbox, so `shell` is top-level and
also available as `workspace shell` ([`02-…`](02-command-surface.md) §2, §4.1).

**Exemplars.** gcloud's `gcloud GROUP … COMMAND` hierarchy (`gcloud compute instances create`) and
Fuchsia's mandated "noun noun … verb" ordering model strict grouping. Heroku groups as
`topic:command` with plural-noun topics and verb commands, and *lists* objects at the topic root
(`heroku config`, never `config:list`). git, docker/podman, and kubectl all use `<tool> <group|noun>
<verb>`; git additionally promotes hot paths (`git commit`, `git status`) to top-level verbs beside
grouped plumbing — the exact pattern podbox applies to `shell`.

## 3. Flags and options — long/short, GNU `--flag=value`, `--no-` negation, terminator

**Recommendation.** podbox MUST prefer flags to positional arguments for anything non-obvious; every
flag MUST have a full `--long` form and MAY have a single-letter `-x` alias reserved for the most-common
options. It MUST accept GNU-style `--flag value` and `--flag=value`, MUST support a `--` terminator that
ends option parsing (so a user command can begin with `-`), and SHOULD provide `--no-<flag>` negation
for booleans that default on. Flags MUST be classified as **global** (valid on every command; see
[`02-…`](02-command-surface.md) §3) or **command-local**; globals MUST behave identically wherever they
appear. Ambiguous required positionals MUST be avoided in favor of named flags.

**Rationale.** Flags are self-documenting, order-independent, and completion-friendly, where positionals
force the reader to remember order. The `--` terminator is essential for podbox because `shell`/`exec`
forward a **user argv vector** that will legitimately contain leading dashes
([`02-…`](02-command-surface.md) §4.5). GNU `--flag=value` avoids ambiguity when a value looks like a
flag.

**Exemplars.** clig.dev: "Prefer flags to args," "have full-length versions of all flags," and support
`--` to end flags. The Heroku guide prefers required flags over positional arguments to remove ambiguity
(`heroku fork --from … --to …`). Fuchsia mandates GNU double-dash options "at least 3 characters long."
git (`git log -- <path>`), docker, and kubectl all honor `--` and long/short pairs; docker/kubectl use
`--flag=value` pervasively.

## 4. Help and discoverability — `-h`/`--help` everywhere, `help <cmd>`, usage-on-error, examples

**Recommendation.** podbox MUST accept `-h`/`--help` at **every** level (tool, group, command) and MUST
provide `help [command]`. Passing `--help` MUST override other flags and print help rather than act. On a
usage error (unknown flag, missing required input) it MUST print a **concise usage message on stderr**
plus the exit code for usage errors, and SHOULD point to `--help`. Help text SHOULD **lead with
examples** of common invocations. Shell **completion** MUST be a first-class, shippable feature
(`podbox completion <shell>`).

**Rationale.** Help is the primary documentation users actually read; putting `--help` everywhere and
leading with examples minimizes context-switching, and usage-on-error turns a mistake into a teachable
moment instead of a stack trace. Completion makes a noun-verb grammar tractable to type.

**Exemplars.** clig.dev: "you should be able to add `-h` to the end of anything," "Display concise help
text by default," and "Lead with examples." Fuchsia makes `--help` mandatory everywhere with a fixed
structure. gh, git, docker, kubectl, cargo, and rustup all ship completion generators and `<tool> help
<command>`; kubectl and gh are notable for example-led help pages.

## 5. Output and scriptability — human default to TTY, `--quiet`/`--verbose`, `--json`, stream split

**Recommendation.** podbox's default output MUST be human-readable and MAY adapt to whether stdout is a
TTY. It MUST send **primary data to stdout** and **all diagnostics — progress, logs, prompts, errors —
to stderr**. It MUST offer a stable machine format via `--json`, MUST offer `--quiet`/`-q` to suppress
non-essential output and `--verbose`/`-v` to raise diagnostic detail, and MUST NOT emit color or control
characters into `--json`. Machine output SHOULD be treated as a compatibility surface that does not
break scripts across releases without cause.

**Rationale.** The stream split lets `podbox … --json | jq` compose cleanly while progress still reaches
the user; mixing them corrupts pipelines. `--json` serves persona **P-AGENT**/**P-AUTO**
([`00-…`](00-goals-and-non-goals.md) §3) directly, removing any need to scrape human text
([`10-…`](10-errors-output-and-scriptability.md)).

**Exemplars.** clig.dev: "Human-readable output is paramount … Send output to `stdout` … Send messaging
to `stderr`," plus `--json`/`--plain`. Heroku: "The Heroku CLI is for humans before machines," data on
stdout, actions/spinners on stderr, and `--json` for `jq`. gcloud's `--format=json|yaml|value` and
`--verbosity` levels, and Fuchsia's `--machine` JSON, model the same split.

## 6. Exit-code discipline — `0` plus stable, documented non-zero classes

**Recommendation.** podbox MUST return `0` on success and a **stable, documented, non-zero** code per
failure class, and MUST keep those codes stable across releases. Distinct classes MUST be
distinguishable so automation can branch without parsing text. The taxonomy is fixed in
[`02-command-surface.md`](02-command-surface.md) §5 (`0` success; `1` general; `2` usage/config syntax;
`3` unmet host/runtime requirement; `4` validation; `5` workspace missing/not-running; `6` build
failed; `7` destructive action refused) and restated in
[`10-…`](10-errors-output-and-scriptability.md).

**Rationale.** Exit codes are the contract scripts branch on; a single opaque `1` forces text-scraping
and breaks silently when messages change. Distinct classes let a caller retry a transient build failure
(`6`) differently from a refused destructive action (`7`).

**Exemplars.** clig.dev and Fuchsia both mandate zero-on-success / non-zero-on-failure. git, docker, and
kubectl document distinct non-zero codes (e.g. git's `1` vs `128`; kubectl's usage vs runtime codes);
rustup and cargo return stable non-zero codes for build vs usage failures.

## 7. Config precedence — one documented chain, no hidden global default

**Recommendation.** podbox MUST resolve configuration through **one documented precedence chain**, with
an explicit override (a flag) at the top and no silently-applied global fallback at the bottom. The
canonical chain is fixed in [`03-config-and-xdg-layout.md`](03-config-and-xdg-layout.md) §5 — CLI flag →
environment variable → project registry → local workspace file → work-clone sibling discovery — and,
per improvement 1, MUST NOT include a user-global `default/devcontainer.json` level. When nothing
resolves, podbox MUST fail with an actionable error and remedy, never silently apply a default.

**Rationale.** A published "flags > env > project > user > system" ordering is the near-universal
convention; users reason about overrides only when the order is explicit and total. podbox deliberately
*drops* the bottom implicit-default level so an unconfigured workspace fails loudly instead of silently
inheriting a global config ([`00-…`](00-goals-and-non-goals.md) §6, "explicit precedence").

**Exemplars.** clig.dev's ordering (flags → env → project config → user config → system config) is the
template. gcloud layers flags over `--project`/config properties; git layers command-line over
`--file`/local/global/system config; docker and kubectl layer flags over env over config file
(`KUBECONFIG`, contexts). podbox adopts the *shape* while removing the implicit default.

## 8. Idempotence and destructive-action safety — bypassable prompts, `--yes`/`--force`, `--dry-run`

**Recommendation.** podbox MUST make mutating operations idempotent where feasible (re-running converges
rather than compounds). It MUST NOT *require* an interactive prompt: destructive verbs MUST confirm only
on an interactive TTY and MUST be bypassable with `--yes`/`--force`; noninteractive invocations lacking
the bypass MUST fail with the destructive-action-refused code rather than block
([`02-…`](02-command-surface.md) §5, code `7`). Every mutating command MUST support `--dry-run` that
describes the changes without making them. Reversibility expectations for each destructive verb MUST be
documented.

**Rationale.** Prompts protect humans but strand agents (**P-AGENT**) and scripts (**P-AUTO**); making
them bypassable and non-blocking serves both. `--dry-run` is the safety net for reconcile, `image
build`, `down`, and `prune`, letting a user preview before committing
([`05-…`](05-workspace-lifecycle-and-shell-ux.md), [`06-…`](06-image-builds-and-change-detection.md)).

**Exemplars.** clig.dev: "Never require a prompt," "Confirm before doing anything dangerous," `--no-input`,
and "`--dry-run` … describe the changes that would occur." gcloud's `--quiet`/`-q` "disables all
interactive prompts … useful for scripting." Heroku requires that "args or flags can always be provided
to bypass the prompt." kubectl's `--dry-run=client|server` and git's `--dry-run` model the preview
convention.

## 9. Self-check convention — `version` plus a `doctor`-style environment check

**Recommendation.** podbox MUST expose its version via both `podbox version` and `--version`, and MUST
provide a first-class **`doctor`** subcommand that checks **all** program requirements — host support,
hardware-virtualization access, permissions, runtime/build capability, filesystem/XDG layout, config and
manifest validity, network and credential policy — and reports each with a specific, actionable
remediation (never a generic "unavailable"). `doctor` MUST support `--json` and MUST exit non-zero when
any hard check fails. Relevant commands (notably `init`) MUST run `doctor` as a post-step
([`02-…`](02-command-surface.md) §4.3, [`08-…`](08-doctor-and-diagnostics.md); invariant **U3**).

**Rationale.** Environments drift and prerequisites are easy to miss; a dedicated self-check turns "it
doesn't work" into a precise, fixable diagnosis and is exactly persona **P-OPS**'s tool. Bundling it as a
post-step of `init` catches a broken host at setup time rather than at first `shell`.

**Exemplars.** `flutter doctor` and `brew doctor` are the canonical environment-check subcommands; `gh`
ships `gh auth status` as a scoped readiness check; rustup exposes `rustup check`/`rustup show`. podbox
generalizes these into a single required `doctor` covering the full requirement set.

## 10. Conventions podbox adopts (the checklist `02-…` satisfies)

The command surface in [`02-command-surface.md`](02-command-surface.md) MUST satisfy every item below;
each derives from a sub-section above and is verified in [`TRACEABILITY.md`](TRACEABILITY.md).

- [ ] **Structure.** Noun-group + verb grammar (`podbox <group> <verb>`); the hot path `shell` promoted
      to a top-level verb and also available as `workspace shell`. (§2 → `02-…` §2, §4.1)
- [ ] **Flags.** Long `--flag` for every option, `-x` aliases only for common ones; GNU
      `--flag`/`--flag=value`; `--no-` negation for on-by-default booleans; a `--` terminator before any
      forwarded user argv. (§3 → `02-…` §3, §4.5)
- [ ] **Global vs local flags.** A documented global-flag set (`--config`, `--manifest`, `--workspace`,
      `--json`, `--quiet`/`-q`, `--verbose`/`-v`, `--no-color`, `--yes`/`--force`, `--dry-run`,
      `--help`/`-h`, `--version`) valid on every command. (§3 → `02-…` §3)
- [ ] **Help & discoverability.** `-h`/`--help` at every level; `help [command]`; usage-on-error to
      stderr; example-led help; `podbox completion <shell>`. (§4 → `02-…` §4.9)
- [ ] **Output & scriptability.** Human default; stdout = data, stderr = diagnostics; `--json`;
      `--quiet`/`--verbose`; no color/control chars in JSON. (§5 → `02-…` §3, `10-…`)
- [ ] **Exit codes.** Stable `0`–`7` taxonomy, distinct per failure class, documented. (§6 → `02-…` §5)
- [ ] **Config precedence.** One documented chain (flag → env → registry → local → sibling), **no**
      user-global default level; loud failure when nothing resolves. (§7 → `03-…` §5)
- [ ] **Idempotence & safety.** Idempotent `init`/`reconcile`/`up`; destructive verbs (`down`, `prune`)
      confirm-or-`--force`, non-blocking noninteractively (exit `7`); `--dry-run` on every mutating
      command. (§8 → `02-…` §4.5, §4.6, §5)
- [ ] **Self-check.** `version`/`--version`; first-class `doctor` over all requirements with specific
      remediation and `--json`; `doctor` run as a post-step of `init`. (§9 → `02-…` §4.3, `08-…`)

## Reference Evidence (non-normative)

- The source table in §1 lists every guide and tool reference consulted, with access dates. All
  normative recommendations in §§2–9 are podbox's own; the citations show the state-of-the-art practice
  each recommendation distils, not a mandate to imitate any one tool.
- Legacy `dctl` command groups (`ws`, `image`, `init`, `deploy`, `config`, `doctor`, `net`) are cited as
  the baseline being re-derived, not as a source of conventions; see
  [`02-command-surface.md`](02-command-surface.md) Reference Evidence.
