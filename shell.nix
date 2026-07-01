# Legacy `nix-shell` entry point.
# Delegates to the flake's devShell via flake-compat so there is a single
# source of truth (flake.nix).
(import
  (fetchTarball "https://github.com/NixOS/flake-compat/archive/refs/tags/v1.1.0.tar.gz")
  { src = ./.; }
).shellNix