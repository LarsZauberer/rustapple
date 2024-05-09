{
  description = "rustapple nix flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    packages.${system}.default = pkgs.rustPlatform.buildRustPackage rec {
      pname = "rustapple";
      version = "0.1";
      cargoLock.lockFile = ./Cargo.lock;
      src = pkgs.lib.cleanSource ./.;
      nativeBuildInputs = [
        pkgs.pkg-config
        pkgs.alsa-lib
      ];
    };
  };
}
