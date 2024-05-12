{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage {
  pname = "rustapple";
  version = "0.1";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
  nativeBuildInputs = [
    pkgs.pkg-config
    pkgs.alsa-lib
    pkgs.ffmpeg
    pkgs.libclang.lib
    pkgs.clang
    pkgs.stdenv.cc.libc
  ];
  buildInputs = [
    pkgs.pkg-config
    pkgs.alsa-lib
    pkgs.ffmpeg
    pkgs.libclang.lib
    pkgs.clang
    pkgs.stdenv.cc.libc
  ];

  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
}
