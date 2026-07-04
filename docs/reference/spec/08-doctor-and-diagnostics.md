# Doctor and diagnostics

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Normative. Defines the first-class `doctor` command: its check catalog, severity model, diagnostics
> quality bar, output and exit contract, and post-command invocation. Command names, flags, and exit
> codes are used verbatim from [`02-command-surface.md`](02-command-surface.md); every hard rule here
> is collected in [`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md). A section
> labelled **Reference Evidence** is non-normative.

## 1. Purpose

`doctor` is a **first-class** command, not an afterthought. It MUST check **all** program requirements
— host OS, hardware-virtualization access, user permissions, runtime/image-builder capability,
filesystem/XDG layout, config and manifest validity, cache/state consistency, workspace identity,
network policy, credential policy, terminal support, and known-incompatible environment conditions —
and MUST NOT limit itself to probing a single runtime executable path (improvement 3, invariant
[U3](11-invariants-and-guarantees.md)).

The design intent: a user or agent facing any readiness problem runs `podbox doctor` and receives a
grouped report that names each unmet requirement precisely and tells them exactly how to fix it.
Every failure MUST be actionable; a generic "unavailable" is a spec violation (see §4).

Synopsis (from [`02-…` §4.3](02-command-surface.md)):

```text
podbox doctor [--scope all|host|runtime|config|workspace|images|network] [--json] [--quiet]
```

`--scope` narrows the run to one category group; the default is `all`. `--json` emits stable
structured diagnostics; `--quiet` suppresses per-check progress and prints only the summary plus any
failures.

## 2. Check categories

Checks are grouped into categories, each mapped to a `--scope` value. Every check has a **stable check
ID** (e.g. `HOST-KVM`) that MUST NOT change across releases once published, so automation and docs
references can rely on it. The catalog below is normative in its coverage; an implementation MAY add
further checks but MUST cover every row.

| Check ID       | Category (`--scope`) | What it verifies                                                                                   | Severity when unmet | Example remediation                                                                                     |
| -------------- | -------------------- | -------------------------------------------------------------------------------------------------- | ------------------- | ------------------------------------------------------------------------------------------------------- |
| `HOST-OS`      | host                 | The host OS/kernel is a supported platform for the required capability classes.                    | fail                | "Unsupported kernel `<x>`; podbox requires a Linux kernel providing KVM-class virtualization or a documented platform equivalent." |
| `HOST-KVM`     | host                 | The current user has read/write access to the hardware-virtualization device (`/dev/kvm`-class).   | fail                | "No rw access to the virtualization device; add your user to the owning group and re-login, or grant a device ACL." |
| `HOST-NESTED`  | host                 | CPU advertises nested-virtualization hints when running inside a VM.                                | warning             | "No `vmx`/`svm` hint; you may be in a VM without nested virt — the microVM may run slowly or not at all." |
| `PERM-USERNS`  | host                 | User-namespace mapping works (multi-line uid/gid map; subuid/subgid range allocated).              | fail                | "User namespaces are not usable; allocate a subuid/subgid range for your user and re-initialize the runtime." |
| `PERM-CGROUP`  | host                 | The host provides the cgroup version and manager the rootless runtime requires.                    | fail                | "Rootless operation needs cgroups v2 with the systemd manager; enable v2 and set the cgroup manager."   |
| `RT-ISOLATION` | runtime              | A KVM-class hardware-virtualization microVM isolation boundary is available and selectable.        | fail                | "No microVM isolation boundary is available; install a runtime stack that provides one (see `07-…`)."   |
| `RT-ROOTLESS`  | runtime              | The container runtime is present and operable rootless.                                            | fail                | "Rootless container runtime not operable; install/configure a rootless OCI runtime."                    |
| `RT-SMOKE`     | runtime              | A minimal sandbox can be created and started under the microVM runtime end-to-end.                 | fail                | "MicroVM smoke test failed: `<captured runtime stderr>`; reproduce with `<exact command>` and check virtualization-device permissions." |
| `RT-BUILDER`   | runtime              | An OCI-image builder capable of building the declared images is available.                         | fail                | "Image-builder capability missing; install a rootless OCI image builder."                               |
| `RT-NETBACK`   | runtime              | The runtime's network backend supports the egress-allowlist enforcement pattern.                   | fail                | "Network backend `<x>` cannot enforce the egress allowlist; configure a supported backend."             |
| `FS-XDG`       | config               | The four XDG roots (config/cache/state/data) exist, are the expected type, and are writable.       | fail                | "Config root `<path>` is missing or not writable; run `podbox init` or fix permissions."                |
| `FS-LAYOUT`    | config               | The config tree obeys the layout: `devcontainer/` layers only, `manifests/` manifests only.        | fail                | "`devcontainer/<x>.yaml` is a manifest in the layers directory; move it to `manifests/` (see `03-…`)."   |
| `CFG-SYNTAX`   | config               | `config.toml` parses as its declared format family.                                                | fail                | "`config.toml` line `<n>`: `<parse error>`; fix the syntax."                                             |
| `CFG-SCHEMA`   | config               | `config.toml` satisfies its schema (types, required keys, enum values).                            | fail                | "`[runtime].profile = '<x>'` is not a known profile; use `microvm` or a profile your runtime advertises." |
| `MAN-SCHEMA`   | config               | Each manifest under `manifests/` satisfies the manifest schema.                                    | fail                | "Manifest `<name>` is missing the required ordered `layers` array; add it (see `04-…`)."                 |
| `MAN-LAYERREF` | config               | Every layer named by every manifest resolves to a real layer directory.                            | fail                | "Manifest `<name>` references layer `<x>` with no `devcontainer/<x>/`; create the layer or fix the name." |
| `MAN-COMPOSE`  | config               | Each manifest composes to a valid `devcontainer.json` (per-fragment + composed-result validation). | fail                | "Manifest `<name>` composes to an invalid result: `<validation error>`; fix the offending layer key."   |
| `STATE-CACHE`  | config               | Cache and state are internally consistent (composed/build freshness metadata is readable/coherent).| warning             | "Composition cache for `<manifest>` is unreadable; it will be recomposed on next use — no action needed unless it recurs." |
| `WS-IDENTITY`  | workspace            | Registered workspaces have a stable, non-colliding identity, and each maps to a resolvable config. | fail                | "Workspace `<path>` has no resolvable config and no registry entry; run `podbox init` or pass `--config`." |
| `IMG-FRESH`    | images               | Declared images are present and provably built from their current source graph.                    | warning             | "Image `<name>` is stale against its source graph; it will rebuild on next `image build`/`reconcile`, or run `podbox image build <name>`." |
| `NET-POLICY`   | network              | The resolved egress policy is well-formed and enforceable (default-deny plus a valid allowlist).   | fail                | "Egress allowlist entry `<x>` is malformed; correct it in the manifest `network.allow`."                |
| `CRED-POLICY`  | network              | No long-lived credential directory is configured for live-mount; a control socket is not mounted.  | fail                | "A long-lived credential directory is set to live-mount; remove it and use scoped credential injection (see `07-…`)." |
| `TERM-TTY`     | host                 | Terminal/TTY capability needed for interactive `shell` is present (or its absence is reported).    | warning             | "No interactive TTY detected; interactive `podbox shell` will fail — use `--reconcile always` and argv `exec` for headless runs." |
| `ENV-KNOWN`    | host                 | No known-incompatible environment condition is present (unsupported nesting, conflicting config).  | warning             | "Detected `<known-bad condition>`; see the referenced docs for the supported configuration."            |

The `--scope host|runtime|config|workspace|images|network` selector runs the checks whose category
column matches; `--scope all` runs the full catalog.

## 3. Severity levels

Each check reports exactly one severity:

| Severity  | Meaning                                                                                          | Affects exit? |
| --------- | ------------------------------------------------------------------------------------------------ | ------------- |
| `pass`    | The requirement is met.                                                                           | No            |
| `warning` | A non-blocking concern; podbox can still operate, possibly degraded.                              | Only in strict mode |
| `fail`    | A **hard** requirement is unmet; dependent operations cannot succeed until it is fixed.           | Yes           |
| `skipped` | The check did not run (out of `--scope`, or a prerequisite check failed so this one is moot).     | No            |
| `unknown` | The check could not determine a result (a required probe input was itself unavailable).           | No, but MUST be surfaced |

Rules:

- A `fail` MUST correspond to a hard requirement whose absence blocks the dependent capability.
- A check MUST NOT report `pass` when it could not actually verify the requirement; it MUST report
  `unknown` instead. `unknown` MUST carry a message explaining what could not be determined and why.
- When a check is `skipped` because a prerequisite failed, its message SHOULD name the prerequisite
  check ID so the reader fixes the root cause first.

## 4. Diagnostics quality

This is the load-bearing quality bar (invariant [U3](11-invariants-and-guarantees.md)):

- Every `fail` and every `warning` MUST name the **exact missing requirement** and MUST include a
  **concrete remediation path** — a command to run, a file/permission to change, or a documented
  reference — not a generic status word.
- A message of the form "runtime unavailable", "check failed", or "not found" **without** the specific
  missing artifact and a concrete next step is **non-compliant**.
- Remediation MUST be phrased against **capability classes** and user-facing artifacts, not a mandated
  tool; where a concrete tool aids the user, it MUST be marked as an illustrative example (consistent
  with [`README` neutrality rule](README.md) and invariants [N1–N2](11-invariants-and-guarantees.md)).
- Where a probe captured underlying evidence (e.g. runtime stderr from a smoke test), the failure
  message SHOULD include that evidence and a **reproduction command** so the user can re-run the exact
  failing step.

## 5. Output and exit behavior

### 5.1 Human output (default)

The default is a **human-readable grouped report**: checks grouped by category, each line showing the
check ID, a severity marker, and the one-line label; failures and warnings additionally print their
remediation. A trailing summary states counts per severity and the overall result. Output stream
discipline follows [`10-errors-output-and-scriptability.md`](10-errors-output-and-scriptability.md):
the report is diagnostic and MUST go to stderr when `--json` occupies stdout; in default human mode the
report is the command's primary output.

### 5.2 Machine output (`--json`)

`--json` MUST emit **stable, structured diagnostics** on stdout. Each check entry MUST include:

- `id` — the stable check ID (§2),
- `category` — the `--scope` category,
- `status` — one of the severities in §3,
- `message` — the specific human-readable finding,
- `evidence` — captured probe detail (MAY be empty),
- `remediation` — the concrete fix (present for `fail`/`warning`),
- `docs` — a documentation reference (e.g. a section anchor in this shelf).

The top-level object MUST carry a schema/version field and per-severity counts. JSON output MUST NOT
contain color or control characters (see [`10-…`](10-errors-output-and-scriptability.md)).

### 5.3 Exit behavior

- `doctor` MUST exit **non-zero if any hard check fails**. The exit code MUST be the most specific
  applicable code from the [`02-…` §5 taxonomy](02-command-surface.md); an unmet host/runtime
  requirement maps to exit code **`3`**. Config/manifest validation failures surfaced by `doctor` map
  to **`4`**; a `config.toml` syntax failure maps to **`2`**.
- `warning`-severity checks MUST NOT cause a non-zero exit **unless a strict mode is requested** (e.g.
  a `--strict`/strict-config setting), in which case warnings are treated as failures for exit
  purposes and MUST be documented as such.
- `skipped` and `unknown` MUST NOT by themselves cause a non-zero exit; `unknown` on a hard-check
  requirement SHOULD be surfaced prominently because it hides a potential failure.

## 6. Post-command checks

Post-step invocation is a first-class behavior (improvement 4, invariant
[U3](11-invariants-and-guarantees.md)):

1. **`init` MUST run `doctor` afterward** (unless `--dry-run`) and surface its diagnostics
   ([`02-…` §4.2](02-command-surface.md)). This guarantees a freshly-initialized user immediately
   learns of any unmet host/runtime requirement.
2. **Any command that materially changes runtime, config, or build state** — at minimum the first
   `workspace up` for a workspace, `workspace reconcile`, and `image build` — MUST run a **scoped**
   `doctor` check afterward, **or**, when an automatic check would be too expensive to run inline,
   MUST print the exact command the user should run (e.g. `podbox doctor --scope runtime`). The scope
   chosen SHOULD match what the command touched (runtime bring-up → `--scope host,runtime`; config
   edit → `--scope config`; build → `--scope images`).
3. **Report-vs-fail policy.** A post-step `doctor` finding is **reported without necessarily failing a
   primary command that already succeeded.** If the primary operation completed, a post-step
   `warning`/`fail` MUST be printed clearly (and included in `--json` output for that command) but the
   command's exit code reflects the **primary** operation's success. A post-step check MUST NOT be
   used to retroactively fail an already-completed mutation; instead it warns the user that a
   dependent future operation may fail and points to `podbox doctor` for the full picture. When a
   command's success genuinely **depends** on a readiness condition (e.g. `workspace up` cannot bring
   a sandbox up without the isolation boundary), that condition is a **pre-step** hard requirement and
   fails the command directly with exit `3` — it is not a post-step.

## Reference Evidence (non-normative)

The legacy `dctl doctor` implements a 14-probe host preflight scoped to libkrun + rootless Podman
readiness; podbox generalizes it to all program requirements and adds stable check IDs and `--scope`.
Mapping of legacy probes to podbox check IDs:

- `lib/dctl/commands/doctor/_dispatch.sh` — sequential probes 1..14 with a printed summary and
  pass/fail exit; the ordering note explains why `podman_info.sh` groups non-consecutive probes 3 and
  10.
- `lib/dctl/commands/doctor/crun_libkrun.sh` — probe 1 (`crun` built with `+LIBKRUN`) and probe 2
  (`krun` → `crun` symlink): legacy analogues of `RT-ISOLATION`/`RT-ROOTLESS` prerequisites, and a
  model for **specific** remediation strings ("do not check `crun --help` for `--krun`").
- `lib/dctl/commands/doctor/libkrun.sh` — probe 4 (libkrun `>= MIN_LIBKRUN_VER`): version-floor
  remediation, analogue of a `RT-ISOLATION` sub-check.
- `lib/dctl/commands/doctor/kvm.sh` — probes 5/5b/6 (`/dev/kvm` rw, POSIX ACL, `kvm` group): the
  concrete evidence for `HOST-KVM`, including the "group membership is not retroactive; `newgrp` or
  re-login required" remediation detail.
- `lib/dctl/commands/doctor/subid.sh` and `userns.sh` — probes 7/12/13 (`/etc/subuid`+`/etc/subgid`
  range, multi-line `uid_map`, `unprivileged_userns_clone`): the evidence for `PERM-USERNS`.
- `lib/dctl/commands/doctor/cgroups.sh` — probes 8/9 (cgroups v2, systemd cgroup manager): evidence
  for `PERM-CGROUP`.
- `lib/dctl/commands/doctor/podman_info.sh` — probe 3 (create+start+inspect smoke), probe 10 (default
  OCI runtime): the end-to-end evidence for `RT-SMOKE`, including capturing runtime stderr and
  printing a reproduction command — the model for §4's evidence + reproduction requirement.
- `lib/dctl/commands/doctor/network_backend.sh` — probe 11 (rootless network backend): evidence for
  `RT-NETBACK`.
- `lib/dctl/commands/doctor/nested_virt.sh` — probe 14 (`vmx`/`svm` cpuinfo hints, `warning`
  severity): the direct model for `HOST-NESTED`.
