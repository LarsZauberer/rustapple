let
  pkgs = import <nixpkgs> {};
in
  pkgs.mkShell {
    name = "rustapple";
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
