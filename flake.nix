{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

  };

  outputs =
    inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = inputs.nixpkgs.legacyPackages.${system};
        toolchain = inputs.fenix.packages.${pkgs.pkgsBuildHost.system}.fromToolchainFile {
          file = ./.rust-toolchain.toml;
          sha256 = "sha256-xgApk6OFxgHuss/VOC9hWwDhDuVF7QoSuWFiozdEWTk=";
        };
        buildInputs = with pkgs; [
          toolchain
          nasm
          # We need ld
          binutils
          # Grub-mkrescue (to be replaced with a script maybe?)
          grub2
          libisoburn
          # Testing
          qemu
        ];
      in
      {

        devShells.default = pkgs.mkShell {
          buildInputs = buildInputs;
        };
      }
    );

}
