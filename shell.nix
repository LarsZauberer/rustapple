let
  pkgs = import <nixpkgs> {};
in
  pkgs.mkShell {
    packages = [
      pkgs.pkg-config
      pkgs.alsa-lib
    ];
  }
