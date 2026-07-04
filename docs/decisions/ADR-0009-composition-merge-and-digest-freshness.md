# ADR-0009: Deterministic composition, merge semantics, and digest-keyed freshness

## Context and Problem Statement

podbox composes an ordered manifest + layers into a single `devcontainer.json` (P1–P4) that must be
reproducible, must protect the user's leaf layer on reconcile (P2), and must never be served stale
(P5, C7).

## Considered Options

- Delegate merge to an external tool.
- mtime-based cache freshness.
- In-house deterministic merge with a content digest.

## Decision Outcome

Chosen option: **in-house deterministic merge + content digest** — `domain/manifest.rs` implements P3
verbatim (scalars last-wins; `mounts` concatenate; `postCreateCommand` / `containerEnv` / `remoteEnv`
merge-by-key; `runArgs` / `workspaceMount` / `workspaceFolder` first-class; `network.allow` union).
`services/compose.rs` performs auto-composition (P4), writing the composed `devcontainer.json` to the
cache root **only** (C7 — derived, never hand-edited). Freshness (P5) is keyed by a digest over
manifest + layer content + schema version + composition-rules version + policy inputs — stronger than
mtime; unknown/unprovable freshness forces recomposition or fails closed. The last layer is the
protected leaf (P2); preceding layers reconcile.

## Consequences

- Good: the reproducibility guarantee (same inputs → same output) holds; no silent stale compose; the
  leaf is never destroyed on reconcile.
- Good: the digest is the same primitive image freshness uses (ADR-0010), so `domain/digest.rs` is
  shared.
- Bad: composition rules must be versioned and kept in lockstep with the neutral `04` — a maintenance
  obligation.
- Enacted by: `src/domain/manifest.rs`, `src/domain/digest.rs`, `src/services/compose.rs`;
  `docs/reference/spec/04-manifest-and-composition-model.md`.

## Status

Accepted
