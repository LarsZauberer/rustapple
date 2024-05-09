{
  description = "rustapple nix flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
      in {
        packages = {
          default = pkgs.rustPlatform.buildRustPackage {
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
          };
        };

        # Expose executable
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/rustapple";
        };

        defaultPackage = self.packages.${system}.default;
      }
    );
}
