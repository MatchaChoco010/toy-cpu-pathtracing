{ pkgs, craneLib }:
let
  commonArgs = {
    src = craneLib.cleanCargoSource ../.;
    strictDeps = true;
    buildInputs = with pkgs; [ ];
    nativeBuildInputs = with pkgs; [ mold clang ];
    pname = "toy-cpu-pathtracing";
    preBuild = "ulimit -s unlimited";
  };
  myCrateClippy = craneLib.cargoClippy (commonArgs // {
    cargoArtifacts = craneLib.buildDepsOnly (commonArgs);
    cargoClippyExtraArgs = "--all-targets";
  });
  myCrate = craneLib.buildPackage (commonArgs);
in {
  build = myCrate;
  checks = { inherit myCrate myCrateClippy; };
}
