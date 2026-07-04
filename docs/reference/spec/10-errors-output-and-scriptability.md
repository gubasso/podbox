# Errors, output, and scriptability

> **Theses** — A: *the sandbox is the interface* (`podbox shell`, shown first in help);
> B: *reconcile-on-change is the everyday path* (`workspace reconcile`, first-class & idempotent).

> Normative. Defines the output-stream discipline, machine and human output contracts, the canonical
> exit-code taxonomy, prompting rules, color/TTY behavior, and verbosity levels. RFC-2119 keywords
> (MUST / MUST NOT / SHOULD / MAY) are binding. Sections labelled **Reference Evidence** are
> non-normative.

The overriding goal of this document: an **automation caller MUST be able to drive podbox reliably
using exit codes and `--json` alone**, never by scraping human-readable text. Every rule below serves
that contract. This document restates the exit-code taxonomy from
[`02-command-surface.md`](02-command-surface.md) §5 as the canonical reference; the surface document
and this document MUST agree, and where they appear to differ the taxonomy in
[`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md) governs.

## 1. Output streams

podbox MUST separate output by stream so that data and diagnostics never interleave on the same
channel (invariant **U4**, [`11-…`](11-invariants-and-guarantees.md)):

- **stdout** carries the command's **primary output** — the requested result and all machine data.
  When `--json` is set, stdout carries **only** the JSON document (see §2). A caller MAY redirect
  stdout to a file or a pipe and receive exactly the result, with no progress noise mixed in.
- **stderr** carries **progress, status, warnings, errors, and prompts**. Anything a human reads to
  understand *how* a command is going — spinners, step logs, remediation hints, confirmation prompts,
  and error messages — MUST go to stderr, never stdout.

Rules:

- A command that produces no primary data (for example a successful `workspace down`) MAY write
  nothing to stdout and communicate solely via its exit code plus stderr status.
- Errors and prompts MUST NOT be written to stdout. A script capturing stdout MUST NOT receive
  human-facing prose.
- Progress output on stderr MUST NOT be required for correctness: suppressing it (`--quiet`, or a
  non-TTY stderr) MUST NOT change stdout, the exit code, or any produced file.

## 2. Machine output

`--json` selects a stable, machine-consumable representation on stdout. The contract:

- **Stdout is pure JSON.** Under `--json`, stdout MUST contain exactly one well-formed JSON document
  (or a documented JSON stream, one object per line, for streaming commands) and nothing else — no
  banners, no progress, no trailing human text.
- **No color or control characters.** JSON output MUST NOT contain ANSI color codes, cursor-control
  sequences, or other terminal control characters, regardless of TTY state or color settings (§5).
- **Versioned output objects.** Every top-level JSON object MUST carry a `schema_version` field (an
  integer or documented version string) identifying the output-object version, so callers can pin and
  migrate. Adding this field is mandatory even when the payload is otherwise trivial.
- **Stability rules.** Within a given `schema_version`, field **names and types MUST NOT change or be
  removed**. New optional fields MAY be added without a version bump; a removal or a
  meaning/type change of an existing field MUST bump `schema_version`. Callers MUST tolerate unknown
  fields (forward compatibility), mirroring the config forward-compat rule in
  [`03-…`](03-config-and-xdg-layout.md) §3.
- **Errors in JSON mode.** When `--json` is set and a command fails, podbox SHOULD emit a structured
  error object on stdout (with at least a stable machine `code`, a human `message`, and where
  applicable `remediation`) in addition to setting the exit code, so a JSON caller need not read
  stderr. The exit code remains authoritative.
- **Stable identifiers.** Machine fields that other tooling keys on — `doctor` check IDs
  ([`08-…`](08-doctor-and-diagnostics.md)), workspace identity/label
  ([`05-…`](05-workspace-lifecycle-and-shell-ux.md)), digests
  ([`04-…`](04-manifest-and-composition-model.md), [`06-…`](06-image-builds-and-change-detection.md))
  — MUST be stable across releases within a `schema_version`.

A caller MUST be able to determine success, failure class, and result payload from the exit code plus
the JSON document alone.

## 3. Human output

Default (non-`--json`) output is for a human reader and MUST remain useful without being required for
scripting:

- **Concise by default.** The default rendering MUST be a concise summary of what happened (what was
  composed, built, reconciled, started, or removed), not a verbose transcript. Detail beyond the
  summary belongs behind `--verbose` (§6).
- **Clear next actions.** On both success and failure, human output SHOULD state the concrete next
  action where one exists — for example, pointing a drifted workspace at `podbox workspace reconcile`
  ([`05-…`](05-workspace-lifecycle-and-shell-ux.md)), or telling an unconfigured workspace to run
  `podbox init` or pass `--config` ([`03-…`](03-config-and-xdg-layout.md) §5).
- **No stack traces for expected errors.** Expected, classified failures (usage error, unmet host
  requirement, validation failure, missing workspace, refused destructive action, build failure) MUST
  be reported as a plain, actionable message plus the matching exit code — never as an internal stack
  trace or debugger dump. Raw traces MAY appear only for genuinely unexpected internal faults (exit
  `1`) and SHOULD be gated behind `--verbose`.
- **Actionable diagnostics.** Error messages MUST name the specific problem and, where a remedy
  exists, the specific fix. Generic "unavailable"/"failed" messages are non-compliant, consistent
  with the `doctor` diagnostics-quality rule ([`08-…`](08-doctor-and-diagnostics.md), invariant
  **U3**).

## 4. Exit codes (canonical reference)

This is the canonical restatement of the exit-code taxonomy from
[`02-command-surface.md`](02-command-surface.md) §5. The codes are **stable and documented**;
automation MUST be able to branch on them without parsing any message text.

| Code | Meaning                                                        |
| ---- | ------------------------------------------------------------- |
| `0`  | Success.                                                       |
| `1`  | General/unexpected failure.                                   |
| `2`  | Usage or config **syntax** error (bad flags, unparsable config). |
| `3`  | Unmet **host/runtime requirement** (doctor hard-check failed).|
| `4`  | **Validation** failed (schema/semantic: manifest, layer refs, composed result). |
| `5`  | Workspace **not found / not running** when required.          |
| `6`  | **Build** failed.                                             |
| `7`  | Unsafe/**destructive action refused** (needs `--yes`/`--force`). |

When it applies, in detail:

- **`0` — Success.** The command completed its contract. A no-op that was already in the desired state
  (an idempotent `init`, `workspace reconcile`, or `image build` with nothing to do) MUST also exit
  `0`; idempotent success is still success.
- **`1` — General/unexpected failure.** A failure that does not fit any more specific class: an
  unexpected internal fault or an otherwise-unclassified error. Implementations MUST prefer a more
  specific code (`2`–`7`) whenever one applies; `1` is the residual class, not a default.
- **`2` — Usage or config syntax error.** Malformed invocation (unknown flag, missing required
  argument, mutually exclusive flags) **or** unparsable configuration (`config.toml` or a manifest
  that is not syntactically valid in its format family). Also the exit code when config resolution
  finds nothing and there is no fallback ([`03-…`](03-config-and-xdg-layout.md) §5). Usage errors MUST
  be accompanied by a usage hint on stderr.
- **`3` — Unmet host/runtime requirement.** A required `doctor` hard-check failed: the host lacks a
  capability podbox needs (a KVM-class hardware-virtualization boundary, rootless runtime capability,
  user-namespace/subid mapping, a conforming network backend, etc. — [`07-…`](07-runtime-and-infrastructure.md),
  [`08-…`](08-doctor-and-diagnostics.md)). This is the code `doctor` returns when a hard check fails,
  and the code a runtime-dependent command returns when a **pre-step** host/runtime readiness
  requirement blocks the operation (e.g. `workspace up` cannot proceed without the isolation
  boundary). A **post-step** `doctor` finding after a command has already succeeded is *reported*,
  not fatal — the command's exit code reflects the primary operation, per the report-vs-fail policy
  in [`08-…`](08-doctor-and-diagnostics.md) (and ADR-0011).
- **`4` — Validation failed.** Input parsed but is **semantically** invalid: a manifest whose
  `layers` reference a missing layer directory, a layer or composed `devcontainer.json` that violates
  schema/semantic rules, or a `config.toml` value that parses but fails its validation rule
  ([`03-…`](03-config-and-xdg-layout.md) §3, [`04-…`](04-manifest-and-composition-model.md)). Distinct
  from `2`, which is purely syntactic.
- **`5` — Workspace not found / not running when required.** A command that requires an existing or
  running sandbox (e.g. `workspace exec`, `workspace shell` under `--reconcile never`,
  `workspace status` on demand) found no matching workspace, or the sandbox was not in the required
  running state ([`05-…`](05-workspace-lifecycle-and-shell-ux.md)).
- **`6` — Build failed.** An image build failed during execution — including a build that failed
  because freshness could not be proven and no cached artifact could be safely reused
  ([`06-…`](06-image-builds-and-change-detection.md)). Distinct from `3` (the *host* could not build
  at all) and `4` (the build *inputs* were invalid before building).
- **`7` — Unsafe/destructive action refused.** A destructive command (`workspace down`, `image
  prune`, and any other guarded mutation) was invoked noninteractively without the required
  `--yes`/`--force`, or the confirmation was declined. The action was **refused**, nothing was
  destroyed, and the caller can retry with the bypass flag (invariant **U5**).

Automation guidance: branch on the exit code first, then read the JSON payload for detail. Human
message text MUST NOT be treated as a stable API.

## 5. Prompting

Interactive prompting is a convenience for humans and MUST NOT be a barrier to automation (invariant
**U5**):

- **TTY-gated.** podbox MUST prompt only when the relevant stream is an interactive TTY. When input
  is not a TTY (a pipe, a CI job, an agent harness), podbox MUST NOT block on a prompt.
- **Flags bypass prompts.** Every prompt MUST have a flag that answers it non-interactively —
  `--yes`/`--force` for destructive confirmations, `--reconcile always|never` for reconcile decisions,
  and equivalent selectors for other choices. When the bypass flag is present, no prompt is shown.
- **Noninteractive failure is clear.** A noninteractive invocation that needs input it was not given
  MUST fail immediately with a clear message naming the missing input and the flag that supplies it,
  and MUST exit with the matching code (`7` for a refused destructive action; `2` for other missing
  required input) — it MUST NOT hang waiting on a TTY that will never answer.
- Prompts are written to stderr (§1); a prompt MUST NOT contaminate stdout or `--json` output.

## 6. Color and TTY

- **Color is TTY-gated by default.** podbox MUST colorize human output only when the target stream is
  an interactive TTY and color is enabled. When stdout/stderr is not a TTY (redirected or piped),
  color and other terminal control sequences MUST be suppressed by default.
- **`--no-color` and environment convention.** `--no-color` MUST disable all color. podbox MUST also
  honor the `NO_COLOR` environment convention (any non-empty value disables color), and the
  `defaults.color = auto|always|never` setting in `config.toml`
  ([`03-…`](03-config-and-xdg-layout.md) §3). Precedence, highest first: `--no-color`/explicit CLI
  intent → `NO_COLOR` → `config.toml` `defaults.color` → the `auto` (TTY-detected) default.
- **`--json` is never colorized.** JSON output MUST NOT contain color or control characters under any
  color setting (§2).
- Color MUST be purely cosmetic: removing it MUST NOT change the information content of the output,
  the exit code, or any produced file.

## 7. Logging and verbosity

Two global flags ([`02-…`](02-command-surface.md) §3) tune diagnostic volume on stderr without
affecting stdout data or exit codes:

- **`--quiet` / `-q`.** Suppresses progress and non-essential status on stderr, leaving only errors
  (and, where applicable, final result data on stdout). `--quiet` is the script-safe mode: stdout and
  the exit code MUST be identical whether or not `--quiet` is set.
- **`--verbose` / `-v`.** Increases diagnostic detail on stderr (step-level logs, resolved paths,
  decision rationale such as *why* an image was rebuilt or a cache reused). Verbosity affects stderr
  only; it MUST NOT alter stdout data, `--json` payloads, or the exit code. Implementations MAY accept
  repeated `-v` for higher levels.
- `--quiet` and `--verbose` are mutually exclusive; supplying both is a usage error (exit `2`).
- **Script-safe stability.** For a fixed invocation and state, stdout (and any `--json` document) and
  the exit code MUST be **deterministic and independent of verbosity, color, and TTY state**. Only the
  human-oriented stderr stream varies with these controls. This is the guarantee that lets automation
  rely on exit codes and JSON without scraping human text (invariant **U4**).

## Reference Evidence (non-normative)

- Legacy `dctl` mixes progress and result on the same stream in several commands and lacks a stable
  `--json` contract; podbox's stream discipline and versioned JSON objects are the deliberate
  correction. See `lib/dctl/commands/` and `docs/ARCHITECTURE.md`.
- The exit-code taxonomy originates in [`02-command-surface.md`](02-command-surface.md) §5 and is
  bound as an invariant set in [`11-invariants-and-guarantees.md`](11-invariants-and-guarantees.md).
