# podbox documentation

Index into the project's Diataxis documentation zones (see
`docs-n-notes/tech/programming/docs-design`). Each zone is a reader promise, not a topic bucket. This
page is only an index — it holds no canonical facts of its own.

## Zones

- **[`reference/spec/`](reference/spec/README.md)** — the **technology-neutral product contract**:
  what podbox *is and does*. Command surface, `config.toml` schema, composition model, lifecycle,
  runtime capability classes, diagnostics, and the 33 normative invariants. Promoted from the original
  neutral spec shelf; the four canon docs (`README`, `02`, `03`, `11`) and `TRACEABILITY.md` govern.
  This zone names **no** language or tool (invariants N1–N3).
- **[`decisions/`](decisions/)** — lean MADR ADRs. The **Rust implementation binding** lives here as
  `ADR-0001`…`ADR-0013` (crate layout, module boundaries, four-edit subcommand pattern, error/exit
  taxonomy, config loader, CLI-wrapper architecture, composition, image freshness, doctor,
  dependency-management policy, testing). `template.md` is the drop-in ADR shape.
- **[`reference/`](reference/)** — lookup material beyond the spec:
  [`implementation-status.md`](reference/implementation-status.md) (what works today vs.
  deferred — the single source of truth for status), `spec/RUST-BINDING-TRACEABILITY.md`
  (invariant → ADR + module), `rust-checklist.md` (idiomatic-Rust/CLI gate), `external-facts.md`
  (perishable web facts with revalidation dates), and `reconciliation-report.md`.
- **[`guides/`](guides/)** — task docs: [`getting-started.md`](guides/getting-started.md) (a
  hands-on walkthrough of the working surface), `contributing-rust.md` (dependency-management
  rule), `adding-a-subcommand.md` (the four-edit walkthrough).
- **[`explanation/`](explanation/)** — the mental model: `architecture.md` binds the neutral
  capability classes to the Rust module map and cites the isolation-backend evidence shelf.

## Where things live

- The neutral **what** contract: `reference/spec/`.
- The Rust **how/why** binding: `decisions/` (durable rationale) + `reference/spec/13-rust-implementation-binding.md`
  (compact lookup) — never a second heavyweight shelf.
- Author instructions / normative rules for the product: `reference/spec/11-invariants-and-guarantees.md`.
