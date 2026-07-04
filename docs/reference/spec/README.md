# podbox Specification

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

A complete, **technology-agnostic** specification for **podbox** — a command-line program that
provisions isolated, reproducible, per-workspace devcontainer sandboxes (for AI agents and general
development work) on a **KVM-class hardware-virtualization microVM** boundary.

This shelf specifies **what podbox is and does** — its command surface, configuration, composition
model, lifecycle, runtime requirements, diagnostics, and invariants. It is **not** an implementation:
it names no programming language, framework, parser library, or internal module layout, and no single
runtime tool is required. Anyone could implement podbox from this shelf in any technology.

## Purpose

podbox lets a developer or an autonomous agent open a hardened, disposable sandbox for a workspace,
work inside it, and reconcile it cheaply when configuration changes — with a real isolation boundary
between untrusted/agent-run code and the host. It is a from-scratch redesign of the `dctl` /
`devcontainerctl` tool, carrying forward that tool's proven composition model while fixing its config
organization, command surface, build reliability, and UX.

## Normative language

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are used per RFC 2119.
Every hard requirement is collected in [`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md)
and cross-referenced from the descriptive documents. A section labelled **Reference Evidence** is
non-normative: it cites the legacy `dctl` implementation or external material as evidence, never as a
requirement.

## Technology-neutrality rule

Requirements are phrased as **capability classes**, never as named tools:

- podbox MAY require, for example, "a KVM-class hardware-virtualization microVM isolation boundary",
  "an OCI-image-consuming, rootless-operable container runtime", "a native `devcontainer.json`
  interpreter driving lifecycle by argv vector", and "a TOML-family structured config format".
- Concrete tools (Podman, `crun --krun`, libkrun, Firecracker, Kata, gVisor, …) appear **only** as
  explicitly-marked, non-normative *illustrative examples*, cited to the backend reference shelf.
- The spec MUST NOT mandate an implementation language, framework, parser library, or module layout,
  and no single named tool may be the only compliant implementation.
- Public **file/config formats** (`config.toml`, manifest documents, `devcontainer.json`) are API
  contracts and MAY be specified; the code that reads them is not.

The isolation rationale itself (threat model, runtime catalog, the microVM decision, known runtime
pitfalls) is **not re-argued here** — it is cited to the vendor-neutral reference shelf at
`~/DocsNNotes/tech/infra/sandbox-isolation-backends/` (see [`07-…`](07-runtime-and-infrastructure.md)
and [Reference Evidence](#reference-evidence)).

## Core design decisions

The nine improvements this redesign encodes, each traceable in
[`TRACEABILITY.md`](TRACEABILITY.md):

1. **Config reorganization** — `devcontainer/` holds layer directories only; `manifests/` holds all
   manifests; `images/` unchanged; `config.toml` replaces `projects.yaml`; `default/devcontainer.json`
   removed. ([`03-…`](03-config-and-xdg-layout.md))
2. **`deploy` folded into `init`** — `init` produces a lean, clean baseline; no standalone `deploy`.
   ([`02-…`](02-command-surface.md))
3. **First-class `doctor`** — checks all program requirements with specific diagnostics.
   ([`08-…`](08-doctor-and-diagnostics.md))
4. **Post-step checks** — `init` (and other relevant commands) run `doctor` afterward.
5. **Automatic composition** — a final `devcontainer.json` is composed from layers + manifests.
   ([`04-…`](04-manifest-and-composition-model.md))
6. **Full XDG compliance**, user-home focused. ([`03-…`](03-config-and-xdg-layout.md))
7. **Reliable-by-default builds** — no silently-stale images.
   ([`06-…`](06-image-builds-and-change-detection.md))
8. **Reconcile-first lifecycle** — `workspace reconcile` (the former `reup`) is the everyday path.
   ([`05-…`](05-workspace-lifecycle-and-shell-ux.md))
9. **Shell-centric UX** — entering the sandbox (`shell`) is the central interface.

Two binding operator decisions frame the design: the command surface is **re-derived from
state-of-the-art CLI best practices** ([`01-…`](01-cli-design-research.md)), not inherited from
`dctl`; and this spec lives in-repo under `docs/reference/spec/` (promoted per the docs-design
Diataxis zones).

## Document map and reading order

Read in this order:

```text
00-goals-and-non-goals          → what podbox is; personas; goals/non-goals; principles
01-cli-design-research          → distilled SOTA CLI best practices → podbox conventions checklist
02-command-surface        (canon) → every command, flag, exit code, output contract
03-config-and-xdg-layout  (canon) → XDG roots, config tree, config.toml schema, precedence
04-manifest-and-composition-model → layers + manifests → composed devcontainer.json; merge; freshness
05-workspace-lifecycle-and-shell-ux → state/drift model; up/reconcile/shell/exec/down; hooks; pairing
06-image-builds-and-change-detection → source graph; reliable-by-default; freshness proof
07-runtime-and-infrastructure     → capability-class requirements; adapter concept; security posture
08-doctor-and-diagnostics         → the first-class doctor: checks, severity, output, post-step
09-state-cache-and-data-model     → config/cache/state/data inventory; concurrency; cleanup; recovery
10-errors-output-and-scriptability → stdout/stderr; JSON; prompting; color; verbosity; exit codes
11-invariants-and-guarantees (canon) → the collected normative MUST/SHOULD list + guarantees
12-glossary                        → terms of art
TRACEABILITY                       → requirement → section matrix (completeness gate)
```

The four **canon** documents (`02`, `03`, `11`, and this `README`) fix every cross-referenced name,
flag, path, and rule; all other documents defer to them.

## Relationship to dctl

podbox is a redesign of `dctl` / `devcontainerctl`. It keeps `dctl`'s layer+manifest composition
model and its microVM/rootless runtime posture, and deliberately departs on: config organization,
the `deploy`→`init` fold, a first-class `doctor`, reliable-by-default builds, reconcile-first
lifecycle, and a command surface re-derived from best practices. A reader coming from `dctl` should
consult the per-document **Reference Evidence** sections for the mapping.

## Reference Evidence

Non-normative sources cited across this shelf:

- Legacy implementation: `/workspaces/devcontainerctl/` — `CLAUDE.md`, `docs/ARCHITECTURE.md`,
  `schemas/compose.schema.yaml`, `lib/dctl/` (command tree, `_lib/paths.sh`,
  `_lib/workspace/resolve_config.sh`, `runtime/{common,krun}.sh`, `lifecycle.sh`).
- Live legacy config tree: `~/.dotfiles/dctl/.config/dctl/`.
- Real-usage evidence: `~/.dotfiles/kitty/.local/bin/kitty-dctl-pair`.
- **Backend reference shelf (authoritative for isolation rationale):**
  `~/DocsNNotes/tech/infra/sandbox-isolation-backends/` — threat model (`00-…`), runtime catalog
  (`10-…`), the libkrun decision (`20-…`), libkrun-vs-Firecracker (`30-…`), reference
  implementations (`40-…`), native-orchestration decision (`50-…`), operational notes / known bugs
  (`60-…`).
