# Manifest and composition model

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Normative. Defines how manifests and layer directories compose into the final `devcontainer.json`:
> layer location, the selectable-config rule, layer ordering, deterministic merge semantics, the
> derived composition artifact, and digest-based cache freshness. Sections labelled **Reference
> Evidence** are non-normative.
>
> This document defers to the canon documents ([`02-command-surface.md`](02-command-surface.md),
> [`03-config-and-xdg-layout.md`](03-config-and-xdg-layout.md),
> [`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md), [`README.md`](README.md)) for
> paths, flags, and rule wording. The hard rules elaborated here are collected as invariants **P1–P5**
> and **C7** in [`11-…`](11-invariants-and-guarantees.md).

## 1. Manifest purpose and location

A **manifest** is the unit of selection: it names an ordered composition of layers plus optional
runtime, network, and image policy, and it is what a user or a `[[projects]]` registry entry selects
by name (see [`03-…`](03-config-and-xdg-layout.md) §3–§4). Composition — merging the selected layers
into a single effective config — is a derived operation the manifest drives; a manifest never carries
the composed result itself.

Locations are fixed by [`03-…`](03-config-and-xdg-layout.md) §2 and MUST be honored:

- **Manifests** live under `$PODBOX_CONFIG_HOME/manifests/<name>.<ext>` and nowhere else. A manifest
  placed under `devcontainer/` is invalid (**C1**).
- **Layers** live under `$PODBOX_CONFIG_HOME/devcontainer/<layer>/devcontainer.json` (plus any files
  that layer contributes). A layer directory placed under `manifests/` is invalid (**C2**).

## 2. Selectable-config rule

Manifests are the **selection API**. A layer directory that is not referenced by any manifest MUST
NOT be selectable and MUST NOT be listed by `manifest list` or offered by `init` (**P1**).

- Selectability derives solely from manifest references, never from the mere presence of a
  `devcontainer/<layer>/` directory. Unreferenced layer directories MAY exist (as building blocks or
  work-in-progress) but are invisible to selection until a manifest references them.
- `manifest list` MUST enumerate manifests, not layer directories. `manifest show <name>` MUST report
  the ordered layers a manifest composes. `manifest validate <name>` MUST fail (exit `4`) when a
  referenced layer directory is missing or unreadable.

## 3. Layer ordering

A manifest's `layers` array is a **non-empty, ordered list** read **base → leaf** (**P1**):

- The **first** entry is the base layer; each subsequent entry is applied over the accumulated
  result; the **last** entry is the **leaf**.
- The **leaf** layer is **user-protected**: `init` and `workspace reconcile` MUST NOT overwrite a
  user-modified leaf when reconciling shared layers (**P2**, see
  [`05-…`](05-workspace-lifecycle-and-shell-ux.md)).
- All **preceding** (non-leaf) layers are **shared**: they are reconciled from seed sources and are
  expected to converge to their canonical content.

Ordering is significant for every last-wins merge rule below; a compliant implementation MUST apply
layers strictly in declared order.

## 4. Merge semantics

Composition MUST be **deterministic**: the same manifest and the same layer contents MUST always
produce a byte-stable composed `devcontainer.json` (**P3**, and Guarantee *Reproducibility* in
[`11-…`](11-invariants-and-guarantees.md) §9). The merge rules, carried forward capability-neutrally
from the legacy compose model, are:

| Key                                          | Merge rule                                                        |
| -------------------------------------------- | ----------------------------------------------------------------- |
| Scalar fields (e.g. `remoteUser`, `image`)   | **Last-wins** — the last layer that sets the key provides it.     |
| `mounts`                                      | **Concatenate** in layer order.                                   |
| `postCreateCommand`, `postStartCommand`      | **Merge by key** (object form); string/array forms preserved.     |
| `containerEnv`, `remoteEnv`                   | **Merge by key** — later layers override same-named keys.         |
| `runArgs`                                     | **First-class merge key** — combined across layers in order.      |
| `workspaceMount`, `workspaceFolder`          | **First-class merge keys** — last-wins per key.                   |
| `network.allow` (manifest)                   | **Union** into the composed egress allowlist (dedup, ordered).    |

Additional requirements:

- **Deterministic output ordering.** The composed artifact MUST serialize keys and merged
  collections in a stable, defined order so that identical inputs yield identical bytes (a
  precondition for the digest in §6). Ordering MUST NOT depend on filesystem iteration order,
  hash-map iteration order, or wall-clock time.
- **Per-fragment and composed-result validation.** Each layer fragment MUST be validated against the
  layer schema before merge, and the composed result MUST be validated against the composed-config
  schema after merge. Either failure MUST surface as exit code `4` (validation) per
  [`02-…`](02-command-surface.md) §5, naming the offending layer or key.
- **No merge tool is mandated.** These are semantic requirements on the *output*, not on the
  mechanism; the parser/merger implementation is unspecified (**N1**, **N3**).

## 5. Runtime profile selection

A manifest's optional `runtime` block selects a backend **by capability profile, not by a tool
enum**. `runtime.profile` names a capability class — the default profile is a **KVM-class
hardware-virtualization microVM** (see [`07-runtime-and-infrastructure.md`](07-runtime-and-infrastructure.md)
and **S1**) — and `runtime.resources.{memory_mib, cpus}` supply advisory budgets. A conforming
implementation MUST map a profile to any backend that satisfies the profile's capability class; it
MUST NOT require that `runtime.profile` name a specific product. `network.allow` entries union into
the composed egress allowlist under a default-deny posture (**S4**).

## 6. Final composition artifact

The final `devcontainer.json` is **automatically composed** from the selected manifest plus its
ordered layers (**P4**) and written to the **cache** root
(`${XDG_CACHE_HOME:-$HOME/.cache}/podbox`, see [`03-…`](03-config-and-xdg-layout.md) §1). It is the
runtime input consumed when a sandbox is created.

- The composed artifact is a **derived** artifact. It MUST NOT be hand-edited and MUST NOT be treated
  as a source of truth (**C7**). Sources of truth are the layer `devcontainer.json` files and the
  manifest; edits belong there and take effect on the next composition.
- `manifest compose <name>` renders the composed artifact into the cache on demand
  ([`02-…`](02-command-surface.md) §4.7); `workspace reconcile` recomposes as its first step
  ([`05-…`](05-workspace-lifecycle-and-shell-ux.md)).
- podbox MUST NOT write a workspace-local `.devcontainer/` as the composed output; the composed
  artifact lives in the cache root only.

## 7. Cache freshness

The composition cache MUST be keyed by a **digest** — not by modification time alone — computed over
at least (**P5**):

- the selected **manifest** content;
- the content of **every layer** the manifest references (base → leaf);
- the **layer schema version** and **composed-config schema version**;
- the **composition-rules version** (the version of the merge semantics in §4); and
- any **policy inputs** that affect the composed output (e.g. runtime profile, resource budgets, and
  network allowlist inputs).

Freshness rules:

- A cached composed artifact MAY be reused **only** when its recorded digest matches a freshly
  computed digest over the current inputs. This digest is the `composed_digest` recorded per project
  in `config.toml` ([`03-…`](03-config-and-xdg-layout.md) §3).
- If freshness cannot be proven — missing digest, unreadable inputs, or a changed digest — podbox
  MUST **recompose** or **fail closed**. Stale composed output MUST NOT be reused silently (**P5**;
  parallels the build contract in [`06-…`](06-image-builds-and-change-detection.md)).
- A digest that is **stronger than mtime-only** is required precisely because mtime is not a reliable
  change signal (touch-without-edit, clock skew, restored backups); the digest binds freshness to
  content and rule versions instead.

## Reference Evidence (non-normative)

- `schemas/compose.schema.yaml` — the legacy manifest contract this model carries forward: a required
  ordered `layers: [string]` array (last-wins scalars), an optional `runtime` block (`name` enum,
  historically only `krun`, plus schema-only `resources.memory_mib`/`resources.cpus`), and an
  optional `network.allow` host allowlist. podbox generalizes the tool-named `runtime.name` enum into
  the capability-profile `runtime.profile` of §5.
- `docs/ARCHITECTURE.md` (Merge behavior) — the legacy rules restated in §4: scalars last-wins,
  `mounts` concatenate, `postCreateCommand`/`containerEnv`/`remoteEnv` merge by key, and
  `runArgs`/`workspaceMount`/`workspaceFolder` as first-class merge keys.
- `lib/dctl/commands/init/_generate_cache.sh` — the legacy composed-cache generator (manifest lookup,
  layer validation, ordered JSON merge, runtime/network overlays) whose **mtime-based** freshness §7
  deliberately strengthens to a content-and-version digest.
