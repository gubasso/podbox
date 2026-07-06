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
version must be published manually. After this one-time bootstrap, CI publishes every subsequent
release over OIDC with no stored token.

0. **Prerequisite — crate metadata.** crates.io **rejects** a publish without `description` and a
   license, and warns without `repository`. Confirm `Cargo.toml` has `description`, `license`,
   `repository`, and `readme`, then validate the package builds and ships the intended files (no token
   needed):

   ```bash
   ./scripts/publish-dry
   ```

1. **Create a scoped API token** at <https://crates.io/settings/tokens>. Use a narrow, disposable,
   least-privilege token — not a broad "manage everything" one:
   - **Name:** `podbox-bootstrap-first-publish` (obviously a one-off, so it gets revoked later).
   - **Endpoint scopes:** **`publish-new` only** — the first publish *creates* the crate, so
     `publish-new` is required and `publish-update` is not. Leave `yank`, `change-owners`, and
     `legacy` unchecked.
   - **Crate scopes:** the exact name **`podbox`** (no wildcard).
   - **Expiration:** the **shortest** option offered — this token only needs to live long enough for
     one publish.

2. **Log in** and paste the token (cargo stores it in `$CARGO_HOME/credentials.toml`):

   ```bash
   cargo login
   ```

3. **Validate again** (this is what the auth-gated publish will build):

   ```bash
   ./scripts/publish-dry
   ```

4. **Publish** the first version:

   ```bash
   ./scripts/publish
   ```

