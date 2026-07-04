# Goals and non-goals

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Normative where RFC-2119 keywords appear; otherwise framing. Establishes what podbox is, who it
> serves, the two theses the rest of the shelf is built around, and the boundaries of the design.
> Sections labelled **Reference Evidence** are non-normative.

This document orients every other document. The command surface
([`02-command-surface.md`](02-command-surface.md)), the on-disk layout
([`03-config-and-xdg-layout.md`](03-config-and-xdg-layout.md)), and the collected rules
([`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md)) exist to serve the goals stated
here. The isolation rationale is **not argued here** — it is cited to the backend reference shelf (see
[`07-runtime-and-infrastructure.md`](07-runtime-and-infrastructure.md) and
[Reference Evidence](#reference-evidence-non-normative)).

## 1. Problem statement

Developers and autonomous AI agents increasingly run untrusted or self-modifying code — package
installers, build steps, agent-authored scripts — that should not have unmediated access to the host.
The workflow needs a **secure, reproducible, per-workspace sandbox** that a person or an agent can open
in seconds, work inside, throw away, and recreate cheaply when its configuration changes, with a
**real isolation boundary** (hardware virtualization, not a shared kernel) between that code and the
host.

podbox provisions exactly that: it composes a devcontainer definition from reusable layers, builds the
backing image reliably, and runs each workspace as an isolated microVM sandbox that the user or agent
enters to work. It is a from-scratch redesign of the `dctl` / `devcontainerctl` tool, keeping that
tool's proven layer+manifest composition model and its microVM/rootless posture while fixing its
configuration organization, command surface, build reliability, and everyday UX.

## 2. Framing theses

Two theses frame the whole design and MUST be visible in every derived document:

- **Thesis A — the sandbox is the interface (improvement 9).** *Entering and working inside the
  sandbox is the central experience the rest of the design serves.* The shortest, most-used path is
  "open a shell in the workspace's sandbox." Every other capability — composition, image builds,
  diagnostics — exists to make that entry fast, safe, and reliable. Accordingly the surface promotes
  `podbox shell` to a top-level verb shown first in help ([`02-…`](02-command-surface.md) §2, §4.1;
  invariant **U1**).
- **Thesis B — reconcile-on-change is the everyday path (improvement 8).** *The common operation is
  not first-time bring-up; it is re-aligning a running sandbox after a configuration, manifest, image,
  or policy edit.* podbox therefore makes `workspace reconcile` a first-class, idempotent, routine
  operation — the formal successor to the legacy `ws reup` habit — cheap enough to run after any edit
  ([`02-…`](02-command-surface.md) §4.5, [`05-…`](05-workspace-lifecycle-and-shell-ux.md); invariant
  **I2**).

Everything below is consistent with these two theses; where a later document appears to contradict
them, that document is wrong and MUST be reconciled.

## 3. Primary users / personas

podbox MUST serve all five personas below; no persona may be designed out in favor of another.

- **P-DEV — the daily interactive developer who lives in `shell`.** Opens a sandbox for a repository,
  works inside it interactively for hours, edits shared/leaf layers, and expects re-entry and
  reconcile to be near-instant. This persona anchors Thesis A. (Evidence: paired-pane `shell` launch,
  [Reference Evidence](#reference-evidence-non-normative).)
- **P-AGENT — the headless AI agent.** Driven noninteractively; needs deterministic behavior, stable
  exit codes, `--json` output, no blocking prompts, and a hardware-virtualization boundary because it
  runs untrusted, self-authored commands. Every interactive path MUST have a noninteractive
  equivalent for this persona ([`10-…`](10-errors-output-and-scriptability.md)).
- **P-MAINT — the layer/manifest maintainer.** Curates shared layers and the manifests that compose
  them; needs `manifest validate`/`compose` and deterministic, reproducible composition so a change
  to a shared layer propagates predictably without destroying users' leaf layers
  ([`04-…`](04-manifest-and-composition-model.md)).
- **P-OPS — the operator diagnosing host readiness.** Sets up or troubleshoots a machine; needs a
  first-class `doctor` that checks **all** requirements (hardware-virt access, rootless mapping,
  cgroups, network backend, config/layout) with specific, actionable remediation, not a generic
  "unavailable" ([`08-…`](08-doctor-and-diagnostics.md)).
- **P-AUTO — the script/automation caller.** Wraps podbox in CI, launchers, or higher-level tooling;
  relies on the exit-code taxonomy, stable JSON, and stdout(data)/stderr(diagnostics) discipline, and
  MUST never have to scrape human-readable text ([`02-…`](02-command-surface.md) §5,
  [`10-…`](10-errors-output-and-scriptability.md)).

## 4. Goals

podbox MUST be designed to achieve the following. Each goal traces to the sections and invariants that
make it concrete.

- **G1 — Shell-first UX.** Entering the sandbox is the shortest, most prominent path (Thesis A;
  [`02-…`](02-command-surface.md) §4.1, [`05-…`](05-workspace-lifecycle-and-shell-ux.md); **U1**).
- **G2 — Reconcile-first lifecycle.** Re-aligning a sandbox after any edit is a first-class, idempotent
  routine (Thesis B; [`05-…`](05-workspace-lifecycle-and-shell-ux.md); **I2**).
- **G3 — Safe, lean `init`.** `init` produces a sane, clean, minimal starter baseline the user MAY
  adopt — never a bloated template set — is idempotent and non-destructive to user-modified leaf
  layers, and runs `doctor` afterward ([`02-…`](02-command-surface.md) §4.2; **U2**).
- **G4 — Reliable composition.** A final `devcontainer.json` is composed automatically and
  deterministically from ordered layers + a manifest, with digest-based freshness rather than
  mtime-only ([`04-…`](04-manifest-and-composition-model.md); **P3**–**P5**).
- **G5 — Reliable image builds.** A normal build detects source changes by default and never serves a
  stale image silently ([`06-…`](06-image-builds-and-change-detection.md); **B1**–**B2**).
- **G6 — Full XDG compliance.** Config, cache, state, and data live under XDG roots; installed files
  are seed sources only ([`03-…`](03-config-and-xdg-layout.md); **C5**–**C6**).
- **G7 — Explicit diagnostics.** `doctor` checks all requirements with specific remediation and is both
  standalone and a post-step ([`08-…`](08-doctor-and-diagnostics.md); **U3**).
- **G8 — Hardware-virtualization isolation.** The primary boundary is a KVM-class microVM, rootless, with
  default hardening ([`07-…`](07-runtime-and-infrastructure.md); **S1**–**S5**).
- **G9 — Project / work-clone isolation.** Each workspace, including parallel work-clones of one
  repository, keeps separate sandbox identity via a stable workspace label
  ([`05-…`](05-workspace-lifecycle-and-shell-ux.md); **I1**).
- **G10 — Scriptability.** Stable exit codes, `--json`, `--quiet`, and stdout/stderr discipline let
  agents and automation drive podbox without scraping ([`10-…`](10-errors-output-and-scriptability.md);
  **U4**).
- **G11 — Recoverability.** State is explicit enough that destructive commands are safe and partial
  failures (partial init, partial reconcile, failed build, stale pointers) are recoverable
  ([`09-…`](09-state-cache-and-data-model.md)).

## 5. Non-goals

podbox explicitly does **not** aim to do the following. These are boundaries, not deferrals.

- **NG1 — No mandated language/runtime tool.** The spec MUST NOT require any implementation language,
  framework, parser library, or single named runtime tool. Requirements are capability classes; named
  tools appear only as illustrative, non-normative examples (invariants **N1**–**N3**).
- **NG2 — No full upstream devcontainer feature parity.** podbox supports a defined subset of the
  `devcontainer.json` schema sufficient for its workflow; it does not promise to implement every
  upstream Dev Container feature, Feature-registry mechanism, or editor integration
  ([`04-…`](04-manifest-and-composition-model.md), [`07-…`](07-runtime-and-infrastructure.md)).
- **NG3 — No GUI.** podbox is a CLI. Any graphical or editor front-end is out of scope; the surface is
  designed for humans-at-a-terminal and machines, not a windowed UI.
- **NG4 — No cluster scheduler.** podbox provisions per-workspace sandboxes on a single host. It is not
  an orchestrator, scheduler, or multi-node control plane; it does not place, balance, or reconcile
  workloads across a fleet.
- **NG5 — No shared-kernel container as the primary boundary.** A namespace/shared-kernel container
  MUST NOT be the primary isolation boundary. Any such fallback is lower-assurance, explicitly labeled,
  and never silently substituted for the microVM boundary (invariant **S1**).
- **NG6 — No default live-mount of long-lived credentials.** podbox MUST NOT bind-mount long-lived host
  credential directories into the sandbox by default; credential access is scoped, short-lived, and
  auditable (invariant **S5**).

## 6. Design principles

The following principles guide every design choice; they are the rationale behind the invariants.

- **Human-first, machine-ready CLI.** Optimize for a human at a TTY by default, but make every path
  scriptable — a machine-readable form and a noninteractive form of everything
  ([`01-…`](01-cli-design-research.md), [`10-…`](10-errors-output-and-scriptability.md)).
- **Boring, explicit state.** Prefer an obvious, inspectable state model (`status`, drift, digests) over
  clever implicit behavior; a user or agent should always be able to ask "what is the current state and
  why" ([`05-…`](05-workspace-lifecycle-and-shell-ux.md),
  [`09-…`](09-state-cache-and-data-model.md)).
- **Explicit precedence.** Config resolution follows one documented chain with no hidden global default
  ([`03-…`](03-config-and-xdg-layout.md) §5; **C8**).
- **Idempotence where possible.** Re-running `init`, `reconcile`, or `up` converges to the same state
  rather than compounding side effects.
- **Safe destructive actions.** Destructive verbs confirm on a TTY, are bypassable with `--yes`/
  `--force`, and support `--dry-run` (**U5**).
- **Fail closed, never silently stale.** Unprovable freshness forces recomposition or rebuild, or an
  explicit failure, rather than reuse of stale output (**P5**, **B2**).
- **Local-first.** All state is on the user's machine under XDG roots; podbox requires no central
  service to function (**C5**).
- **Minimal starter baseline.** Seeded defaults are lean and adoptable, not prescriptive (**U2**).
- **Technology neutrality.** Every requirement is a capability class; no implementation stack is
  mandated (**N1**–**N3**).

## 7. Relationship to dctl (deliberate departures)

podbox is a redesign of `dctl` / `devcontainerctl`. It **keeps** dctl's layer+manifest composition
model, its per-workspace-label container identity, and its microVM/rootless runtime posture. It
**deliberately departs** in the following ways so a reader coming from dctl knows what changed and why:

| Area              | dctl (legacy)                                            | podbox (this spec)                                                      |
| ----------------- | -------------------------------------------------------- | ---------------------------------------------------------------------- |
| Config layout     | `devcontainer/` mixes layer dirs **and** `*.yaml`; `projects.yaml`; `default/devcontainer.json` | `devcontainer/` layers only; `manifests/` for all manifests; `config.toml`; **no** default file ([`03-…`](03-config-and-xdg-layout.md)) |
| Deploy vs init    | standalone `deploy` group (plan/apply/reset)             | **no public `deploy`**; folded into a lean `init` ([`02-…`](02-command-surface.md) §4.2) |
| Diagnostics       | `doctor` present but partial                             | first-class `doctor` over **all** requirements, run as a post-step ([`08-…`](08-doctor-and-diagnostics.md)) |
| Builds            | plain `image build` could mask source changes; needed `--full-rebuild` / manual `CACHEBUST` | reliable-by-default change detection; `--full-rebuild` only an escape hatch ([`06-…`](06-image-builds-and-change-detection.md)) |
| Everyday path     | `ws up` / `ws reup` mental model                         | `shell` central; `workspace reconcile` first-class (Theses A/B) |
| Surface origin    | inherited, ad hoc                                        | **re-derived from CLI best practices** ([`01-…`](01-cli-design-research.md)) |

The **isolation rationale** for choosing a hardware-virtualization microVM over a shared-kernel
container — the threat model, the runtime catalog, and the known runtime pitfalls — is **not restated
here.** It is cited to the vendor-neutral backend reference shelf and required as a capability class in
[`07-runtime-and-infrastructure.md`](07-runtime-and-infrastructure.md).

## Reference Evidence (non-normative)

- Legacy implementation: `/workspaces/devcontainerctl/` — `CLAUDE.md`, `docs/ARCHITECTURE.md`,
  `schemas/compose.schema.yaml`, `lib/dctl/` (command tree, `_lib/paths.sh`,
  `_lib/workspace/resolve_config.sh`, `runtime/{common,krun}.sh`, `lifecycle.sh`).
- Live legacy config tree: `~/.dotfiles/dctl/.config/dctl/` — evidence of the mixed layer/manifest
  directory and `projects.yaml` this redesign separates.
- Real-usage evidence for Thesis A (shell-central): `~/.dotfiles/kitty/.local/bin/kitty-dctl-pair`
  launches side-by-side panes each running `dctl ws shell`.
- **Backend reference shelf (authoritative for isolation rationale; cited, not restated):**
  `~/DocsNNotes/tech/infra/sandbox-isolation-backends/` — threat model and principles (`00-…`), runtime
  catalog (`10-…`), the libkrun-on-Linux decision (`20-…`), libkrun-vs-Firecracker (`30-…`), reference
  implementations (`40-…`), the native-orchestration decision (`50-…`), and operational notes / known
  bugs (`60-…`).
