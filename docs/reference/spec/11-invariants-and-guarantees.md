# Invariants and guarantees

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Normative. The single collected list of every hard rule in this specification, each cross-referenced
> to the document that elaborates it. RFC-2119 keywords (MUST / MUST NOT / SHOULD / MAY) are binding.

One of the four **canon** documents. Where a descriptive document and this list appear to differ, the
descriptive document is wrong and MUST be reconciled to this list.

## 1. Configuration invariants

- **C1.** `devcontainer/` MUST contain layer directories only; a manifest file there is invalid.
  (`03-…` §2)
- **C2.** `manifests/` MUST contain all composition manifests and nothing else; a layer directory
  there is invalid. (`03-…` §2, §4)
- **C3.** The global config MUST be `config.toml`; the legacy `projects.yaml` MUST NOT be used.
  (`03-…` §3)
- **C4.** No `default/devcontainer.json` exists; there MUST be no user-global default devcontainer,
  and resolution MUST NOT include such a level. (`03-…` §2, §5)
- **C5.** podbox MUST be fully XDG compliant across config, cache, state, and data roots. (`03-…` §1)
- **C6.** Installed / data-root files are **seed sources only**; once user config exists, runtime
  operations MUST read only from the config/cache/state roots. (`03-…` §1, `09-…`)
- **C7.** The composed `devcontainer.json` is a **derived** artifact in the cache and MUST NOT be
  hand-edited or treated as source. (`04-…`)
- **C8.** Config resolution precedence MUST be exactly: CLI flag → env var → project registry → local
  workspace file → work-clone sibling discovery, and MUST fail with an actionable error (exit `2`)
  when nothing resolves. (`03-…` §5, `02-…` §5)

## 2. Composition invariants

- **P1.** A manifest MUST declare an ordered `layers` list; **manifests are the selection API** — a
  layer directory without a referencing manifest MUST NOT be selectable or listed. (`03-…` §4, `04-…`)
- **P2.** The last layer is the **leaf** (user-protected on reconcile); preceding layers are
  **shared** (reconciled). (`04-…`)
- **P3.** Merge semantics MUST be deterministic: scalars last-wins; `mounts` concatenate;
  `postCreateCommand`, `containerEnv`, `remoteEnv` merge by key; `runArgs`, `workspaceMount`,
  `workspaceFolder` are first-class merge keys; `network.allow` entries union. (`04-…`)
- **P4.** The final `devcontainer.json` MUST be automatically composed from manifest + ordered
  layers. (`04-…`, `02-…` §4.7)
- **P5.** Composition cache freshness MUST be keyed by a digest over manifest content, layer content,
  schema version, composition-rules version, and relevant policy inputs — **stronger than
  mtime-only**. Unknown/unprovable freshness MUST force recomposition or fail closed; stale composed
  output MUST NOT be reused silently. (`04-…`)

## 3. Isolation and security invariants

- **S1.** The primary isolation boundary MUST be **hardware-virtualization (KVM-class) microVM**, not
  a shared-kernel namespace container. Any non-hardware-virt fallback MUST be lower-assurance,
  explicitly labeled, and never silently selected as equivalent. (`07-…`, backend shelf `00-…`/`20-…`)
- **S2.** The runtime MUST be operable **rootless**. (`07-…`)
- **S3.** Default hardening MUST apply: drop-all-capabilities, no-new-privileges, and a tmpfs `/tmp`;
  the host `/tmp` MUST NOT be bind-mounted as the sandbox `/tmp`. No privileged mode by default.
  (`07-…`, `09-…`)
- **S4.** Default network egress posture MUST be deny; outbound access is governed by a composed
  allowlist (manifest + user), enforced in-guest. (`07-…`, `03-…` §4)
- **S5.** Long-lived host credential directories MUST NOT be live-mounted; credentials MUST be
  scoped, short-lived, and auditable. A container control socket MUST NOT be live-mounted. (`07-…`)
- **S6.** Any relaxation of a security default MUST be explicit, named, visible in `doctor`, and
  reflected in `status`. (`08-…`, `11-…`)

## 4. Execution invariants

- **X1.** Lifecycle hooks and `exec` MUST run as **argv vectors**; a multi-line `sh -c` entrypoint
  MUST NOT be emitted under the microVM runtime (the `\n`-mangling constraint). (`05-…`, `07-…`,
  backend shelf `50-…`/`60-…`)
