# State, cache, and data model

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Normative. Inventories what podbox stores across the config, cache, state, and data roots plus the
> runtime-created sandbox resources, and defines concurrency, cleanup, and recovery behavior. Roots,
> paths, and command names are used verbatim from [`03-config-and-xdg-layout.md`](03-config-and-xdg-layout.md)
> and [`02-command-surface.md`](02-command-surface.md); hard rules are collected in
> [`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md). A section labelled
> **Reference Evidence** is non-normative.

## 1. State inventory

podbox distributes persistent data across the four XDG roots defined in
[`03-…` §1](03-config-and-xdg-layout.md), plus resources the runtime creates outside those roots:

| Store        | Root                       | Owner            | Nature                                            |
| ------------ | -------------------------- | ---------------- | ------------------------------------------------- |
| **config**   | `$PODBOX_CONFIG_HOME`      | user             | source of truth: layers, manifests, images, `config.toml` |
| **cache**    | `${XDG_CACHE_HOME}/podbox` | derived          | recomputable artifacts; safe to delete            |
| **state**    | `${XDG_STATE_HOME}/podbox` | runtime bookkeeping | operational records; deletable but costly to lose |
| **data**     | `${XDG_DATA_HOME}/podbox`  | installer        | read-only **seed sources** (invariant [C6](11-invariants-and-guarantees.md)) |
| **sandbox**  | runtime-managed            | runtime          | containers/microVM instances, runtime volumes, networks |

Rules:

- Each store MUST hold only its declared class of data. Config is the sole user source of truth; cache
  and state MUST be reconstructable or re-establishable without user re-authoring (cache fully; state
  by re-registration/re-reconcile).
- **Runtime-created sandbox resources** (the containers/microVMs, their ephemeral volumes and
  networks) live in the runtime's own storage, not under the XDG roots. podbox references them by the
  identity mapping in state (§3); it MUST NOT treat their presence as authoritative config.
- Deleting the entire cache root MUST be safe: podbox MUST recompute what it needs on next use.

## 2. Cache contents

The cache root (`${XDG_CACHE_HOME}/podbox`) holds only **derived, recomputable** artifacts:

- **Composed `devcontainer.json`** — the automatically generated composition artifact per manifest,
  the runtime input; a derived artifact, never hand-edited (invariant [C7](11-invariants-and-guarantees.md),
  see [`04-…`](04-manifest-and-composition-model.md)).
- **Composition metadata** — the digest inputs proving composition freshness (manifest content, layer
  content, schema version, composition-rules version, relevant policy inputs) per invariant
  [P5](11-invariants-and-guarantees.md).
- **Build-freshness metadata** — the source-graph digests proving an image was built from its current
  declared source (see [`06-…`](06-image-builds-and-change-detection.md), invariants
  [B1–B2](11-invariants-and-guarantees.md)).
- **Optional doctor cache** — memoized results of expensive `doctor` probes, when caching them is
  cheaper than re-probing; MUST be invalidated conservatively and MUST NOT mask a changed host
  condition (a stale-safe doctor cache is a `warning`, see [`08-…` `STATE-CACHE`](08-doctor-and-diagnostics.md)).
- **Transient session data** — scratch produced during a single command run.

Any cache entry whose freshness cannot be proven MUST be treated as absent and recomputed; podbox MUST
NOT serve a cache entry it cannot prove current (composition [P5](11-invariants-and-guarantees.md),
builds [B2](11-invariants-and-guarantees.md)).

## 3. State contents

The state root (`${XDG_STATE_HOME}/podbox`) holds **operational bookkeeping** that is expensive to
lose but is not user-authored config:

- **Workspace registry** — the canonical mapping of workspace identity (canonical path + label) to its
  selected manifest and resolution metadata. This is the persisted form of the `[[projects]]`
  registry described in [`03-…` §3](03-config-and-xdg-layout.md); it keys on stable workspace identity
  so work-clones stay separate (invariant [I1](11-invariants-and-guarantees.md)).
- **Sandbox identity mapping** — workspace identity → the runtime's sandbox/container identity, so
  `workspace` commands find the right sandbox by label.
- **Last-known runtime/container identity** — the last observed container/microVM handle and runtime
  profile for each workspace, used to detect drift and to locate resources for `down`.
- **Last reconcile summary** — the outcome of the most recent `workspace reconcile` (what recomposed,
  what rebuilt, hooks run), surfaced by `status`.
- **Failure records** — durable records of the last partial/failed `init`, `reconcile`, or `build`, so
  recovery (§6) can detect and resume/clean up.
- **Lock files (if used)** — per-workspace locks serializing state-mutating operations (§4).

State MUST be resilient to being partially rebuilt: a lost registry can be reconstructed by
re-registering workspaces; a lost sandbox mapping can be re-established by `reconcile`.

## 4. Data contents

The data root (`${XDG_DATA_HOME}/podbox`) holds **installed seed/reference assets only**: starter
templates, schemas, and docs shipped with an installation.

- Data-root files are **seed sources only**. Once user config exists, runtime operations MUST read
  exclusively from the config/cache/state roots and MUST NOT read the data root as a runtime source of
  truth (invariant [C6](11-invariants-and-guarantees.md)).
- `init` copies/reconciles from these seeds into user config ([`02-…` §4.2](02-command-surface.md));
  after that, the seed is inert for runtime purposes.
- The data root is treated as read-only by podbox; podbox MUST NOT write it during normal operation.

## 5. Concurrency

- **State-mutating operations on a single workspace MUST be serialized** (invariant
  [I3](11-invariants-and-guarantees.md)). Two commands that would both mutate the same workspace's
  state (e.g. concurrent `workspace reconcile`, or `workspace up` racing `workspace down`) MUST NOT
  run interleaved; the second MUST either wait or fail with a clear "operation in progress" message
  and a non-zero exit code, never corrupt shared state.
- Serialization SHOULD be scoped **per workspace identity**: operations on distinct workspaces
  (including separate work-clones of one repository) MUST be allowed to proceed in parallel.
- **Read-only commands** (`status`, `doctor`, `manifest list|show`, `config show|paths`,
  `image list|inspect`) MUST tolerate an in-progress mutating operation: they MUST report the current
  observable state with a clear "operation in progress" indication rather than blocking indefinitely
  or erroring.
- If lock files are used, a stale lock (owner process gone) MUST be detectable and recoverable so a
  crashed command does not permanently wedge a workspace (see §6).

## 6. Cleanup

Cleanup verbs and what each MAY remove vs. MUST preserve:

| Verb                            | MAY remove                                                                 | MUST preserve                                                        | Confirmation                          |
| ------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------- | ------------------------------------- |
| `workspace down`                | the workspace's sandbox/container instance and its ephemeral runtime resources | user config; the workspace registry entry; the composed cache; images | Required unless `--yes`/`--force`; supports `--dry-run` |
| `image prune`                   | unreferenced/stale image artifacts and their build-freshness metadata      | images still referenced by a registered workspace or manifest; user config | Required unless `--force`; supports `--dry-run` |
| cache cleanup (cache-root delete)| any derived artifact in the cache root (composed JSON, digests, doctor cache) | config, state, and data roots; runtime sandbox resources            | Not required (cache is recomputable), but SHOULD report what was cleared |

Rules:

- Destructive cleanup MUST be gated by confirmation on a TTY, bypassable with `--yes`/`--force`, and
  MUST support `--dry-run` (invariant [U5](11-invariants-and-guarantees.md)); refusing an unsafe
  action without the bypass flag exits `7` per [`02-…` §5](02-command-surface.md).
- No cleanup verb may delete **user config** (`$PODBOX_CONFIG_HOME`): `down` and `prune` operate on
  runtime resources and derived artifacts, never on the user's layers, manifests, or `config.toml`.
- `workspace down` MUST leave the workspace **re-creatable**: after `down`, a later `workspace up` or
  `podbox shell` reconstructs the sandbox from config with no user re-authoring.
- Cache cleanup MUST NOT remove state; removing derived artifacts forces recomputation (§2), it does
  not lose bookkeeping.

## 7. Recovery

podbox MUST behave predictably and fail safe when it finds inconsistent state:

- **Corrupt config** — an unparsable `config.toml` MUST fail with exit `2` (syntax) and a specific
  location/remedy; a schema-invalid but parsable config MUST fail affected operations with exit `4`
  and name the offending key. podbox MUST NOT silently ignore or overwrite a corrupt user config.
  Surfaced by `doctor` (`CFG-SYNTAX`/`CFG-SCHEMA`, see [`08-…`](08-doctor-and-diagnostics.md)).
- **Stale state pointing to missing runtime resources** — when the sandbox identity mapping references
  a container/microVM the runtime no longer has, podbox MUST treat the sandbox as **absent** (not
  running), report it via `status`, reconcile the stale pointer, and allow `workspace up`/`shell`/
  `reconcile` to recreate it. A dangling pointer MUST NOT block bring-up.
- **Partial `init`** — if `init` was interrupted mid-seed, a re-run MUST be **idempotent**: complete
  the seed, reconcile shared layers, and preserve any user-modified leaf layer
  ([`02-…` §4.2](02-command-surface.md)); it MUST NOT leave the config tree in a half-seeded state that
  future runs cannot repair.
- **Partial `reconcile`** — if `reconcile` failed after removing the old sandbox but before the new one
  was healthy, state MUST record the failure (§3), `status` MUST report the workspace as not-running/
  failed, and a re-run of `reconcile` MUST recover by recomposing and recreating from config. The
  user's leaf layer MUST remain intact throughout (invariant [P2](11-invariants-and-guarantees.md)).
- **Failed builds** — a failed `image build` MUST NOT leave behind build-freshness metadata that would
  let a later run treat the incomplete image as fresh (invariant [B2](11-invariants-and-guarantees.md));
  the next build MUST re-attempt from the declared source graph. `reconcile` MUST verify image
  freshness before creating a sandbox and MUST fail with remediation rather than proceed on a
  known-stale image ([`06-…`](06-image-builds-and-change-detection.md)).
- **Stale locks** — a lock whose owning process is gone MUST be reclaimable so a crash does not
  permanently serialize out future operations (§5).

## Reference Evidence (non-normative)

- `lib/dctl/_lib/paths.sh` — legacy XDG path printers; podbox separates the roles into distinct
  config/cache/state/data roots per [`03-…` §1](03-config-and-xdg-layout.md).
- `lib/dctl/commands/init/_generate_cache.sh` — legacy composed-`devcontainer.json` cache generation
  with mtime-based freshness; podbox strengthens freshness to digest-based (invariant
  [P5](11-invariants-and-guarantees.md)) and locates it in the cache root.
- `lib/dctl/commands/init/do.sh` — legacy `init` registers project defaults in `projects.yaml` and
  runs a smoke test; podbox replaces `projects.yaml` with the `config.toml` registry (§3) and runs
  `doctor` as the post-step ([`08-…` §6](08-doctor-and-diagnostics.md)).
- `lib/dctl/commands/ws/{down,status}.sh` and `deploy/{reset,apply}.sh` — legacy sandbox teardown and
  seed/reconcile behavior redistributed here into `workspace down` cleanup and the recovery model.
