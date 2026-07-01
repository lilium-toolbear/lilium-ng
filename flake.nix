{
  description = "Lilium NG — Rust rewrite of the DZMM/Lilium backend";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Edition 2024 requires a stable Rust toolchain >= 1.85.
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Filter out nix files and other non-source noise from the build input.
        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = path: type:
            # keep the default crane filter behavior, drop nix/dotfiles
            !(pkgs.lib.hasSuffix ".nix" path)
            && baseNameOf path != "flake.lock";
        };

        # Native deps shared by the build and the dev shell.
        nativeBuildInputs = with pkgs; [ pkg-config ];
        buildInputs = with pkgs; [ openssl ];

        commonArgs = {
          inherit src;
          pname = "lilium";
          version = "0.1.0";
          inherit nativeBuildInputs buildInputs;
          strictDepends = true;
        };

        # Build dependencies once and cache them, then build the actual crate.
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        lilium = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          # Only build the main binary, not every workspace member.
          cargoExtraArgs = "--bin lilium";
          # The lilium-common build.rs embeds a git hash when available; in the
          # sandbox .git is absent and the script degrades gracefully.
          doCheck = false;
        });
      in
      {
        packages.default = lilium;
        packages.lilium = lilium;

        # `nix develop` / `nix-shell` — a ready-to-hack environment.
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          packages = with pkgs; [
            rustToolchain
            git
            cargo-watch
            cargo-nextest
            postgresql
          ] ++ nativeBuildInputs;
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
      });
}