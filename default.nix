# Legacy `nix-build` entry point.
# Delegates to the flake's default package via flake-compat so there is a
# single source of truth (flake.nix). Produces a `result/` symlink.
(import
  (fetchTarball "https://github.com/NixOS/flake-compat/archive/refs/tags/v1.1.0.tar.gz")
  { src = ./.; }
).defaultNix