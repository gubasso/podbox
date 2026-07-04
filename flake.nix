{
  description = "podbox rust dev shell (toolchain from rust-toolchain.toml)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        # Reads channel + components + targets straight from rust-toolchain.toml.
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            toolchain
            # ADR-0013 testing/quality stack (lib deps land via `cargo add`):
            pkgs.cargo-nextest # test runner
            pkgs.cargo-insta   # snapshot review (insta)
            pkgs.cargo-mutants # mutation testing
            pkgs.cargo-bloat   # binary size analysis
            pkgs.cargo-machete # unused-dependency lint
            pkgs.cargo-deny    # license/advisory policy
            pkgs.cargo-audit   # advisory audit
            pkgs.just          # task runner
            pkgs.pre-commit    # git hooks
          ];
          # No -sys/native deps yet (Cargo.toml has no dependencies). Add when a
          # crate links a system library, e.g.:
          # buildInputs = [ pkgs.openssl ];
          # nativeBuildInputs = [ pkgs.pkg-config ];
          shellHook = ''echo "podbox dev shell ready (toolchain from rust-toolchain.toml)"'';
        };
      }
    );
}
