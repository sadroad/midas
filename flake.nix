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
    flake-utils.lib.eachSystem ["x86_64-linux"] (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
        lib = pkgs.lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          toolchainPkgs:
            toolchainPkgs.rust-bin.fromRustupToolchainFile
            ./rust-toolchain.toml
        );

        unfilteredRoot = ./.;

        commonArgs = {
          src = lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources unfilteredRoot)
              (lib.fileset.fileFilter (file: lib.any file.hasExt ["js" "css"]) unfilteredRoot)
              (lib.fileset.maybeMissing ./assets)
            ];
          };
          strictDeps = true;

          buildInputs = with pkgs; [
          ];

          nativeBuildInputs = with pkgs; [
            tailwindcss_4
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs
          // {
            pname = "midas-deps";
          });

        midasPackage = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
            CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
            preBuild = ''
              mkdir -p assets
            '';
            postInstall = ''
              if [ -d "./assets" ]; then
                mkdir -p $out/assets
                cp -rT ./assets $out/assets
              else
                echo "Warning: ./assets doesn't exist"
              fi
            '';
          });

        midasClippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

        midasFmt = craneLib.cargoFmt {
          inherit (commonArgs) src;
        };

        dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = "midas";
          tag = "latest";
          config = {
            Cmd = ["${midasPackage}/bin/midas"];
            ExposedPorts = {
              "3000/tcp" = {};
            };
          };
          contents = [
            midasPackage
          ];
        };
      in {
        packages = {
          default = midasPackage;
          docker = dockerImage;
        };

        checks = {
          inherit midasPackage midasFmt midasClippy;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            cargo-watch

            #tailwindcss
            tailwindcss_4
            watchman
          ];
        };

        formatter = pkgs.alejandra;
      }
    );
}