5. **Configure Trusted Publishing** on the crate settings page
   (<https://crates.io/crates/podbox/settings>, the "Trusted Publishing" section — a crate *owner*
   only). Add a GitHub Actions publisher matching `.github/workflows/release.yml`:
   - **Repository owner:** `gubasso`
   - **Repository name:** `podbox`
   - **Workflow filename:** `release.yml` — this is the workflow *file* name, **not** the workflow's
     `name:` field (which is `release-plz`).
   - **Environment:** leave blank (the `release-plz` job declares no `environment:`).

   The publisher matches on owner + repo + workflow filename (+ optional environment) — it is
   **branch-agnostic**, so the `develop` trigger needs no change here. From now on CI mints a
   short-lived OIDC token itself — see
   [Trusted Publishing / OIDC](#trusted-publishing--oidc-default-for-ci).

6. **Revoke the bootstrap token** on <https://crates.io/settings/tokens>. Its only job is done; CI no
   longer needs it. Keep a long-lived token only if you want the local escape hatch (see
   [Token fallback](#token-fallback)).

7. **(Recommended) Enforce Trusted Publishing** — see
   [Require trusted publishing](#require-trusted-publishing-hardening) below.

## Authentication setup

### Trusted Publishing / OIDC (default for CI)

Short-lived, no long-lived secret.

- **With release-plz:** grant the job `permissions: id-token: write` and do **not** set
  `CARGO_REGISTRY_TOKEN` — release-plz mints the OIDC-backed token itself, and it does **not** use
  `rust-lang/crates-io-auth-action`.
- **With a plain `cargo publish` workflow:** use `rust-lang/crates-io-auth-action` to mint a
  short-lived token, then run `cargo publish`.

#### Configure the trusted publisher (one-time)

Do this once, after the crate exists (i.e. after the first manual publish). On the **crate settings
page** — <https://crates.io/crates/podbox/settings>, the **Trusted Publishing** section (a crate
*owner* only) — add a GitHub Actions publisher matching `.github/workflows/release.yml`:

1. Open <https://crates.io/crates/podbox/settings> and find **Trusted Publishing** → **Add**.
2. Fill the form:
   - **Repository owner:** `gubasso`
   - **Repository name:** `podbox`
   - **Workflow filename:** `release.yml` — the file name, **not** the workflow's `name:`
     (`release-plz`).
   - **Environment:** leave blank (the job declares no `environment:`).
3. Save. The publisher matches on owner + repo + workflow filename (+ optional environment) and is
   **branch-agnostic**, so the `develop` trigger needs no change here.

This is the same configuration referenced by step 5 of
[First release (manual)](#first-release-manual); after it is in place, CI publishes every release
over OIDC with no stored token.

### Token fallback

When OIDC is unavailable, or for local publishing, use a long-lived token: `cargo login` locally, or a
`CARGO_REGISTRY_TOKEN` secret in CI.

### Require trusted publishing (hardening)

The crate settings page has a **"Require trusted publishing for all new versions"** checkbox. When
enabled, crates.io **rejects every publish that authenticates with an API token** (both local
`cargo login` tokens and a `CARGO_REGISTRY_TOKEN`); only an OIDC exchange from the configured trusted
publisher can push a new version. This eliminates the long-lived-token attack surface entirely and is
the strongest posture crates.io offers.

**Recommended:** enable it — podbox releases exclusively over CI + OIDC, so nothing legitimate uses a
token. The one caveat: it disables the local token escape hatch below, so a hand-publish
([Manual release if CI is down](#manual-release-if-ci-is-down)) requires temporarily unchecking the
box first. It only affects *new* versions; the existing publish is untouched.

## SemVer policy

For a library crate, `cargo-semver-checks` gates public-API compatibility and runs natively inside
release-plz. Run it locally with `./scripts/release semver-check`. A binary-only crate has no public
API to check but still follows semantic versioning for its releases.

## Routine automated release

**Branch model:** `develop` is the integration branch and the release trigger; `master` is the
released-code mirror. `.github/workflows/release.yml` runs release-plz on `develop`, and its
`promote` job fast-forwards `master` onto the release tag.

> **You do not hand-create the tag.** release-plz creates the tag `vX.Y.Z` for you when you merge its
> release PR. Your only actions are two merges — feature work into `develop`, then the release PR.
> The tag, the crates.io publish, and the `master` promotion are all automated after that second
> merge.

```text
merge feature branch ─▶ develop        (you)
                          │
                          ▼
        release-plz opens the "release PR"     (auto: version bump + changelog)
                          │
                          ▼
              merge the release PR      (you ← the only release decision)
                          │
                          ▼
   tag vX.Y.Z + cargo publish over OIDC        (auto: release-plz)
                          │
                          ▼
     promote job fast-forwards master ─▶ vX.Y.Z    (auto: needs release-plz)
```

1. Merge feature work to `develop`. Commit messages must be
   [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `feat!:`) — that is
   what release-plz reads to pick the next version and write the changelog.
2. release-plz opens/updates the release PR on `develop` (version bump in `Cargo.toml`/`Cargo.lock` +
   `CHANGELOG.md`). It appears automatically on every push to `develop`.
3. Review the PR; merge it. **This is the release gate — the one human decision.**
4. release-plz tags `vX.Y.Z` and publishes to crates.io over OIDC (no stored token).
5. The `promote` job (`needs: release-plz`) fast-forwards `master` onto the tag `vX.Y.Z`.

### Worked example

Starting state: `podbox` is at `0.1.0`; `develop` and `master` both point at the `0.1.0` commit. You
merge two feature branches into `develop`:

```text
feat: add --json output flag
fix: skip empty archives
```

The `feat:` is the highest-ranked change, so release-plz proposes a **minor** bump, `0.1.0 → 0.2.0`,
and opens a PR titled `chore: release podbox 0.2.0` containing the version bump and a `CHANGELOG.md`
entry. You review and merge it. From that merge, everything is automatic:

```text
before                         after merging the release PR

develop  ● 0.1.0               develop  ●── 0.1.0
master   ● 0.1.0                          ╲
                                           ●── feat/fix commits
                                            ╲
                                             ● 0.2.0  ◀─ tag v0.2.0  (release-plz)
                                                      └▶ cargo publish 0.2.0 (OIDC)
                               master   ●───────────▶ ● 0.2.0  (promote: --ff-only)
```

`master` now points at the exact commit tagged `v0.2.0` and published to crates.io — no drift, linear
history. The next release repeats the loop from `develop`.

You can drive release-plz locally instead of waiting for CI — see
[Local operator release](#local-operator-release).

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
3. If [Require trusted publishing](#require-trusted-publishing-hardening) is enabled, temporarily
   **uncheck** it on <https://crates.io/crates/podbox/settings> — otherwise crates.io rejects the
   token-based publish. Re-enable it once CI is healthy again.
4. Ensure auth is configured (`cargo login`).
5. `./scripts/publish`.

## Yank and rollback

A published version cannot be overwritten or deleted, only yanked:

- `cargo yank --version X.Y.Z` — prevent new dependents from selecting it.
- `cargo yank --version X.Y.Z --undo` — reverse a yank.

Fix forward by publishing a new patch version.
