# Reconciliation report

> Reference (lookup). The bidirectional change ledger for the spec-authoring work: what changed in
> **podbox** (`/workspaces/podbox`) and what was reconciled back into the canonical
> **docs-n-notes** shelves (`/home/gbasso/Projects/docs-n-notes`). "Both ends are improveable" — this
> table records every touched artifact with its decision, status, and how it was verified. No `git`
> was run (commits are a separate, explicitly-authorized step).

## 1. Podbox spec changes

| Artifact | Decision | Status | Rationale | Verification |
| -------- | -------- | ------ | --------- | ------------ |
| Neutral product shelf (33 invariants, 2 theses, 5 personas, 11 goals, 6 non-goals, command surface, config schema) | keep | applied | Fixed inputs; only expression improved, no product decision changed | Per-file diff vs originals shows only banner + fixes; exact 33-invariant count parity |
| `tmp-specs/` → `docs/reference/spec/` | modify | applied | Promote (rewrite into Diataxis reference zone) per docs-design; `tmp-specs/` removed after verification | `find docs`; `tmp-specs` absent; grep battery clean |
| Stray `</content></invoke>` in `08` **and** `</content>` in `09` | remove | applied | EOF remnant tags; `09` was an extra defect beyond the plan's `08` | `grep -rn "</content>\|</invoke>"` → clean |
| `.plan/podbox/` self-location refs (`README`, `TRACEABILITY` §2) | modify | applied | Stale after promotion → `docs/reference/spec/` | grep in `docs/` → none |
| A/B thesis banner atop each promoted doc | add | applied | Keep both theses visible in every derived doc (`08`-plan 2.3) | 15/15 files carry the banner |
| 13 lean ADRs `docs/decisions/ADR-0001…0013` | add | applied | Rust binding as MADR-minimal ADRs (operator: "lean ADRs, follow docs-design") | 5 sections, ≤350 words, one `Status`, invariant IDs cited (all 13 pass) |
| `docs/decisions/template.md` | add | applied | docs-design ADR template drop-in | byte-for-byte identical to source |
| `RUST-BINDING-TRACEABILITY.md` | add | applied | Bind all 33 invariants + R1 to ADR + module | 34 rows, every invariant ≥1 binding |
| `docs/reference/rust-checklist.md` | add | applied | Idiomatic-Rust/CLI gate from cli-design `99-checklist` | every box satisfied-by-ADR or waived-by-ADR |
| Invariant **R1** (cargo-CLI-only) in `11-…` §10 + `docs/guides/contributing-rust.md` | add | applied | Operator-mandated dependency hygiene, phrased as a capability class so N1–N3 hold | R1 in `11`; TRACEABILITY + RUST-BINDING rows; guide links ADR-0012 |
| `13-rust-implementation-binding.md`, `external-facts.md`, `explanation/architecture.md`, `guides/adding-a-subcommand.md`, `docs/README.md` | add | applied | Compact reference page, perishable-fact registry, mental model, four-edit walkthrough, zone index | files present; capability→module map complete; facts carry revalidate-by dates |
| Exit-code clash (BSD sysexits vs podbox `0`–`7`) | modify | applied | podbox canon `02`/`10`/`11` governs; sysexits are influence only | ADR-0005 + `rust-checklist` waiver |
| `deny_unknown_fields` vs forward-compat | modify | applied | podbox `03` "preserve unknown keys" overrides cli-spec `05` default | ADR-0007 + `rust-checklist` waiver |
| Hello-world `println!` in `src/main.rs` | keep (flag) | applied (flagged) | Future no-print-outside-`ui/` violation; implementation out of scope unless the optional skeleton phase runs | Recorded here; resolved only if Phase 8 replaces `main.rs` |
| Legacy `dctl` evidence re-read | keep | blocked | Sources (`devcontainerctl`, dotfiles) absent from this environment | Non-normative Reference-Evidence summaries in the shelf suffice |

## 2. docs-n-notes applied changes (`/home/gbasso/Projects/docs-n-notes`)

| Artifact | Decision | Status | Rationale | Verification |
| -------- | -------- | ------ | --------- | ------------ |
| `rust/cli-spec/07-dependencies.md` — new `## Adding dependencies` section | add | applied | Shelf was silent on `cargo add`; encode resolve-and-lock rule + forbid hand-edited pins | `grep "Adding dependencies"` present; worked example uses `cargo add` |
| `rust/cli-spec/07-dependencies.md` — crate currency | modify | applied | `thiserror` 1→2, `toml` 0.8→1 (spec 1.1), `anstream`→1.0; pinning policy intact | version strings updated in-chapter |
| `rust/cli-spec/adr/0001-cargo-cli-only-dependencies.md` | add | applied | Default-dependency-policy change warrants an ADR in the shelf's own format | file present; `# ADR-0001 — …`, `**Status:** Accepted` |
| `rust/cli-spec/templates/Cargo.toml.template` — de-hardcoded deps | modify | applied | Templates must not re-pin stale versions by hand | `[dependencies]` block replaced with a `cargo add` comment; `[package]` preserved |
| `rust/cli-spec/README.md` — templates listing | modify | applied | Listing omitted real files | now includes `main.rs.machine.template`, `cli/example.rs.template`, `commands/example.rs.template` |
| `rust/cli-spec/AGENTS.md` — `source-files:` + last-synced | modify | applied | `06-testing-and-quality/` was missing; rescan after ch.07 edits | `06-testing-and-quality/` listed; last-synced 2026-07-04 |
| `programming/cli-design/99-checklist.md` — dependency-hygiene item | add | applied | Language-agnostic principle (`cargo add`/`uv add`/`go get`, never hand-pin) | item present under CI/shipping; no `adr/` invented (shelf has none) |

## 3. docs-n-notes flagged proposals (`Status: Proposed` — NOT applied)

Captured in `rust/cli-spec/adr/PROPOSALS.md`:

| Proposal | Decision | Status | Rationale |
| -------- | -------- | ------ | --------- |
| P1 — Recommend `miette` 7.x for rich user-facing diagnostics | add | proposed | Newer path for source-span CLI diagnostics; not yet mandated |
| P2 — Note `etcetera` (`choose_app_strategy`) as a `directories` alternative | add | proposed | Trending XDG-forcing strategy for CLI-first tools (uv) |
| P3 — `.devcontainer-lock.json` lockfile-by-default + `outdated`/`upgrade` | add | proposed | devcontainer-wrapping consideration |
| P4 — cli-spec `05-config.md` `deny_unknown_fields` trade-off note / non-strict variant | modify | proposed | A downstream project (podbox) overrode it for forward-compat; shelf should note the trade-off |
| P5 — language-agnostic dependency-hygiene ADR for cli-design | add | proposed | Only if/when cli-design adopts an `adr/` convention (none exists today) |

## Files touched (two-repo summary)

- **podbox** (`/workspaces/podbox`): `docs/` created (README, `reference/spec/` ×17 incl. promoted 15
  + RUST-BINDING-TRACEABILITY + 13-rust-implementation-binding, `reference/` rust-checklist +
  external-facts + reconciliation-report, `decisions/` 13 ADRs + template, `guides/` ×2,
  `explanation/` ×1); `tmp-specs/` removed; `.gitignore` appended `/​.draft/`.
- **docs-n-notes** (`/home/gbasso/Projects/docs-n-notes`): `rust/cli-spec/` (`07-dependencies.md`,
  `README.md`, `AGENTS.md`, `templates/Cargo.toml.template`, `adr/0001-…`, `adr/PROPOSALS.md`);
  `programming/cli-design/99-checklist.md`.
