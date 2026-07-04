# Glossary

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Non-normative. Defines every term of art used across this shelf so the documents read consistently.
> Definitions restate — never override — the normative rules; where a term carries a hard requirement,
> the binding statement lives in the linked document and in
> [`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md). Terms are listed
> alphabetically; the linked document is the defining source for each.

### argv-vector exec

Running a command inside the sandbox as an explicit argument vector (a list of arguments) rather than
as a single multi-line shell string passed to `sh -c`. podbox MUST drive lifecycle hooks and `exec`
this way, because the microVM runtime mangles embedded newlines in a shell-blob entrypoint (invariant
**X1**). See [`05-…`](05-workspace-lifecycle-and-shell-ux.md), [`07-…`](07-runtime-and-infrastructure.md).

### cache root

The XDG **cache** directory `${XDG_CACHE_HOME:-$HOME/.cache}/podbox`, holding derived artifacts —
the composed `devcontainer.json`, composition metadata, and build-freshness metadata. Cache contents
are regenerable from config and MUST NOT be a source of truth. See
[`03-…`](03-config-and-xdg-layout.md) §1, [`09-…`](09-state-cache-and-data-model.md).

### capability profile

A named set of runtime capabilities a manifest selects via `runtime.profile` (default `microvm`),
identifying a runtime backend **by the guarantees it must provide** rather than by a hard-coded tool.
It is the manifest-level expression of the technology-neutrality rule. See
[`03-…`](03-config-and-xdg-layout.md) §3–§4, [`07-…`](07-runtime-and-infrastructure.md).

### capability class / runtime capability

A requirement phrased as *what must be possible* (e.g. "a KVM-class hardware-virtualization microVM
isolation boundary", "an OCI-image-consuming, rootless-operable container runtime") instead of a named
tool. All podbox requirements are stated as capability classes; concrete tools appear only as
non-normative illustrative examples (invariants **N1–N2**). See [`07-…`](07-runtime-and-infrastructure.md).

### composed devcontainer.json

The final, automatically generated `devcontainer.json` produced by composing a manifest's ordered
layers. It is a **derived** artifact stored in the cache root and consumed as the runtime input; it
MUST NOT be hand-edited or treated as source (invariant **C7**). See
[`04-…`](04-manifest-and-composition-model.md).

### composition

The deterministic process of merging a manifest's ordered layers (base→leaf) into the composed
`devcontainer.json`: scalars last-wins, `mounts` concatenate, key-merged maps
(`postCreateCommand`/`containerEnv`/`remoteEnv`), first-class merge keys
(`runArgs`/`workspaceMount`/`workspaceFolder`), and unioned `network.allow`. See
[`04-…`](04-manifest-and-composition-model.md) (invariant **P3**).

### config root

The XDG **config** directory `${XDG_CONFIG_HOME:-$HOME/.config}/podbox`, denoted `$PODBOX_CONFIG_HOME`
throughout this shelf. It is the user-owned source of truth and contains exactly `devcontainer/`
(layers), `manifests/`, `images/`, and `config.toml`. See [`03-…`](03-config-and-xdg-layout.md) §1–§2.

### config.toml

The single global configuration file at the config root, holding global defaults plus the project
registry; it replaces the legacy `projects.yaml` (invariant **C3**). TOML is the documented format
family, not an implementation mandate. See [`03-…`](03-config-and-xdg-layout.md) §3.

### data root

The XDG **data** directory `${XDG_DATA_HOME:-$HOME/.local/share}/podbox`, holding installed **seed
sources** only (templates, schemas, docs). Once user config exists, data-root files are never a
runtime source of truth (invariant **C6**). See [`03-…`](03-config-and-xdg-layout.md) §1.

### doctor

The first-class diagnostics command that checks **all** program requirements — host, runtime, config,
workspace, images, network — and reports specific, actionable remediation. It runs both standalone and
as a post-step of `init` and other host-dependent commands (invariants **U3**, **U2**). See
[`02-…`](02-command-surface.md) §4.3, [`08-…`](08-doctor-and-diagnostics.md).

### drift

A detected mismatch between a running sandbox and its declared inputs — a change in manifest digest,
layer digest, local-override digest, image-source digest, runtime policy, mount policy, network policy,
or config schema version. Drift is surfaced by `status` and resolved by `reconcile`. See
[`05-…`](05-workspace-lifecycle-and-shell-ux.md).

### egress allowlist

The composed set of outbound network destinations a sandbox may reach, formed by unioning each
manifest's `network.allow` entries. The default egress posture is **deny**; outbound access is
governed by this allowlist and enforced in-guest (invariant **S4**). See
[`03-…`](03-config-and-xdg-layout.md) §4, [`07-…`](07-runtime-and-infrastructure.md).

### exec

The `workspace exec` verb: runs a noninteractive command inside an existing sandbox **by argv vector**
(`-- <argv...>`), never as a shell blob. Requires a running sandbox (else exit `5`). See
[`02-…`](02-command-surface.md) §4.5, and *argv-vector exec* above.

### freshness proof

Evidence that a cached artifact — a composed `devcontainer.json` or a built image — was produced from
the *current* declared inputs. Composition keys freshness on a digest over manifest, layers, schema
version, composition-rules version, and policy inputs; image builds key it on the image source graph.
A cached result MAY be reused only when its freshness proof succeeds; otherwise podbox recomposes,
rebuilds, or fails closed (invariants **P5**, **B1–B2**). See
[`04-…`](04-manifest-and-composition-model.md), [`06-…`](06-image-builds-and-change-detection.md).

### image

An OCI-format container image built from a definition under `$PODBOX_CONFIG_HOME/images/<name>/` (or
referenced externally by the composed config), used as the sandbox's filesystem. podbox consumes
images through a capability-class container runtime; the builder is not mandated. See
[`06-…`](06-image-builds-and-change-detection.md).

### image source graph

The complete set of inputs that determine an image's contents: the image definition file, every file
in the declared build context, included scripts/assets, output-affecting build args, base-image
identity / pull-policy result, the podbox schema/build-policy version, and any baked-in identity
inputs. An image MUST NOT be served as fresh unless provably built from the current source graph
(invariant **B1**). See [`06-…`](06-image-builds-and-change-detection.md).

### init

The command that creates or reconciles a **lean, clean baseline** — a minimal starter manifest plus
the layer directories it references and a starter `config.toml` — seeding from the data root into user
config. It is idempotent, non-destructive to user-modified leaf layers, never writes a workspace-local
`.devcontainer/`, and runs `doctor` afterward. It folds in the removed `deploy` command (invariant
**U2**). See [`02-…`](02-command-surface.md) §4.2.

### layer

A directory `devcontainer/<layer>/devcontainer.json` (plus any files it contributes) that provides one
fragment of a composed configuration. Layers live under the config root's `devcontainer/` directory
only; a layer is selectable solely by being referenced from a manifest. See
[`03-…`](03-config-and-xdg-layout.md) §2, [`04-…`](04-manifest-and-composition-model.md).

### leaf layer

The **last** layer in a manifest's ordered `layers` list. On `reconcile` the leaf is **user-protected**
— reconciling shared layers never destroys the user's leaf (invariant **P2**, the non-destructive
reconcile guarantee). See [`04-…`](04-manifest-and-composition-model.md).

### manifest

A document under `$PODBOX_CONFIG_HOME/manifests/<name>.<ext>` that declares an ordered `layers`
composition plus optional runtime/network/image policy. **Manifests are the selection API**: a layer
directory not referenced by any manifest is neither selectable nor listed (invariants **P1**, **C2**).
See [`03-…`](03-config-and-xdg-layout.md) §4, [`04-…`](04-manifest-and-composition-model.md).

### microVM / hardware-virtualization boundary

The primary isolation boundary podbox requires: a KVM-class hardware-virtualization microVM (a
lightweight guest with its own kernel), **not** a shared-kernel namespace container. Any
non-hardware-virt fallback is lower-assurance, explicitly labeled, and never silently treated as
equivalent (invariant **S1**). The rationale is cited to the backend reference shelf, not re-argued.
See [`07-…`](07-runtime-and-infrastructure.md).

### podbox

The command-line program specified by this shelf: it provisions isolated, reproducible, per-workspace
devcontainer sandboxes on a microVM boundary for AI agents and developers. It is a technology-agnostic
redesign of the legacy `dctl` / `devcontainerctl` tool. See [`README.md`](README.md),
[`00-…`](00-goals-and-non-goals.md).

### project registry

The `[[projects]]` entries in `config.toml` mapping each registered workspace identity to its selected
manifest and freshness bookkeeping (`composed_digest`, `image_digest`). It is precedence level 3 of
config resolution, letting `workspace` commands resolve config without a per-workspace file. See
[`03-…`](03-config-and-xdg-layout.md) §3, §5.

### reconcile

The first-class `workspace reconcile` verb: recompose, ensure image freshness, safely stop/remove the
old sandbox, create a new one, run lifecycle hooks by argv vector, and report a summary. It is
idempotent and the **recommended path after any config change** — the formal replacement for the
legacy `ws reup` habit (invariant **I2**). See [`02-…`](02-command-surface.md) §4.5,
[`05-…`](05-workspace-lifecycle-and-shell-ux.md).

### runtime adapter

The conceptual external contract between podbox and a runtime backend, expressed as capability
operations (`run`, `exec`, `list`, `remove`, `build`, `inspect`, and logs/copy as needed). It names no
implementation module and mandates no specific tool; any backend satisfying the operations and the
required capability classes conforms. See [`07-…`](07-runtime-and-infrastructure.md).

### sandbox

The concrete, disposable, per-workspace isolated environment podbox creates and enters — a running
guest on the microVM boundary, built from the composed `devcontainer.json` and its image. Each sandbox
is keyed by workspace identity so work-clones stay separate. See
[`05-…`](05-workspace-lifecycle-and-shell-ux.md).

### scoped credential

A short-lived, narrowly-scoped, auditable credential injected into a sandbox on demand. Long-lived host
credential directories MUST NOT be live-mounted, and a container control socket MUST NOT be live-mounted
(invariant **S5**). See [`07-…`](07-runtime-and-infrastructure.md).

### seed source

An installed file under the data root used only to initialize (seed) user config. Seed sources are
never read at runtime once user config exists (invariant **C6**); `init` copies from them into the
config root. See [`03-…`](03-config-and-xdg-layout.md) §1, [`02-…`](02-command-surface.md) §4.2.

### shared layer

Any layer in a manifest that precedes the leaf. Shared layers are **reconciled** (updated) on
`reconcile`, in contrast to the protected leaf layer (invariant **P2**). See
[`04-…`](04-manifest-and-composition-model.md).

### shell

The promoted top-level `shell` command and the central UX: ensure a sandbox exists, reconcile per
policy, then enter an interactive login-capable shell — or run a supplied command as an argv vector.
Entering the sandbox is the interface the rest of the design serves (invariant **U1**). Also available
as `workspace shell`. See [`02-…`](02-command-surface.md) §4.1,
[`05-…`](05-workspace-lifecycle-and-shell-ux.md).

### starter baseline

The minimal, non-bloated set `init` seeds — one small example manifest, the layer directories it
references, and a starter `config.toml` — offered as a reference the user *may or may not* adopt.
Selected by `--starter minimal|none`. See [`02-…`](02-command-surface.md) §4.2.

### state root

The XDG **state** directory `${XDG_STATE_HOME:-$HOME/.local/state}/podbox`, holding the workspace
registry runtime state, sandbox identity mapping, locks, reconcile summaries, and failure records. See
[`03-…`](03-config-and-xdg-layout.md) §1, [`09-…`](09-state-cache-and-data-model.md).

### up

The `workspace up` verb: start a sandbox if one is absent for the resolved config, and do **not**
recreate an existing healthy sandbox unless explicitly asked. On detected drift of a running sandbox it
reports the drift and points to `reconcile`. See [`02-…`](02-command-surface.md) §4.5,
[`05-…`](05-workspace-lifecycle-and-shell-ux.md).

### workspace

A single project working tree that podbox provisions a sandbox for, identified by its canonical path
and stable identity. Distinct work-clones of the same repository are distinct workspaces. See
[`05-…`](05-workspace-lifecycle-and-shell-ux.md).

### workspace identity / workspace label

The stable key — canonical workspace name plus path — that ties a workspace to its `[[projects]]`
registry entry and its sandbox, surfaced to the runtime as a **workspace label** so parallel
work-clones never collide (invariant **I1**). See [`03-…`](03-config-and-xdg-layout.md) §3,
[`05-…`](05-workspace-lifecycle-and-shell-ux.md).

### XDG

The XDG Base Directory specification podbox follows fully, placing config, cache, state, and data under
their respective `${XDG_*_HOME}` roots (each in a `podbox/` subdirectory) with documented `$HOME`
fallbacks (invariant **C5**). See [`03-…`](03-config-and-xdg-layout.md) §1.
