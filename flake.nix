{
  description = "Midas development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};

        # Function to determine platform-specific binary URL
        getBinaryUrl = system:
          if system == "x86_64-linux"
          then {
            url = "https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux";
            sha256 = "sha256-XMXXusoqCJ1zow2C7UEhBtV2AzeV/RoZfs+hahNwglw=";
          }
          else if system == "aarch64-darwin"
          then {
            url = "https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-aarch64-macos";
            sha256 = ""; # Replace with actual hash after first build
          }
          else throw "Unsupported system: ${system}";

        # Get binary URL for current system
        binaryInfo = getBinaryUrl system;

        # Derivation to download the pre-built lightpanda executable
        lightpanda = pkgs.stdenv.mkDerivation {
          pname = "lightpanda";
          version = "nightly";

          # Download the pre-built binary from GitHub releases
          src = pkgs.fetchurl {
            url = binaryInfo.url;
            sha256 = binaryInfo.sha256;
          };

          # We need to extract the executable to a file
          dontUnpack = true;
          
          # Tools needed to patch the binary for NixOS
          nativeBuildInputs = with pkgs; [ 
            autoPatchelfHook 
            makeWrapper
          ];

          # Dependencies needed by the binary at runtime
          buildInputs = with pkgs; [
            glib
            stdenv.cc.cc.lib # for libstdc++
          ];

          # Copy, patch the downloaded binary, and wrap it with environment variables
          installPhase = ''
            mkdir -p $out/bin
            cp $src $out/bin/lightpanda
            chmod +x $out/bin/lightpanda
            
            # Wrap the binary to set the environment variable to disable telemetry
            wrapProgram $out/bin/lightpanda \
              --set LIGHTPANDA_DISABLE_TELEMETRY true
          '';

          meta = with pkgs.lib; {
            description = "Lightpanda Browser - Fast headless browser for web automation";
            homepage = "https://lightpanda.io";
            license = licenses.mit;
            platforms = platforms.linux ++ platforms.darwin;
            mainProgram = "lightpanda";
          };
        };
      in {
        # Output package containing the built lightpanda executable
        packages = {
          lightpanda = lightpanda;
          default = lightpanda;
        };

        # Output app to run lightpanda easily
        apps = {
          lightpanda = flake-utils.lib.mkApp {
            drv = lightpanda;
          };
          default = self.apps.${system}.lightpanda;
        };

        # Development shell for the "Midas" project, which *uses* lightpanda
        devShells.default = pkgs.mkShell {
          # Include only tools needed by the Midas project itself,
          # plus the lightpanda package built by this flake.
          buildInputs = with pkgs; [
            # Tools potentially needed by the Midas project:
            python3
            git
            go
            curl
            # Add any other tools your Midas project requires (e.g., nodejs?)

            # Make the 'lightpanda' executable available in the shell PATH.
            # Nix automatically includes its runtime dependencies.
            self.packages.${system}.lightpanda
          ];

          # Disable telemetry for Lightpanda
          LIGHTPANDA_DISABLE_TELEMETRY = "true";

          shellHook = ''
            echo "Midas development environment ready."
            echo "The 'lightpanda' executable is available in your PATH."
            echo "Telemetry has been disabled for Lightpanda."
            # Add any other setup needed specifically for Midas development below
          '';
        };
      }
    );
}
