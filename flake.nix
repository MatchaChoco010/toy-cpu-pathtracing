{
  description = "A simple CPU path tracing implementation in Rust.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };
      in {
        devShells.default = pkgs.callPackage ./nix/dev-shell.nix { };
        packages.default = pkgs.callPackage ./nix/toy-cpu-pathtracing.nix { };
      });
}
