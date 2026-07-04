# Configuration schema and on-disk layout

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Normative. Defines the canonical on-disk layout, the `config.toml` schema, the manifest location,
> and the config-resolution precedence chain. Terms in **Reference Evidence** are non-normative.

This document is one of the four **canon** documents (with
[`02-command-surface.md`](02-command-surface.md), [`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md),
and [`README.md`](README.md)); every other document defers to the paths, roots, and precedence
defined here.

## 1. XDG roots

podbox MUST be fully XDG Base Directory compliant and user-home focused. It uses four roots, each
under a `podbox/` subdirectory:

| Role       | Root                                              | Holds                                                            |
| ---------- | ------------------------------------------------- | --------------------------------------------------------------- |
| **config** | `${XDG_CONFIG_HOME:-$HOME/.config}/podbox`        | user-owned source of truth: layers, manifests, images, config   |
| **cache**  | `${XDG_CACHE_HOME:-$HOME/.cache}/podbox`          | derived artifacts: composed `devcontainer.json`, build metadata |
| **state**  | `${XDG_STATE_HOME:-$HOME/.local/state}/podbox`    | workspace registry, sandbox identity mapping, locks, summaries  |
| **data**   | `${XDG_DATA_HOME:-$HOME/.local/share}/podbox`     | installed **seed sources** only (templates, schemas, docs)      |

Rules:

- All user-owned config, cache, state, and data MUST live under these roots unless a documented
  environment override is used.
- A single documented override (e.g. `PODBOX_HOME`) MAY redirect all four roots under one prefix for
  development/testing; individual per-root overrides, when defined, take precedence, and `XDG_*`
  fallbacks apply when no override is set.
- **Installed / data-root files are seed sources only.** Once user config exists, runtime operations
  read exclusively from the config/cache/state roots. Data-root files are never a runtime source of
  truth (see [`09-state-cache-and-data-model.md`](09-state-cache-and-data-model.md)).

Throughout this shelf, `$PODBOX_CONFIG_HOME` denotes the resolved **config** root above.

## 2. Config tree (normative shape)

The config root has exactly this shape:

```text
$PODBOX_CONFIG_HOME/
├── devcontainer/     # layer directories ONLY  (manifest files are invalid here)
├── manifests/        # composition manifests ONLY  (layer directories are invalid here)
├── images/           # user-owned image build definitions and assets
└── config.toml       # global config + project/workspace registry
```

Normative invariants (restated in [`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md)):

- `devcontainer/` contains **layer directories only**. Each layer is
  `devcontainer/<layer>/devcontainer.json` (plus any files that layer contributes). A `*.yaml`
  manifest placed here is invalid.
- `manifests/` contains **all composition manifests** and nothing else. A layer directory placed
  here is invalid.
- `images/` keeps its image-source role: `images/<name>/` holds an image's build definition and
  build context/assets.
- `config.toml` is the single global config file; it **replaces** the legacy `projects.yaml`.
- There is **no `default/devcontainer.json`** and no user-global default devcontainer file. It does
  not exist and is not part of resolution (see §5).

This is the primary departure from the legacy layout, where `devcontainer/` mixed layer directories
and `*.yaml` manifests together, a top-level `projects.yaml` held global config, and a
`default/devcontainer.json` provided a last-resort config. See §6 for the migration mapping.

## 3. `config.toml` schema

`config.toml` is a public **config API contract** and MAY be specified here; the parser/library that
reads it is an implementation detail and MUST NOT be mandated. TOML is the chosen **format family**;
any equivalent structured format could serve an implementation, but the documented surface is TOML.

Top-level tables (all keys optional unless marked required; unknown keys MUST be preserved and
ignored for forward compatibility, never rejected):

```toml
# --- global defaults -------------------------------------------------------
[defaults]
manifest       = "general"          # default manifest when a workspace selects none
reconcile      = "auto"             # default shell reconcile policy: auto | always | never
color          = "auto"             # auto | always | never
output         = "text"             # default human output; --json overrides per invocation

# --- runtime policy defaults ----------------------------------------------
[runtime]
# Selects a runtime backend by CAPABILITY PROFILE, not a hard-coded tool.
# The default profile is a KVM-class hardware-virtualization microVM (see 07-...).
profile        = "microvm"          # microvm | <other capability profiles an impl advertises>
memory_mib     = 4096               # advisory resource budget
cpus           = 2                  # advisory vCPU budget

# --- image policy defaults -------------------------------------------------
[images]
pull_policy    = "missing"          # missing | newer | always | never

# --- network policy defaults ----------------------------------------------
[network]
egress         = "deny"             # deny | allowlist  (default posture is default-deny)

# --- project / workspace registry -----------------------------------------
# One [[projects]] entry per registered project or work-clone. The registry
# maps a canonical workspace identity to its selected manifest, so `ws`
# commands resolve config without a per-workspace file.
[[projects]]
name            = "acme-api"        # canonical project name (identity key)
path            = "/home/u/src/acme-api"  # canonical workspace path
manifest        = "python"          # selected manifest name (from manifests/)
composed_digest = "sha256:…"        # last composed devcontainer.json digest (freshness bookkeeping)
image_digest    = "sha256:…"        # last built image digest (freshness bookkeeping)

# --- migration bookkeeping (non-normative) --------------------------------
[meta]
schema_version  = 1                 # config schema version, for safe migration
```

