# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/gubasso/podbox/compare/v0.1.0...v0.1.1) - 2026-07-07

### Added

- *(cli)* build out command surface with init, status, and completion
- *(workspace)* add lifecycle commands and network allowlist policy
- *(runtime)* add podman-krun microvm backend and guest channel
- *(image)* add image build command with runtime and state adapters
- build out cli application core over the crate stub

### Other

- *(crate)* split binary over a testable library seam
