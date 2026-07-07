# podbox

A workspace / dev-container tool that gives each project a reproducible, disposable
development sandbox. podbox drives a familiar **OCI / Podman interface**, but runs each
workspace inside a **KVM-class microVM** — a real hardware-virtualization boundary, not a
shared kernel. A primary goal is a **secure sandbox for coding AI agents** (and any
untrusted or self-modifying code) to execute without unmediated access to the host.

## Status

Early development, but no longer a stub. The **configuration, composition, image-freshness,
diagnostics, and lifecycle-planning** surface works today and is covered by tests: you can
scaffold a project, author and compose devcontainer layers, manage the egress allowlist,
build images, and inspect exactly what a reconcile *would* do.

What is **not** wired yet is the **live guest transport** — the in-VM agent a running
sandbox needs. So `workspace shell` / `exec` / `up` / `down` and the *execution* half of
`workspace reconcile` are deferred; the `--dry-run` planning paths of `reconcile` and `down`
work. See [`docs/reference/implementation-status.md`](docs/reference/implementation-status.md)
for the per-command breakdown.

## Requirements

Running a real sandbox targets **Linux with KVM**:

- Linux with `/dev/kvm` available (the microVM boundary is hardware virtualization).
- Rootless [Podman](https://podman.io/) with the **`krun`** (libkrun) runtime.

On other platforms podbox builds and its planning/config commands run, but starting a
sandbox reports `Unsupported`. Run `podbox doctor` to check your host — it reports each
`HOST-*` / `RT-*` prerequisite.

## Install

```bash
# From crates.io
cargo install podbox

# From source
git clone https://github.com/gubasso/podbox
cd podbox
cargo install --path .
```

## Quickstart

The commands below all work today and run hermetically against a scratch `PODBOX_HOME`:

```bash
# Scaffold the XDG config tree (config.toml, a minimal manifest + devcontainer layers)
podbox init

# Check host/runtime/config prerequisites (scope to just config here)
podbox doctor --scope config

# Merge the manifest's layers into one verified devcontainer.json (prints a content digest)
podbox manifest compose minimal

# Inspect the effective egress policy (default-deny) and the effective allowlist
podbox network show

# See workspace state, drift, image freshness, and what a reconcile would do
podbox status

# Shell completions (bash/zsh/fish)
podbox completion bash
```

Every command supports `--json` for machine-readable output and maps failures to a stable
`0`–`7` exit taxonomy. For a guided walkthrough, see the
[getting-started guide](docs/guides/getting-started.md).

## Documentation

Full documentation lives under [`docs/`](docs/README.md), organised as a Diataxis tree:

- **What works today**: [`docs/reference/implementation-status.md`](docs/reference/implementation-status.md)
- **Getting started** (tutorial): [`docs/guides/getting-started.md`](docs/guides/getting-started.md)
- **Product contract** (technology-neutral): [`docs/reference/spec/`](docs/reference/spec/README.md)
- **Rust implementation binding** (ADRs): [`docs/decisions/`](docs/decisions/)
- **Guides** (contributing, adding a subcommand): [`docs/guides/`](docs/guides/)
- **Architecture / rationale**: [`docs/explanation/`](docs/explanation/)

Start at the [documentation index](docs/README.md).

## Development shell

This project ships a Nix flake devShell with all tooling pinned.

```bash
# Interactive: allow direnv to load the shell on cd
direnv allow

# Ad hoc: enter the devShell directly
nix develop
```

## Tasks

Common tasks run through the project task runner:

```bash
just lint    # run linters and formatters
just test    # run the test suite
just build   # build the project
```

## License

Licensed under the [MIT License](LICENSE), Copyright (c) 2026 Gustavo Basso.

## Contributing

Contributions are welcome. See [`docs/guides/contributing-rust.md`](docs/guides/contributing-rust.md)
and open an issue to discuss substantial changes before submitting a pull request. Ensure
`just lint` and `just test` pass.
