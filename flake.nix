{
  description = "Rust project with latest rustc via fenix";

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

        rustToolchain = pkgs.fenix.stable.toolchain;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [ rustToolchain ];

          shellHook = ''
            export RUST_BACKTRACE=1
          '';
        };
      });
}