Field rules:

- `[[projects]]` entries key on a stable **workspace identity** (canonical name + path) so
  work-clones of the same repository keep separate identity and separate sandboxes (see
  [`05-workspace-lifecycle-and-shell-ux.md`](05-workspace-lifecycle-and-shell-ux.md)).
- `composed_digest` / `image_digest` are freshness bookkeeping consumed by composition
  ([`04-…`](04-manifest-and-composition-model.md)) and image builds
  ([`06-…`](06-image-builds-and-change-detection.md)); their exact storage form is not prescribed —
  only that enough is recorded to prove freshness.
- The annotated `config.toml` above is the authoritative field list: each key's default is shown
  inline, and the field rules in this section (together with the manifest and composition docs)
  define its meaning and constraints. Validation failures surface through `doctor` and exit code `2`
  (config syntax) or `4` (semantic validation) per [`02-command-surface.md`](02-command-surface.md).

## 4. Manifest schema and location

All manifests live under `$PODBOX_CONFIG_HOME/manifests/<name>.<ext>`. A manifest declares an ordered
layer composition plus optional runtime/network/image policy. The schema (carried forward from the
legacy compose schema, specified capability-neutrally) is:

- `layers` (**required**): a non-empty, ordered array of layer directory names under
  `devcontainer/`. Ordered base→leaf; the last entry is the **leaf**, preceding entries are
  **shared**. Composition semantics are defined in
  [`04-manifest-and-composition-model.md`](04-manifest-and-composition-model.md).
- `runtime` (optional): `runtime.profile` selects a runtime backend **by capability profile** (not a
  hard-coded tool enum; default profile is a KVM-class microVM), and `runtime.resources.{memory_mib,
  cpus}` give advisory budgets.
- `network` (optional): `network.allow` is a host allowlist whose entries union into the composed
  egress allowlist. Default egress posture is deny (see [`11-…`](11-invariants-and-guarantees.md)).

The manifest is also a public config contract; its concrete serialization (TOML/YAML/JSON family) is
a documented surface, not an implementation mandate. A layer directory **without** any manifest that
references it is not selectable and is not listed — **manifests are the selection API.**

## 5. Config resolution / precedence (revised)

podbox resolves the effective `devcontainer.json` for a workspace through this chain, highest
precedence first:

1. **CLI flag** — an explicit `--config <path>` on the invocation.
2. **Environment variable** — a documented config env var (e.g. `PODBOX_CONFIG`).
3. **Project registry** — the `[[projects]]` entry in `config.toml` for the workspace identity,
   resolving to that manifest's composed cache artifact.
4. **Local workspace file** — a workspace-local `devcontainer.json` (e.g. under `.devcontainer/`),
   when the workspace is permitted to carry one.
5. **Work-clone sibling discovery** — a sibling work-clone's resolved config, when enabled.

There is **no sixth level**: the legacy user-global `default/devcontainer.json` fallback is
**removed**. When nothing resolves, podbox MUST fail with a clear, actionable error and remedy —
"no config for `<workspace>`; run `podbox init` or pass `--config`" — and exit code `2` (see
[`02-command-surface.md`](02-command-surface.md)). Removing the default level is a deliberate change:
there is no implicit global config that silently applies to an unconfigured workspace.

## 6. Migration guidance (non-normative appendix)

Mapping from the legacy `dctl` layout to the podbox layout (descriptive intent only, not a
migration-tool spec):

| Legacy (`~/.config/dctl/`)                         | podbox (`$PODBOX_CONFIG_HOME/`)               |
| -------------------------------------------------- | --------------------------------------------- |
| `devcontainer/<layer>/`                            | `devcontainer/<layer>/` (unchanged role)      |
| `devcontainer/<name>.yaml` (manifests, mixed in)   | `manifests/<name>.<ext>` (moved out)          |
| `images/`                                          | `images/` (unchanged role)                    |
| `projects.yaml`                                    | `config.toml` (format + name change)          |
| `default/devcontainer.json`                        | **removed** (no equivalent; not resolved)     |

## Reference Evidence (non-normative)

- `lib/dctl/_lib/paths.sh` — legacy XDG roots and path printers; manifests currently resolved under
  `DCTL_DEVCONTAINER_DIR` (the mixed `devcontainer/` dir), confirming the mixing this layout
  separates.
- `lib/dctl/_lib/workspace/resolve_config.sh` — the legacy six-level resolution whose last level is
  `${DCTL_CONFIG_DIR}/default/devcontainer.json`; podbox drops that level.
- `schemas/compose.schema.yaml` — the legacy manifest contract carried forward here.
- `~/.dotfiles/dctl/.config/dctl/` — the live legacy tree the migration mapping replaces.
