{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ (import inputs.rust-overlay) ];
        };
        inherit (pkgs) lib;

        # Target musl when building on 64-bit linux
        buildTarget =
          {
            "x86_64-linux" = "x86_64-unknown-linux-musl";
          }
          .${system} or (pkgs.stdenv.buildPlatform.rust.rustcTargetSpec);

        # Set-up build dependencies and configure rust
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            targets = [ buildTarget ];
          }
        );
        cargo-details = lib.importTOML ./Cargo.toml;
        binary-name = cargo-details.package.name;
        commonArgs = {
          nativeBuildInputs = with pkgs; [ pkg-config ];
          CARGO_BUILD_TARGET = buildTarget;
        }
        // lib.optionalAttrs (buildTarget == "x86_64-unknown-linux-musl") {
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        };

        cargo-package = craneLib.buildPackage (
          commonArgs
          // {
            strictDeps = true;
            src = lib.cleanSourceWith {
              src = ./.;
              name = "source";
              filter =
                path: type:
                (lib.match ".*/test_data/.+\.(json|ya?ml)" path != null) || (craneLib.filterCargoSources path type);
            };
            pname = binary-name;
            cargo-deps = craneLib.buildDepsOnly (
              commonArgs
              // {
                src = lib.cleanSourceWith {
                  src = ./.;
                  name = "dependencies";
                  filter = path: _type: lib.match ".*/Cargo\.(toml|lock)$" path != null;
                };
              }
            );
          }
        );
        dockerTag =
          if lib.hasAttr "rev" inputs.self then
            "${lib.toString inputs.self.revCount}-${inputs.self.shortRev}"
          else
            "gitDirty";
      in
      {
        checks = { inherit cargo-package; };
        devShells.default = pkgs.mkShell {
          inputsFrom = [ cargo-package ];
          packages =
            (with pkgs; [
              # rust specific
              cargo-audit
              cargo-auditable
              cargo-cross
              cargo-deny
              cargo-outdated

              # Editor stuffs
              lldb
              rust-analyzer

              # Other tooling
              earthly
            ])
            ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ]
            ++ lib.optionals pkgs.stdenv.isLinux (with pkgs; [ cargo-watch ]); # Currently broken on macOS

          shellHook = ''
            cargo --version
          '';
        };
        packages = rec {
          default = rust;
          rust = cargo-package;
          docker = pkgs.dockerTools.buildImage {
            name = "europe-north1-docker.pkg.dev/nais-io/nais/images/${binary-name}";
            tag = "v${cargo-details.package.version}_${dockerTag}";
            extraCommands = "mkdir -p data";
            config = {
              Cmd = "--help";
              Entrypoint = [ "${cargo-package}/bin/${binary-name}" ];
            };
          };
        };

        formatter = inputs.treefmt-nix.lib.mkWrapper pkgs {
          programs.nixfmt.enable = true;
          programs.rustfmt.enable = true;
        };
      }
    );
}
