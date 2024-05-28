{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./Cargo.lock).package;
in
  pkgs.rustPlatform.buildRustPackage {
    pname = manifest.name;
    version = manifest.version;
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
