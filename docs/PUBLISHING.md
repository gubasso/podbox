# Publishing

How this crate is published to [crates.io](https://crates.io). Helper scripts live under `scripts/`
(`publish-dry`, `publish`, `release`). The auth checks in those scripts are **configuration checks
only** — they confirm crates.io auth is set up, never that a token is valid.

## Publishing model

- **CI-first (recommended):** `release-plz` opens a release PR that bumps the version and updates the
  changelog, `Cargo.toml`, and `Cargo.lock`. Merging that PR publishes the new version automatically.
- **Local (escape hatch):** run the helper scripts by hand when CI is unavailable.

## First release (manual)

Trusted Publishing is configured on crates.io **against an already-existing crate**, so the very first
version must be published manually:

1. Create an API token at <https://crates.io/settings/tokens>.
2. `cargo login` and paste the token (stored in `$CARGO_HOME/credentials.toml`).
3. Validate: `./scripts/publish-dry`.
4. Publish: `./scripts/publish`.
5. Configure Trusted Publishing for this repo/workflow on the crate's crates.io settings page.

## Authentication setup

### Trusted Publishing / OIDC (default for CI)

Short-lived, no long-lived secret.

- **With release-plz:** grant the job `permissions: id-token: write` and do **not** set
  `CARGO_REGISTRY_TOKEN` — release-plz mints the OIDC-backed token itself, and it does **not** use
  `rust-lang/crates-io-auth-action`.
- **With a plain `cargo publish` workflow:** use `rust-lang/crates-io-auth-action` to mint a
  short-lived token, then run `cargo publish`.

### Token fallback

When OIDC is unavailable, or for local publishing, use a long-lived token: `cargo login` locally, or a
`CARGO_REGISTRY_TOKEN` secret in CI.

## SemVer policy

For a library crate, `cargo-semver-checks` gates public-API compatibility and runs natively inside
release-plz. Run it locally with `./scripts/release semver-check`. A binary-only crate has no public
API to check but still follows semantic versioning for its releases.

## Routine automated release

1. Merge feature work to the default branch.
2. release-plz opens/updates the release PR (version bump + changelog).
3. Review the PR; merge it.
4. release-plz tags the release and publishes to crates.io.

## Local operator release

When you need to drive a release by hand:

- `./scripts/release release-plz-update` — update versions + changelog locally.
- `./scripts/release release-plz-pr` — open/refresh the release PR.
- `./scripts/release cargo-release-dry <level>` — dry-run a `cargo-release` bump.
- `./scripts/release semver-check` — check API compatibility.

## Readiness checks

`./scripts/publish-dry` runs `cargo publish --dry-run` and `cargo package --list`. Neither needs a
token; run it any time to confirm the package builds and ships the intended files.

## Optional binary distribution

If this crate ships prebuilt binaries or installers, `dist` (cargo-dist) builds them and attaches them
to GitHub releases. It is separate from crates.io publishing and configured in `dist-workspace.toml`.

## Manual release if CI is down

1. `./scripts/publish-dry` to validate.
2. `./scripts/release semver-check` (library crates).
3. Ensure auth is configured (`cargo login`).
4. `./scripts/publish`.

## Yank and rollback

A published version cannot be overwritten or deleted, only yanked:

- `cargo yank --version X.Y.Z` — prevent new dependents from selecting it.
- `cargo yank --version X.Y.Z --undo` — reverse a yank.

Fix forward by publishing a new patch version.
