# CLAUDE.md

Project-specific guidance for working in this repository.

## Publishing / crate packaging

- The published `.crate` must stay lean: it should contain only what builds the
  binary (`src/`, `Cargo.toml`, auto `Cargo.lock`) plus `README.md` and `LICENSE`
  for the crates.io page. Project docs live in the repo (`docs/`, reachable via the
  `repository` field); dev/CI tooling is never a build input.
- The `exclude` list in `Cargo.toml` enforces this. **Whenever a new top-level
  config or tooling artifact is added** (e.g. another `*.toml` at the root, a new
  dotfile, a `scripts/` sibling, a docs subtree), review and update `exclude` so it
  does not leak into the tarball.
- Verify after any change with `cargo package --list` (or `./scripts/publish-dry`):
  the output should stay limited to `src/`, `Cargo.toml`, `Cargo.lock`, `README.md`,
  `LICENSE`, and Cargo's own synthesized files. Anything else is a signal to extend
  `exclude`.
- Use `exclude` (denylist), not `include` (allowlist): this crate sets
  `license = "MIT"` (an SPDX expression, not `license-file`), so an allowlist would
  silently drop `README.md`/`LICENSE` unless listed explicitly. See
  `docs/PUBLISHING.md` for the full publishing runbook.
