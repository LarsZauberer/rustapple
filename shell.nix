let
  pkgs = import <nixpkgs> {};
in
  pkgs.mkShell {
    name = "rustapple";
    nativeBuildInputs = [
      pkgs.pkg-config
    ];
    buildInputs = [
      pkgs.alsa-lib
    ];
  }
