{
  description = "A simple CPU path tracing implementation in Rust.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };
        craneLib =
          (crane.mkLib pkgs).overrideToolchain (p: p.fenix.stable.toolchain);
        crate = pkgs.callPackage ./nix/cargo.nix { inherit craneLib; };
        shell = pkgs.callPackage ./nix/shell.nix { inherit craneLib; };
      in {
        devShells.default = shell.devShell;
        packages.default = crate.build;
        checks = crate.checks;
      });
}
