let
  pkgs = import <nixpkgs> {};
in
  pkgs.mkShell {
    name = "rustapple";
    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.alsa-lib
    ];
    buildInputs = [
      pkgs.pkg-config
      pkgs.alsa-lib
    ];
  }
