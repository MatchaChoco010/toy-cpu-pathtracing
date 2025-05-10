{ pkgs, craneLib }: {
  devShell = craneLib.devShell { buildInputs = with pkgs; [ mold clang ]; };
}
