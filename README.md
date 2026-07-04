# podbox

A workspace / dev-container tool that wraps a container runtime, giving each
project a reproducible, isolated development environment.

> Status: early development. Nothing is implemented yet beyond a stub binary —
> the product contract and its Rust implementation binding are specified first.

## Documentation

Full documentation lives under [`docs/`](docs/README.md), organised as a
Diataxis tree:

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
and open an issue to discuss substantial changes before submitting a pull
request. Ensure `just lint` and `just test` pass.
