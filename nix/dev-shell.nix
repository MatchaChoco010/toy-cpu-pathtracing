{ mkShell, fenix }:
let toolchain = fenix.stable.toolchain;
in mkShell {
  buildInputs = [ toolchain ];
  shellHook = ''
    export RUST_BACKTRACE=1
  '';
}
