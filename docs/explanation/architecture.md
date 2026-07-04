# Architecture overview

> Explanation (understanding). The mental model that binds podbox's technology-neutral capability
> classes to the Rust module map. It links to the decisions and reference pages rather than restating
> them, and cites the isolation-backend evidence shelf rather than re-arguing it.

## What podbox is, in one paragraph

podbox provisions isolated, reproducible, per-workspace devcontainer sandboxes on a KVM-class
hardware-virtualization microVM boundary. Two theses shape everything: **A — the sandbox is the
interface** (`podbox shell` is top-level and shown first; U1) and **B — reconcile-on-change is the
everyday path** (`workspace reconcile` is first-class and idempotent; I2). The neutral contract in
[`../reference/spec/`](../reference/spec/README.md) says *what* podbox is; this repo implements it in
Rust, with the *why* recorded as ADRs in [`../decisions/`](../decisions/).

## The layered shape

podbox is a single Rust binary organized by cli-design's parse-shape vs runtime-shape split
([ADR-0003](../decisions/ADR-0003-module-boundaries-and-appcontext.md)):

- **Parse-shape** (`cli/`) turns argv into typed clap structs and nothing more.
- **Runtime-shape** is the rest: handlers (`commands/`) project args into domain requests, pure
  domain logic (`domain/`, zero I/O) computes, and adapters (`adapters/`) are the only code that talks
  to the outside world. One `AppContext` is built once in `main` and threaded by reference.

A subcommand is added mechanically via the four-edit rule
([ADR-0004](../decisions/ADR-0004-four-edit-subcommand-pattern.md)); see the
[walkthrough guide](../guides/adding-a-subcommand.md). The full module and command map is in
[`../reference/spec/13-rust-implementation-binding.md`](../reference/spec/13-rust-implementation-binding.md).

## The three hard subsystems

1. **Composition** (P1–P5, C7). An ordered manifest + layers merge deterministically into one
   `devcontainer.json`, written to the cache as a derived artifact. Freshness is a content digest, not
   mtime, and fails closed. Rationale and merge rules:
   [ADR-0009](../decisions/ADR-0009-composition-merge-and-digest-freshness.md).

2. **Image freshness** (B1–B2). An image is never served as fresh unless provably built from the
   current source graph; missing metadata rebuilds or fails rather than serving stale output. Shares
   the digest primitive with composition. See
   [ADR-0010](../decisions/ADR-0010-image-build-freshness.md).

3. **The CLI-wrapper boundary** (X1, X2, S3–S5). podbox drives a rootless microVM container runtime
   and the `devcontainer` CLI subset through typed builders that serialize to argv vectors — never
   `sh -c`. It forwards signals, maps exit codes (child N→N; killed-by-signal→128+N), and applies the
   security hardening flags on the generated command line, where golden-argv tests lock them down. See
   [ADR-0008](../decisions/ADR-0008-cli-wrapper-architecture.md).

## Errors, exit codes, and output

One `AppError` maps every failure to podbox's own `0`–`7` exit taxonomy (not BSD sysexits — see
[ADR-0005](../decisions/ADR-0005-error-taxonomy-and-exit-codes.md)). `ui/` is the only module that
prints; stdout is data, stderr is diagnostics, `tracing` logs go to an XDG state file
([ADR-0006](../decisions/ADR-0006-logging.md)). This is what makes podbox scriptable and
agent-legible (U4).

## Why hardware-virtualization isolation (cited, not re-argued)

The isolation rationale — threat model, runtime catalog, the libkrun decision, libkrun-vs-Firecracker,
and the native-orchestration decision — is owned by the vendor-neutral evidence shelf at
`docs-n-notes/tech/infra/sandbox-isolation-backends/`:

- `00-threat-model-and-principles.md` — why a real VM boundary, not shared-kernel namespaces (S1).
- `20-decision-libkrun-linux.md` — the KVM microVM choice on Linux (S1, S2).
- `30-libkrun-vs-firecracker.md` — monitor trade-offs.
- `50-native-orchestration-decision.md` — why podbox drives lifecycle itself rather than delegating
  to a heavyweight orchestrator (X2).
- `60-podman-libkrun-operational-notes.md` — known runtime pitfalls, incl. that krun does not support
  `podman exec` into the VM, which is why hooks/`exec` are argv vectors podbox emits itself (X1).

podbox docs treat these as non-normative evidence. The neutral spec's own invariants (S1–S6, X1, X2)
are the binding requirements; the backend shelf explains the *why* behind them.