- **X2.** podbox MUST natively parse and compose the supported `devcontainer.json` subset and drive
  lifecycle itself, rather than delegating to a heavyweight orchestrator whose keep-alive shim would
  violate X1. (`07-…`, backend shelf `50-…`)

## 5. Build invariants

- **B1.** `image build` MUST detect source changes correctly by default: an image MUST NOT be served
  as fresh unless it is provably built from the current declared **source graph** and build inputs.
  (`06-…`, `02-…` §4.6)
- **B2.** Silent stale build success MUST NOT occur: missing/unreadable freshness metadata or changed
  inputs MUST cause a rebuild or an explicit failure. `--full-rebuild` is an escape hatch, not a
  correctness requirement. (`06-…`)

## 6. Identity and lifecycle invariants

- **I1.** Each sandbox MUST be keyed by a stable **workspace identity** (canonical path + label) so
  work-clones of the same repository remain separate. (`03-…` §3, `05-…`)
- **I2.** `workspace reconcile` MUST be a first-class, idempotent operation and the recommended path
  after any config change. (`02-…` §4.5, `05-…`)
- **I3.** State-mutating operations on a single workspace MUST be serialized; read-only commands MUST
  tolerate in-progress operations with clear status. (`09-…`)

## 7. UX invariants

- **U1.** Entering/using the sandbox (`shell`) MUST be the central interface, shown first in help.
  (`02-…` §2, §4.1)
- **U2.** `init` MUST yield a lean, clean, non-bloated baseline and MUST run `doctor` afterward
  (unless `--dry-run`). (`02-…` §4.2)
- **U3.** `doctor` MUST check all requirements with specific, actionable diagnostics (no generic
  "unavailable"), and MUST be invokable both standalone and as a post-step. (`02-…` §4.3, `08-…`)
- **U4.** Output MUST follow stream discipline (stdout = data, stderr = diagnostics), offer `--json`
  for machines, honor `--quiet`/`--verbose`/`--no-color`, and never require scraping human text.
  (`10-…`)
- **U5.** Destructive commands MUST be gated by confirmation (bypassable with `--yes`/`--force`) and
  support `--dry-run`. (`02-…`, `10-…`)

## 8. Neutrality invariants

- **N1.** The spec MUST NOT mandate any implementation language, framework, parser library, or
  internal module layout. (`README`, `07-…`)
- **N2.** No single named runtime tool (Podman, `crun`, libkrun, Firecracker, Kata, gVisor, …) may be
  the *only* compliant implementation; such tools appear only as explicitly-marked illustrative
  examples or reference evidence. Requirements are phrased as **capability classes**. (`07-…`)
- **N3.** Public file/config formats (`config.toml`, manifest documents, `devcontainer.json`) MAY be
  specified as API contracts; the code that reads them MUST NOT be specified.

## 9. Guarantees to the user

A conforming implementation guarantees:

- **Reproducibility** — the same manifest + layers deterministically compose the same
  `devcontainer.json`. (P3–P5)
- **No stale builds** — a successful `image build` reflects the current source graph. (B1–B2)
- **A real isolation boundary** — untrusted/agent code runs behind hardware virtualization. (S1)
- **Non-destructive reconcile** — reconciling shared layers never destroys the user's leaf layer.
  (P2, U2)
- **Separate identity per work-clone** — parallel clones never collide. (I1)

## 10. Implementation and contribution invariants

> This group governs **how the implementation binding is built and maintained** — the contributor and
> coding-agent workflow — not the technology-neutral product contract in §1–§9. It is phrased as a
> capability class (a resolve-and-lock package manager), so it does **not** weaken N1–N3: it names no
> language, only a tool *class*, with the Rust binding as the concrete instance.

- **R1.** Dependency management MUST go through the package manager's **resolve-and-lock command**,
  never a hand-edited dependency manifest: additions, removals, feature changes, and upgrades MUST use
  the command that selects a compatible version, resolves the dependency graph, and updates the
  lockfile. Coding agents and contributors MUST NOT hand-add or hand-edit dependency names, versions,
  or features. The lockfile MUST be committed for a binary artifact. *(Rust binding: `cargo add` /
  `cargo remove` / `cargo update`; `Cargo.lock` committed. See ADR-0012 and
  [`../../guides/contributing-rust.md`](../../guides/contributing-rust.md).)*

## Cross-reference index

Each improvement (1–9), operator decision, and implementation invariant (R1) maps to invariants and
sections in [`TRACEABILITY.md`](TRACEABILITY.md). This document is the normative home; descriptive
documents link back here by invariant ID.
