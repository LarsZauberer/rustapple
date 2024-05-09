{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage {
  pname = "rustapple";
  version = "0.1";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
  nativeBuildInputs = [
    pkgs.pkg-config
    pkgs.alsa-lib
  ];
  buildInputs = [
    pkgs.pkg-config
    pkgs.alsa-lib
  ];
}
