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
          inherit nativeBuildInputs buildInputs cargoVendorDir;
          strictDepends = true;
        };

        # Keep the vendored sources as a separately addressable store path. This
        # lets the cache closure below protect downloads from garbage collection.
        cargoVendorDir = craneLib.vendorCargoDeps {
          cargoLock = ./Cargo.lock;
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

        # `nix run` only creates a temporary GC root. This link farm can be
        # rooted explicitly to retain build-only inputs between runs and GCs:
        #
        #   nix build .#cache --out-link .nix-cache
        #
        # Keep this independent of `lilium` so ordinary source changes do not
        # invalidate the cache; Cargo.lock/toolchain changes still do.
        nixCache = pkgs.linkFarm "lilium-nix-cache" [
          { name = "cargo-artifacts"; path = cargoArtifacts; }
          { name = "cargo-sources"; path = cargoVendorDir; }
          { name = "rust-toolchain"; path = rustToolchain; }
          { name = "pkg-config"; path = pkgs.pkg-config; }
          { name = "openssl"; path = pkgs.openssl; }
          { name = "git"; path = pkgs.git; }
          { name = "cargo-watch"; path = pkgs.cargo-watch; }
          { name = "cargo-nextest"; path = pkgs.cargo-nextest; }
          { name = "postgresql"; path = pkgs.postgresql; }
        ];
      in
      {
        packages.default = lilium;
        packages.lilium = lilium;
        packages.cache = nixCache;

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
