# ADR-0010: Reliable-by-default image builds via source-graph change detection

## Context and Problem Statement

An image must never be served as fresh unless provably built from the current declared source graph
(B1), and missing or unreadable freshness metadata must not yield a silent stale success (B2).

## Considered Options

- Trust the runtime's layer cache alone.
- mtime / tag heuristics.
- An explicit source-graph digest with fail-closed semantics.

## Decision Outcome

Chosen option: **source-graph digest, fail-closed** — `domain/image.rs` models the declared build
inputs (the `06` source graph); `domain/digest.rs` computes a freshness proof over them;
`services/image_build.rs` rebuilds when the digest changed or metadata is missing/unreadable, else
reuses. A stale build is never served silently; `--full-rebuild` is an escape hatch, not a
correctness lever (B2). The build itself runs through the runtime adapter (ADR-0008).

## Consequences

- Good: honors B1/B2 and the user guarantee "a successful build reflects the current source graph";
  shares the digest primitive with composition (ADR-0009).
- Bad: computing the source-graph digest costs I/O on every build check — accepted for correctness.
- Bad: unprovable freshness rebuilds or fails rather than guessing — may surprise users expecting a
  cache hit; documented in `06`.
- Enacted by: `src/domain/image.rs`, `src/domain/digest.rs`, `src/services/image_build.rs`,
  `src/adapters/runtime.rs`; `docs/reference/spec/06-image-builds-and-change-detection.md`.

## Status

Accepted
