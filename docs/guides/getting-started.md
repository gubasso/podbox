# Getting started

> Guide (task). A hands-on walkthrough of the podbox surface that works today: scaffold a
> project, compose devcontainer layers, manage the egress allowlist, and read the health and
> status reports. For *what* podbox is, see the [product contract](../reference/spec/README.md);
> for *what is and isn't wired yet*, see [implementation-status](../reference/implementation-status.md).

## Prerequisites

- podbox installed (`cargo install podbox`, or `cargo install --path .` from a checkout).
- For actually starting a sandbox: Linux with `/dev/kvm` and rootless Podman + the `krun`
  runtime. The steps below do **not** require a running sandbox — they exercise the
  config/compose/diagnostics surface, which works on any host podbox builds on.

podbox keeps all of its state under an XDG layout. To follow along in a throwaway location
without touching your real config, point `PODBOX_HOME` at a scratch directory:

```bash
export PODBOX_HOME="$(mktemp -d)"
```

## 1. Scaffold a project

```bash
podbox init
```

This writes a lean, minimal starter and finishes with a `doctor` report:

```
starter: minimal
...
doctor: <summary>
```

It creates the config tree under `$PODBOX_HOME/config/`:

- `config.toml` — defaults, runtime profile (`microvm`), image pull policy, and a
  default-deny `[network]` block.
- `manifests/minimal.toml` — a manifest stacking two layers: `layers = ["base", "minimal"]`.
- `devcontainer/base/devcontainer.json` — the base layer (`image: alpine:latest`).
- `devcontainer/minimal/devcontainer.json` — the leaf layer.
- `images/minimal/devcontainer.json` — the image source.

Use `--starter none` for just the directories and `config.toml`, or `--dry-run --json` to
preview the plan without writing anything.

## 2. See where things live

```bash
podbox config paths
```

```
config: $PODBOX_HOME/config
cache: $PODBOX_HOME/cache
state: $PODBOX_HOME/state
data: $PODBOX_HOME/data
```

`podbox config show` reports the resolved config values with per-key provenance; `config get`
/ `set` / `unset` read and edit individual keys while preserving any unknown keys you've
added (forward-compatibility).

## 3. Compose the manifest

A manifest is an ordered stack of devcontainer layers. Composition merges them
deterministically into one `devcontainer.json` and records a content **digest**:

```bash
podbox manifest compose minimal
```

```
composed: $PODBOX_HOME/cache/composed/minimal/devcontainer.json
digest:   sha256:...
```

The digest is content-addressed (not mtime-based) and fails closed, so a composed artifact is
never served as fresh unless it matches its inputs. `podbox manifest list` / `show` / `validate`
round out manifest inspection.

## 4. Manage the egress allowlist

Sandboxes are **default-deny** for network egress. Inspect the effective policy — the union of
the manifest, its layers, and your user config:

```bash
podbox network show
```

```
posture: deny
allow: []
```

Relax it by allowing a specific `host:port` (or CIDR):

```bash
podbox network allow github.com:443
# → network allowlist updated; run `podbox workspace reconcile` to apply
podbox network show
```

```
posture: deny
allow: github.com:443
```

By default the entry goes into your user config; pass `--manifest-target` to write it into the
manifest instead (so it travels with the project). The policy is composed and serialized for
the runtime, but **in-guest enforcement is not wired yet** — `doctor` reports this as
`RT-NETBACK` (see [implementation-status](../reference/implementation-status.md)).

## 5. Check health and status

```bash
podbox doctor
```

`doctor` runs a stable catalog of checks across host/runtime/config/workspace/images/network
and prints a report (add `--json` for machine output, or `--scope config` to narrow it). On a
host without a KVM microVM runtime some `RT-*`/`HOST-*` checks will `fail` — that's expected
off a Linux+krun host.

```bash
podbox status
```

`status` reports the workspace state (`absent` until you reconcile), the active manifest, config
sources, runtime profile and hardening posture, image freshness, drift, and any relaxations.

## 6. (Optional) build an image

If `podman` is on your `PATH`, podbox can build the image referenced by the composition:

```bash
podbox image build --all         # or: podbox image build minimal
podbox image inspect minimal     # prints the built digest, or "not built"
```

`image build` computes a source-graph freshness proof and only invokes `podman build` when a
rebuild is actually needed; `--dry-run` reports the decision without building.

## What you can't do yet

Opening an interactive sandbox and running commands inside it needs the **live guest
transport**, which is not implemented in this round. These are deferred:

```bash
podbox shell                 # / podbox workspace shell
podbox workspace exec -- ...
podbox workspace up          # always reconciles live (no dry-run)
podbox workspace reconcile   # live; --dry-run plans only
podbox workspace down        # live; --dry-run plans only
```

The **`--dry-run` planning** paths work — e.g. `podbox workspace reconcile --dry-run` shows the
reconcile steps podbox would run, and `podbox workspace down --dry-run` shows the teardown steps.
Track the full picture in [implementation-status](../reference/implementation-status.md).
