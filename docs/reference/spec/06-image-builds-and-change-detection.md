# Image builds and change detection

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Normative. Defines how `podbox image build` decides whether to rebuild an image or reuse a cached
> result. The load-bearing rule is **reliable-by-default change detection**: a build MUST NOT serve a
> stale image as fresh. Terms in **Reference Evidence** are non-normative.

This document elaborates build invariants **B1** and **B2** from
[`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md) and specifies the `image build`
contract summarized in [`02-command-surface.md`](02-command-surface.md) §4.6. It specifies a
**decision contract** (declared inputs → rebuild-or-reuse), not a cache mechanism: any implementation
that satisfies the contract conforms, regardless of how it stores or compares freshness data.

## 1. Image source model

An image usable by podbox is defined in one of two ways:

- **User-owned image definition** — a directory `$PODBOX_CONFIG_HOME/images/<name>/` holding the
  image's build definition (its build file) plus the build context and assets it declares. This is
  the source of truth for images podbox builds itself. (`images/` is defined in
  [`03-config-and-xdg-layout.md`](03-config-and-xdg-layout.md) §2.)
- **External reference** — an image reference declared by the composed `devcontainer.json` (see
  [`04-manifest-and-composition-model.md`](04-manifest-and-composition-model.md)) that podbox does
  not build, only consumes. For an external reference, podbox does not run a build; its freshness is
  governed entirely by the pull policy (§4) applied when the reference is resolved.

`image build` operates on **user-owned image definitions**. Installed / data-root files are seed
sources only; a build MUST read image definitions exclusively from the config root (invariant **C6**,
[`03-…`](03-config-and-xdg-layout.md) §1).

## 2. Reliable default (invariant B1)

A normal `podbox image build` — invoked with no force flag — MUST detect source changes correctly by
default. Concretely:

- A conforming implementation MUST NOT serve an image as fresh unless it can **prove** the image was
  built from the **current declared source graph** (§3) and the current relevant build inputs.
- When that proof succeeds, the build MAY reuse the existing image and report it as up to date.
- When that proof does not succeed — because an input changed, or because freshness cannot be
  established — the build MUST rebuild the affected image or fail explicitly (§4). It MUST NOT reuse
  the prior artifact silently.

Reliable change detection is a **default**, not an opt-in. `--full-rebuild` (§4) is an explicit
force/no-cache escape hatch and MUST NOT be required to obtain a correct result.

## 3. Source graph

The **source graph** of an image is the complete set of inputs whose change could alter the built
artifact. For `podbox image build` it MUST include, at minimum:

- The **image definition file** (the build recipe) for `<name>`.
- **All files in the declared build context** — every file the build definition can reference,
  including files reachable through the context that affect output.
- **Included scripts and assets** the build definition copies in or executes.
- **Build arguments that affect output** — build args passed to the build whose value changes the
  result (for example the user-identity build args baked into the image; see §7 Reference Evidence).
- **Base-image identity / pull-policy result** — the resolved identity of the base image after the
  effective `--pull-policy` (`missing|newer|always|never`) is applied. A change in the base image
  the build resolves to is a source-graph change.
- The **podbox schema / build-policy version** — a change in the build-decision rules or image
  schema version invalidates prior freshness proofs.
- **User-identity inputs**, when they are baked into the image (for example a username or UID/GID
  compiled into the image), since a change to them changes the artifact.

An implementation MAY track a superset of these inputs, but MUST NOT omit any of them. Omitting an
input that affects output is the failure mode invariant **B1** forbids: it lets a changed input go
undetected and a stale image be served as fresh.

## 4. Freshness metadata and cache policy (invariant B2)

A successful build MUST record enough **freshness metadata** to later compare the current source
graph (§3) against the artifact that was built. The **storage format is not prescribed** — a digest,
a manifest of per-input fingerprints, or any equivalent record satisfies the contract, provided it is
sufficient to decide reuse-vs-rebuild reliably.

The cache decision is then:

- A cached image MAY be reused **only when** a freshness proof succeeds — that is, the recorded
  metadata demonstrates the artifact was built from the current source graph and build inputs.
- If the freshness metadata is **missing or unreadable**, or **any tracked input changed**, the build
  MUST rebuild the affected image or fail with an explicit, actionable error (exit code `6`, build
  failed; see [`02-…`](02-command-surface.md) §5). It MUST NOT fall back to reusing the prior
  artifact.
- **Silent stale success is forbidden (invariant B2).** A build MUST NOT report success against an
  image that does not provably reflect the current source graph.

`--full-rebuild` is an explicit **no-cache escape hatch**: it forces a rebuild of the targeted images
(or all images with `--all`) ignoring any cached result. It is a convenience for forcing a clean
rebuild, **not** a correctness requirement — a plain `podbox image build` already rebuilds on any
detected change per B1/B2.

The `--pull-policy missing|newer|always|never` flag governs how the base-image identity component of
the source graph is resolved: `always`/`newer` may resolve a newer base (itself a source-graph
change), `never` pins to the locally present base, and `missing` pulls only when absent. Whichever
policy applies, the **resolved** base identity is what enters the freshness proof.

## 5. Dry run and JSON

`podbox image build ... --dry-run` MUST report what it *would* do without mutating any image or
freshness metadata. Combined with `--json` (`--dry-run --json`), it MUST emit stable machine-readable
output naming, for each targeted image:

- whether it would **rebuild** or **reuse**, and
- **why** — which source-graph input changed (or that freshness could not be proven), or that all
  inputs are unchanged.

The human-readable (non-`--json`) dry run MUST convey the same rebuild/reuse decision and reason in
prose. Output stream and JSON-stability rules follow
[`10-errors-output-and-scriptability.md`](10-errors-output-and-scriptability.md).

## 6. Interaction with reconcile

Image freshness is a precondition for a correct sandbox.
`podbox workspace reconcile` (see [`05-workspace-lifecycle-and-shell-ux.md`](05-workspace-lifecycle-and-shell-ux.md))
MUST verify the freshness of every image its composed config depends on **before** creating a
sandbox. On detected staleness it MUST either rebuild per the effective image policy (§4) or fail
with an actionable remediation naming the stale image and the `podbox image build` invocation to run.
A reconcile MUST NOT create a sandbox from an image that cannot be proven fresh.

`podbox status` (see [`02-…`](02-command-surface.md) §4.4) surfaces image freshness as part of a
workspace's drift state, so a stale image is visible before reconcile is invoked.

## 7. What this contract makes impossible (legacy anti-pattern)

The legacy `dctl` build had exactly the failure mode this contract forbids. A plain `dctl image
build` passed no `--no-cache`, so a stale build cache could mask source changes; in practice only
`--full-rebuild` (which set all-targets **and** no-cache) was reliable. Agent image layers even
carried an explicit `CACHEBUST_AGENTS` build argument driven by a `--refresh-agents` flag — direct
evidence that change detection was **manual, not automatic**. Under invariants **B1** and **B2** this
is impossible for a conforming podbox: a plain `podbox image build` must itself detect the change and
rebuild, `--full-rebuild` is only an escape hatch, and no manual cache-busting build argument is
required to obtain a correct image.

## Reference Evidence (non-normative)

- `lib/dctl/commands/image/build.sh` — the legacy build command. A plain build passes no
  `--no-cache`; `--full-rebuild` sets `all=true` and `no_cache=true` together; the `agents` target
  uses a `--refresh-agents` flag that injects `--build-arg CACHEBUST_AGENTS=$(date +%s)` and a
  `--pull`. It also bakes `USERNAME`, `USER_UID`, and `USER_GID` build args into every image
  (evidence for the user-identity source-graph inputs in §3). This is the concrete change-detection
  gap that invariants **B1**/**B2** close.
- `docs/ARCHITECTURE.md` — describes the layered managed image stack and user-config-only
  Containerfile resolution the podbox `images/` model carries forward.
