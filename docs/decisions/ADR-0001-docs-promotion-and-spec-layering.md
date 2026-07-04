# ADR-0001: Promote the neutral spec into Diataxis `docs/`; Rust binding as lean ADRs

## Context and Problem Statement

podbox's product contract was authored as a technology-neutral 15-document shelf in `tmp-specs/`. It
must become an implementation-ready Rust CLI spec **without** the neutral "what podbox is" contract
(invariants N1–N3) being polluted by Rust-specific "how" decisions, and it needs a home a future
implementer and coding agents can navigate.

## Considered Options

- One heavyweight Rust spec shelf replacing the neutral docs.
- A flat `docs/spec/` holding neutral and Rust content together.
- Diataxis zones: neutral contract in `docs/reference/spec/`, Rust binding as lean ADRs in
  `docs/decisions/` plus a compact reference page.

## Decision Outcome

Chosen option: **Diataxis zones** — promote the neutral shelf (a rewrite into the proper zone, not a
bare move) into `docs/reference/spec/`, and express the Rust binding as lean MADR ADRs, keeping the
two layers distinct so neutrality holds. Reference-zone placement (`docs/reference/spec/`, not bare
`docs/spec/`) is chosen for docs-design fidelity: the neutral contract is lookup material.

## Consequences

- Good: N1–N3 preserved — the binding layer never mandates a language inside the neutral contract;
  each zone is a clear reader promise.
- Good: `config.toml`/manifest/`devcontainer.json` formats stay authoritative as API contracts (N3).
- Bad: two-layer indirection; readers follow links between a decision and its reference page.
- Enacted by: `docs/README.md`, `docs/reference/spec/` (the 15 promoted docs), `docs/decisions/`
  (ADR-0002…ADR-0013); `tmp-specs/` removed after Phase 4 verification.

## Status

Accepted
