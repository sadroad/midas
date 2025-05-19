{
  description = "Midas development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          toolchainPkgs:
            toolchainPkgs.rust-bin.fromRustupToolchainFile
            ./rust-toolchain.toml
        );

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          buildInputs = with pkgs; [
          ];

          nativeBuildInputs = with pkgs; [
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs
          // {
            pname = "midas-deps";
          });

        midasPackage = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            pname = "midas";
          });

        midasClippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

        midasFmt = craneLib.cargoFmt {
          inherit (commonArgs) src;
        };
      in {
        packages.default = midasPackage;

        checks = {
          inherit midasPackage midasFmt midasClippy;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            bacon
          ];
        };

        formatter = pkgs.alejandra;
      }
    );
}
