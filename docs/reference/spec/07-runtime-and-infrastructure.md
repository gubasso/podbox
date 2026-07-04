# Runtime and infrastructure requirements

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Normative. Defines the **capability classes** a conforming podbox runtime and host MUST provide,
> the isolation boundary, the runtime-adapter contract, native devcontainer orchestration, and the
> security posture. Every requirement is phrased as a capability, never as a named tool. Concrete
> tools appear **only** as explicitly-marked illustrative examples or in **Reference Evidence**, both
> non-normative.

The **isolation rationale itself is not re-argued here** — the threat model, the runtime catalog, the
microVM decision, and the known runtime pitfalls are cited to the vendor-neutral backend reference
shelf at `~/DocsNNotes/tech/infra/sandbox-isolation-backends/`. This document requires the
**capability** and cites that shelf for *why*. It elaborates isolation invariants **S1**–**S6**,
execution invariants **X1**–**X2**, and neutrality invariants **N1**–**N3** from
[`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md).

## 1. Capability requirements

A conforming implementation MUST provide each capability class below. Any concrete tool named is an
**illustrative example only** (see §8), never a mandate (invariants **N1**, **N2**).

1. **Hardware-virtualization microVM isolation boundary.** The primary boundary MUST be a **KVM-class
   hardware-virtualization microVM** — a guest running its **own kernel**, not a shared-kernel
   namespace container. A serious host-compromise attempt MUST require a hypervisor-class bug, not a
   routine syscall trick or a kernel LPE. *Rationale — cited, not restated:* shared-kernel namespaces
   are a resource-control mechanism, not a security boundary, and the recurring container-breakout /
   kernel-LPE cadence makes them unacceptable as the primary boundary for adversarial or agent-run
   code (backend shelf `00-threat-model-and-principles.md`, `20-decision-libkrun-linux.md`).
2. **OCI-image-consuming, rootless-operable container runtime.** The runtime MUST consume existing
   OCI image artifacts directly (no bespoke image format the user must author) and MUST be operable
   **rootless** (invariant **S2**), so an in-guest escape lands as the invoking user, not host root.
3. **Native `devcontainer.json` interpreter driving lifecycle by argv-vector exec.** The
   implementation MUST parse and compose the supported `devcontainer.json` subset itself and drive
   lifecycle by **argv-vector exec** — never a multi-line `sh -c` entrypoint (invariant **X1**; see
   §5).
4. **Per-workspace filesystem mounts.** The runtime MUST mount the workspace (and only the declared
   mounts) into the sandbox, keyed to a stable workspace identity so work-clones stay separate
   (invariant **I1**, [`05-workspace-lifecycle-and-shell-ux.md`](05-workspace-lifecycle-and-shell-ux.md)).
5. **Default-deny / allowlisted egress.** The runtime MUST support a **default-deny** outbound
   posture with an allowlist composed from manifest + user policy, enforced in-guest (invariant
   **S4**; [`03-config-and-xdg-layout.md`](03-config-and-xdg-layout.md) §4).
6. **Scoped credential injection.** The runtime MUST support injecting credentials as **scoped,
   short-lived, ephemeral** values rather than live-mounting long-lived credential directories
   (invariant **S5**; §6).
7. **Interactive-TTY and noninteractive-exec support.** The runtime MUST support both an interactive
   login-capable shell session (for `podbox shell`) and noninteractive argv-vector command execution
   (for `podbox workspace exec` and lifecycle hooks).
8. **Resource controls.** The runtime MUST honor advisory resource budgets (for example the
   `memory_mib` / `cpus` budgets from [`03-…`](03-config-and-xdg-layout.md) §3, §4) as bounds on the
   sandbox.

## 2. Technology-neutral boundary

The isolation boundary is specified by **capability class**, not by a named virtual-machine monitor
(invariants **N1**, **N2**):

- On **Linux**, the boundary MUST be a **KVM-class** hardware-virtualization microVM.
- On **other platforms**, the boundary MUST be a **platform-equivalent hardware-virtualization**
  mechanism of the same class (for example an OS-native hardware-virtualization API). No specific VMM
  or runtime is mandated as the only compliant implementation.
- The user-facing authoring surface (`devcontainer.json`, manifests, `config.toml`) MUST stay
  runtime-agnostic and MUST NOT name a runtime; runtime selection is a **capability profile**
  (`runtime.profile`, default `microvm`; see [`03-…`](03-config-and-xdg-layout.md) §3, §4), not a
  hard-coded tool enum.

## 3. Fallbacks

Any fallback that does **not** provide a hardware-virtualization boundary (for example a
userspace-kernel sandbox on a host without hardware virtualization) is a **lower-assurance** boundary
class. Per invariant **S1**, such a fallback:

- MUST be **explicitly labeled** as lower-assurance wherever it is selected or reported (in `doctor`
  and `status`), and
- MUST NOT be **silently selected** as equivalent to the primary boundary.

*Rationale — cited:* the backend shelf accepts a no-hardware-virt fallback only where the workload is
non-adversarial (for example CI running the project's own test code), precisely because the boundary
class is weaker; it is never the default for agent-run or untrusted code (backend shelf
`20-decision-libkrun-linux.md`, `00-threat-model-and-principles.md`).

## 4. Runtime adapter concept

Runtime-specific behavior MUST be confined behind a **runtime adapter**: a thin, swappable external
contract expressed as a small set of operations, so that swapping the underlying runtime is swapping
one adapter and nothing else. This document specifies the adapter as a set of **operations**, not as
an implementation module (invariant **N3** — the code that implements them is unspecified):

| Operation | Responsibility                                                                    |
| --------- | --------------------------------------------------------------------------------- |
| `run`     | Start a sandbox for a workspace from its resolved config under the active runtime.|
| `exec`    | Execute a command **as an argv vector** inside a running sandbox.                 |
| `list`    | Enumerate sandboxes managed by podbox for a workspace.                            |
| `remove`  | Remove the sandbox(es) managed for a workspace.                                   |
| `build`   | Build a local image from an image definition and context.                        |
| `inspect` | Report whether an image or sandbox exists / its identity and state.              |
| `logs`    | Retrieve sandbox output (and any equivalent copy/attach helper an impl needs).   |

Everything outside the adapter — composition, policy defaults, lifecycle dispatch, diagnostics — MUST
stay runtime-agnostic. The adapter is the **only** place a concrete runtime is named.

## 5. Native devcontainer orchestration

podbox MUST natively parse and compose the **supported `devcontainer.json` subset** and drive
lifecycle itself; it MUST NOT delegate to a heavyweight orchestrator whose keep-alive shim would
violate the argv-vector requirement (invariants **X1**, **X2**). Concretely:

- Sandboxes MUST start on the **image's own entrypoint**; podbox MUST NOT inject a multi-line
  `sh -c` keep-alive shim as the entrypoint.
- Every lifecycle hook and every command into the sandbox MUST be issued as an **argv-vector exec**
  against the running sandbox.

*Rationale — cited, not restated:* under the primary microVM runtime, a multi-line `sh -c` payload
crossing the host-to-guest argv path at container-creation time is mangled (a newline is delivered as
a newline plus a stray literal character on the next line), so a shim-based orchestrator is
**structurally unusable** — it fails to bring any container up, independent of image or config. An
adapter that only ever issues single-line / argv-vector commands is immune by construction. See
backend shelf `50-native-orchestration-decision.md` and the known-issue writeup (`\n`-mangling and
its orchestrator fallout) in `60-podman-libkrun-operational-notes.md`.

## 6. Security requirements

The following defaults are normative (invariants **S3**–**S6**; *rationale* cited to the backend
threat model, `00-threat-model-and-principles.md` §2–§3):

- **No privileged mode by default.** The sandbox MUST NOT run privileged by default, and a container
  control socket MUST NOT be live-mounted.
- **No live-mount of long-lived credential directories.** Long-lived credential/token directories
  MUST NOT be bind-mounted into the sandbox. Credentials MUST be forwarded **scoped, short-lived, and
  ephemeral** (invariant **S5**). This is the highest-frequency, lowest-skill risk in the threat
  model and is a policy/config control independent of runtime choice.
- **Host `/tmp` not mounted as sandbox `/tmp`.** The sandbox `/tmp` MUST be a tmpfs; the host `/tmp`
  MUST NOT be bind-mounted as the sandbox `/tmp` (invariant **S3**).
- **Dropped capabilities and no-new-privileges.** Default hardening MUST drop all capabilities and
  set no-new-privileges (invariant **S3**). A permissive profile is an explicit, named opt-in, never
  the default.
- **Default-deny egress policy.** Outbound access MUST be denied by default and governed by the
  composed allowlist, enforced in-guest (invariant **S4**).
- **Explicit, visible relaxations.** Any relaxation of a security default MUST be explicit, named,
  visible in `doctor`, and reflected in `status` (invariant **S6**).

## 7. Host prerequisites

The host MUST provide the following capabilities for the primary boundary to function. These mirror
the diagnostic probes in [`08-doctor-and-diagnostics.md`](08-doctor-and-diagnostics.md), which reports
each as a specific, remediable check (invariant **U3**):

- **Hardware-virtualization device access** — access to a KVM-class hardware-virtualization device
  (a `/dev/kvm`-class interface on Linux, or the platform equivalent elsewhere).
- **User-namespace / subordinate-ID mapping** — user-namespace support and a configured
  subordinate-UID/GID range sufficient for rootless operation.
- **Control groups** — a cgroups hierarchy the runtime can use to apply resource controls.
- **A network backend supporting the egress-allowlist pattern** — a rootless-capable network backend
  compatible with in-guest default-deny egress enforcement.
- **Nested-virtualization awareness** — detection of whether the host is itself virtualized and
  whether nested hardware virtualization is available, so an unavailable primary boundary is reported
  rather than silently downgraded (see §3).

A missing hard prerequisite MUST cause the depending command to fail with an unmet host/runtime
requirement (exit code `3`; [`02-command-surface.md`](02-command-surface.md) §5) and an actionable
remediation — never a generic "unavailable".

## 8. Illustrative, not mandated

The following concrete stack is an **illustrative example only** and is **NOT mandated** by this
specification. It is named to ground the capability classes above in a known-working reference, cited
to the backend shelf:

- **Primary (Linux) example stack:** rootless Podman driving `crun --krun` with libkrun as the
  KVM-class microVM VMM (backend shelf `20-decision-libkrun-linux.md`).
- **Documented alternatives:** bare Firecracker, or Kata Containers (with Firecracker or Cloud
  Hypervisor), behind the same adapter contract (§4); a userspace-kernel sandbox such as gVisor as a
  **lower-assurance** no-hardware-virt fallback per §3 (backend shelf `10-runtimes-catalog.md`,
  `20-decision-libkrun-linux.md`, `30-libkrun-vs-firecracker.md`).

**Any stack that meets the capability classes in §1–§7 conforms.** No named tool
(Podman, `crun`, libkrun, Firecracker, Kata, gVisor, …) is required, and none may be treated as the
only compliant implementation (invariants **N1**, **N2**). podbox is **not** a wrapper specification
for one backend.

## Reference Evidence (non-normative)

- `docs/ARCHITECTURE.md` — the legacy `dctl` runtime model: every runtime op goes through a rootless
  OCI runtime with the microVM (not a shared-kernel namespace) as the boundary; `dctl` interprets
  devcontainer lifecycle keys itself; default egress is allowlisted; long-lived host credential dirs
  are not live-mounted; default hardening is `--cap-drop=ALL`, `--security-opt no-new-privileges`,
  tmpfs `/tmp`.
- `lib/dctl/runtime/common.sh` — the legacy runtime-adapter interface contract: callers depend only
  on runtime-agnostic `rt_run` / `rt_exec` / `rt_ps` / `rt_rm` / `rt_build` / `rt_image_inspect`
  entrypoints, with runtime-specific code confined behind them (concrete evidence for the adapter
  concept in §4).
- **Backend reference shelf (authoritative for isolation rationale, cited not restated):**
  `~/DocsNNotes/tech/infra/sandbox-isolation-backends/` — `00-threat-model-and-principles.md` (why
  namespaces are not a boundary; the agent threat model; security posture), `20-decision-libkrun-linux.md`
  (the KVM-class decision, egress enforcement, host prerequisites, risks accepted),
  `50-native-orchestration-decision.md` (parse natively, exec by argv vector), and
  `60-podman-libkrun-operational-notes.md` (the `\n`-mangling known issue and its orchestrator
  fallout).
