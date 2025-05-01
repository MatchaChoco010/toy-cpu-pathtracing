{ pkgs, craneLib }:
let
  commonArgs = {
    src = craneLib.cleanCargoSource ../.;
    strictDeps = true;
    buildInputs = with pkgs;
      [
        # openssl
      ];
    nativeBuildInputs = with pkgs;
      [
        # pkg-config
      ];
    pname = "toy-cpu-pathtracing";
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
