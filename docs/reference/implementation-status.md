# Implementation status

> Reference (lookup). The single source of truth for **what podbox actually does today**
> versus what the [product contract](spec/README.md) specifies but hasn't landed yet. The
> contract in [`spec/`](spec/README.md) describes the target; this page tracks the gap. When
> the two disagree, the contract is the intended behaviour and this page is the current
> reality — other docs link here instead of restating status (and drifting from it).

## Summary

podbox today is a complete **configuration, composition, image-freshness, diagnostics, and
lifecycle-planning** surface. Every command parses, validates, and reports as specified, and
the podman/krun runtime adapter constructs and runs the real hardened `podman build` /
`podman run --runtime=krun …` command lines. What is **not** wired yet is the **live guest
transport** — the in-VM agent that a running sandbox needs for interactive shells, `exec`,
in-guest network-policy enforcement, and clean stop/remove. Anything that rides that
transport returns a stable error:

```
live guest transport is not implemented in this round
```

`src/bin/podbox-guest.rs` is a stub, so `workspace shell`/`exec`/`up`/`down` and the
*execution* half of `workspace reconcile` cannot complete against a live sandbox yet. The
**`--dry-run` planning** paths of `reconcile` and `down` are fully implemented and tested
(note that `up` always reconciles live and has no dry-run — use `reconcile --dry-run` to
preview).

## Legend

- **Working** — implemented and covered by integration tests; usable today.
- **Partial** — the command works but a specific path (usually live execution) is deferred.
- **Deferred** — surface, flags, and errors exist, but the behaviour needs the guest
  transport / guest agent that is not implemented in this round.

## Command surface

| Command | Status | Notes |
| --- | --- | --- |
| `init` | Working | Scaffolds the XDG config tree (`config.toml`, `manifests/`, `devcontainer/` layers); `--starter {minimal,none}`, `--dry-run`, `--json`; ends with a `doctor` report. |
| `config paths` / `show` / `validate` / `get` / `set` / `unset` | Working | Reads/edits `config.toml`, preserving unknown keys (forward-compat); `show` reports per-key provenance. |
| `manifest list` / `show` / `validate` / `compose` | Working | `compose` merges layer `devcontainer.json` fragments deterministically, unions network allowlists, writes the composed artifact to the cache, and prints `composed:`/`digest:`. |
| `network show` | Working | Reports `posture: deny` and the effective allowlist (union of manifest + layer + user config). |
| `network allow <entry>` | Working | Adds a `host:port`/CIDR entry to the user config (or `--manifest-target` the manifest); idempotent. Composed into policy; **not enforced in-guest yet** (see below). |
| `doctor` | Working | Runs the diagnostic catalog across `host`/`runtime`/`config`/`workspace`/`images`/`network` scopes; human + `--json`. `RT-NETBACK` fails **by design** until in-guest enforcement lands (see [08-doctor-and-diagnostics](spec/08-doctor-and-diagnostics.md)). |
| `status` | Working | Reports workspace state, manifest, config sources, runtime profile, hardening posture, image freshness, drift, and relaxations. |
| `version` | Working | Version plus build SHA/date (`--json`). |
| `completion <shell>` | Working | bash/zsh/fish completions via `clap_complete`. |
| `manpage` | Working | Hidden; emits roff via `clap_mangen`. |
| `image list` / `inspect` / `status` | Working | List image dirs; `inspect` prints the built digest or fails (`not built`, exit 6). |
| `image build [NAME…]` | Working | Computes the source-graph freshness proof and, when a rebuild is needed, invokes the real `podman build` child. `--all`, `--full-rebuild`, `--pull-policy`, `--dry-run`. Needs `podman` on `PATH`. |
| `image prune` | Working | Deletes cached image metadata; requires `--yes`. |
| `workspace reconcile --dry-run` | Working | Plans the reconcile steps (compose → ensure-image → remove-old → create → apply-policy → hooks → summary) and reports them without mutation. |
| `workspace up` | Deferred | Always performs a **live** reconcile (does not honor `--dry-run`); composes and builds the image, then fails at `apply-network-policy`. Use `workspace reconcile --dry-run` to preview. |
| `workspace reconcile` (live) | Deferred | Composes and builds the image, then fails at `apply-network-policy`: in-guest policy handoff rides the guest transport (`Unavailable`). |
| `workspace down` (`--dry-run`) | Working | Plans `stop-sandbox` / `remove-sandbox`; enforces the destructive-refusal guard (needs `--yes` when non-interactive). |
| `workspace down` (live) | Deferred | `stop_workspace` / `remove_workspace` return the guest-transport error. |
| `workspace shell` / `exec` | Deferred | Require the live guest channel (interactive/argv exec into the VM). |

## Capabilities and infrastructure

| Capability | Status | Notes |
| --- | --- | --- |
| Layered config + XDG layout | Working | `figment`-based precedence (`--config` > env `PODBOX_*` > project); unknown keys preserved. |
| Deterministic composition + digest freshness | Working | Content digests (not mtimes); fails closed. |
| Image freshness / change detection | Working | Digest of the source graph + pull policy; never serves a stale image as fresh. |
| State store | Working | File-locked per-workspace JSON state under `$PODBOX_STATE_HOME`; serialized mutations; `Failed`-on-error recovery. |
| Hardened runtime command lines | Working | `podman run --runtime=krun --security-opt=no-new-privileges --cap-drop=ALL --network=none …`, tmpfs-only ephemeral credentials, golden-argv snapshot tested. Non-Linux hosts report `Unsupported`. |
| Human/JSON output + `0`–`7` exit taxonomy | Working | stdout = data, stderr = diagnostics; `--json` carries `schema_version`. |
| Network policy artifact | Partial | The default-deny + allowlist policy is composed and serialized (`net_policy`), but `apply_network_policy` is `Unavailable` — the guest agent that enforces it in-VM is not implemented, so `RT-NETBACK` fails by design. |
| Live guest transport (guest channel) | Deferred | Protocol is defined (`PBXG`, framed argv, no `sh -c`), but no adapter drives it and `podbox-guest` is a stub. Gates shell/exec/live stop-remove/in-guest enforcement/lifecycle hooks. |
| Lifecycle hooks | Deferred | Listed as a reconcile step; execution rides the guest transport. |

## What this means in practice

You can, today, `init` a project, author manifests and layers, `compose` them into a
verified `devcontainer.json`, manage the egress allowlist, build images, and run `doctor`
and `status` to see exactly what a reconcile *would* do. You cannot yet open an interactive
sandbox shell or have podbox run a workspace end-to-end against a live microVM — that lands
with the guest agent.
