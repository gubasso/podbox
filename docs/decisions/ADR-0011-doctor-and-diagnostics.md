# ADR-0011: First-class doctor with a stable check catalog and report-vs-fail policy

## Context and Problem Statement

podbox must diagnose all host/runtime/config requirements with specific, actionable messages (U3),
surface every security relaxation (S6), and run as both a standalone command and a post-step (U2).
Neutral `08` defines the catalog; `10` fixes exit `3` for a hard-check failure — the two must be
reconciled for post-step failures.

## Considered Options

- Generic "unavailable" errors.
- Fail the whole command on any check failure.
- A stable check catalog with a report-vs-fail policy.

## Decision Outcome

Chosen option: **stable catalog + report-vs-fail** — `commands/doctor.rs` + `domain/doctor.rs` carry
the `08` check IDs (`HOST-KVM/NESTED/OS`, `PERM-USERNS/CGROUP`,
`RT-ISOLATION/ROOTLESS/SMOKE/BUILDER/NETBACK`, `FS-XDG/LAYOUT`, `CFG-SYNTAX/SCHEMA`,
`MAN-SCHEMA/LAYERREF/COMPOSE`, `WS-IDENTITY`, `IMG-FRESH`, `NET-POLICY`, `CRED-POLICY`), severities,
and JSON entry fields (`id`, `category`, `status`, `message`, `evidence`, `remediation`, `docs`).
Policy: a **pre-step** hard requirement failing fails the command (exit `3`); a **post-step** failure
after the primary mutation already succeeded is **reported**, not fatal — reconciling `08` against
`10`'s exit-`3` wording. `init` runs doctor afterward (U2).

## Consequences

- Good: honors U3, S6 (relaxations visible in doctor + `status`), U2; the stable IDs double as
  `err.kind` keys (ADR-0005).
- Bad: the report-vs-fail split must be applied per call site consistently — a review-checklist item.
- Enacted by: `src/commands/doctor.rs`, `src/domain/doctor.rs`;
  `docs/reference/spec/08-doctor-and-diagnostics.md`.

## Status

Accepted
