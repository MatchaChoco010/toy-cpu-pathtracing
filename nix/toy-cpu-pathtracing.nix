{ makeRustPlatform, fenix }:
let
  toolchain = fenix.stable.toolchain;
  rustPlatform = makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in rustPlatform.buildRustPackage {
  pname = "toy-cpu-pathtracing";
  version = "0.1.0";

  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;
}
