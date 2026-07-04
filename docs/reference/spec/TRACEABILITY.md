# Traceability matrix

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Non-normative index. Proves every requirement from the source request landed in a concrete section
> of this shelf. Each row cites the governing document/section and the normative invariant(s) from
> [`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md) that bind it.

## 1. The nine improvements

| # | Requirement (from request)                                             | Governing section(s)                                                                                   | Invariant(s)     |
| - | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ---------------- |
| 1 | Config reorg: `devcontainer/` layers-only, `manifests/`, `images/`, `config.toml`, no `default/devcontainer.json` | [`03-…`](03-config-and-xdg-layout.md) §2–§4                                                             | C1, C2, C3, C4   |
| 2 | Remove `deploy`; fold a lean, clean baseline into `init`               | [`02-…`](02-command-surface.md) §2, §4.2                                                                | U2               |
| 3 | First-class `doctor` checking all requirements with specific diagnostics | [`02-…`](02-command-surface.md) §4.3; [`08-…`](08-doctor-and-diagnostics.md)                            | U3, S6           |
| 4 | `init` (and other relevant commands) run `doctor` afterward            | [`02-…`](02-command-surface.md) §4.2–§4.3; [`08-…`](08-doctor-and-diagnostics.md) (Post-Command Checks) | U2, U3           |
| 5 | Keep automatic final `devcontainer.json` composition from layers + manifests | [`04-…`](04-manifest-and-composition-model.md)                                                          | P1–P5, C7        |
| 6 | Full XDG compliance, user-home focused                                 | [`03-…`](03-config-and-xdg-layout.md) §1                                                                | C5, C6           |
| 7 | Reliable-by-default builds; no silently-stale output                   | [`06-…`](06-image-builds-and-change-detection.md); [`02-…`](02-command-surface.md) §4.6                 | B1, B2           |
| 8 | Reconcile-first lifecycle (`reup` habit → first-class `reconcile`)     | [`02-…`](02-command-surface.md) §4.5; [`05-…`](05-workspace-lifecycle-and-shell-ux.md)                  | I2               |
| 9 | Shell-centric UX — entering the sandbox is the central interface       | [`00-…`](00-goals-and-non-goals.md) (Thesis A); [`02-…`](02-command-surface.md) §2, §4.1; [`05-…`](05-workspace-lifecycle-and-shell-ux.md) | U1               |

## 2. Operator decisions

| Decision                                                             | Where honored                                                                     | Invariant(s) |
| ------------------------------------------------------------------- | --------------------------------------------------------------------------------- | ------------ |
| Command surface **re-derived from state-of-the-art CLI best practices** (not inherited from dctl) | [`01-…`](01-cli-design-research.md) → checklist consumed by [`02-…`](02-command-surface.md) §1 | —            |
| Spec promoted in-repo to `docs/reference/spec/` (per docs-design Diataxis zones)  | This shelf's location                                                             | —            |
| Dependency management via the package-manager **resolve-and-lock command** only (operator-mandated; no hand-edited manifests) | [`11-…`](11-invariants-and-guarantees.md) §10; ADR-0012; [`../../guides/contributing-rust.md`](../../guides/contributing-rust.md) | R1           |

## 3. Cross-cutting requirements

| Requirement                                                    | Governing section(s)                                                                       | Invariant(s) |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------------------ | ------------ |
| Technology-neutrality (no mandated language/framework/tool)    | [`README`](README.md) (neutrality rule); [`07-…`](07-runtime-and-infrastructure.md)        | N1, N2, N3   |
| Isolation rationale cited to the backend shelf, not re-argued  | [`07-…`](07-runtime-and-infrastructure.md); [`README`](README.md) (Reference Evidence)     | S1           |
| Hardware-virt (KVM-class) microVM as the primary boundary      | [`07-…`](07-runtime-and-infrastructure.md)                                                 | S1, S2       |
| Default security posture (drop-caps, no-new-privs, tmpfs `/tmp`, no cred live-mount, default-deny egress) | [`07-…`](07-runtime-and-infrastructure.md); [`09-…`](09-state-cache-and-data-model.md)     | S3, S4, S5, S6 |
| Argv-vector exec only (no multi-line `sh -c` entrypoint)       | [`05-…`](05-workspace-lifecycle-and-shell-ux.md); [`07-…`](07-runtime-and-infrastructure.md) | X1, X2       |
| Native devcontainer parsing / composition                     | [`04-…`](04-manifest-and-composition-model.md); [`07-…`](07-runtime-and-infrastructure.md) | X2, P4       |
| Workspace-label identity (work-clones stay separate)          | [`03-…`](03-config-and-xdg-layout.md) §3; [`05-…`](05-workspace-lifecycle-and-shell-ux.md) | I1           |
| Config-resolution precedence (no user-default level)          | [`02-…`](02-command-surface.md) §5; [`03-…`](03-config-and-xdg-layout.md) §5               | C8           |
| Scriptability (stdout/stderr discipline, `--json`, exit codes) | [`10-…`](10-errors-output-and-scriptability.md); [`02-…`](02-command-surface.md) §3, §5    | U4           |
| Destructive-action safety (confirmation, `--yes`/`--force`, `--dry-run`) | [`02-…`](02-command-surface.md) §3–§4; [`10-…`](10-errors-output-and-scriptability.md)      | U5           |
| State/cache/data model + concurrency + recovery               | [`09-…`](09-state-cache-and-data-model.md)                                                 | C6, I3       |
| Terminology consistency                                        | [`12-glossary.md`](12-glossary.md)                                                         | —            |

## 4. Document coverage checklist

Every planned document exists and carries its required sections:

- [x] `README.md` — index, normative language, neutrality rule, core decisions, reading order.
- [x] `00-goals-and-non-goals.md` — problem, personas, goals/non-goals, theses, principles.
- [x] `01-cli-design-research.md` — distilled SOTA CLI conventions + podbox-conventions checklist.
- [x] `02-command-surface.md` — commands, global flags, exit codes, contracts.
- [x] `03-config-and-xdg-layout.md` — XDG roots, config tree, `config.toml`, precedence.
- [x] `04-manifest-and-composition-model.md` — layers, manifests, merge, freshness.
- [x] `05-workspace-lifecycle-and-shell-ux.md` — state/drift model, verbs, hooks, pairing.
- [x] `06-image-builds-and-change-detection.md` — source graph, freshness proof, cache policy.
- [x] `07-runtime-and-infrastructure.md` — capability classes, adapter, security posture.
- [x] `08-doctor-and-diagnostics.md` — check catalog, severity, output/exit, post-step.
- [x] `09-state-cache-and-data-model.md` — inventory, concurrency, cleanup, recovery.
- [x] `10-errors-output-and-scriptability.md` — streams, JSON, prompting, exit codes.
- [x] `11-invariants-and-guarantees.md` — collected MUST/SHOULD list + guarantees.
- [x] `12-glossary.md` — terms of art.
- [x] `TRACEABILITY.md` — this matrix.
