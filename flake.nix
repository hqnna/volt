{
  description = "Volt — TUI Settings Editor for Amp";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      perSystem = {
        system,
        pkgs,
        lib,
        ...
      }: let
        fenixPkgs = inputs.fenix.packages.${system};
        toolchain = fenixPkgs.stable.withComponents [
          "cargo"
          "clippy"
          "rustc"
          "rustfmt"
          "rust-src"
        ];
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain toolchain;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        volt = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
          });

        # Static binary support (Linux only, using musl)
        muslTarget =
          if system == "x86_64-linux"
          then "x86_64-unknown-linux-musl"
          else if system == "aarch64-linux"
          then "aarch64-unknown-linux-musl"
          else null;

        staticToolchain =
          if muslTarget != null
          then
            fenixPkgs.combine [
              (fenixPkgs.stable.withComponents [
                "cargo"
                "rustc"
                "rust-src"
              ])
              fenixPkgs.targets.${muslTarget}.stable.rust-std
            ]
          else null;

        staticCraneLib =
          if staticToolchain != null
          then (inputs.crane.mkLib pkgs).overrideToolchain staticToolchain
          else null;

        staticCommonArgs =
          if muslTarget != null
          then {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;
            CARGO_BUILD_TARGET = muslTarget;
            CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
            HOST_CC = "${pkgs.stdenv.cc.nativePrefix}cc";
            TARGET_CC = "${pkgs.pkgsStatic.stdenv.cc}/bin/${pkgs.pkgsStatic.stdenv.cc.targetPrefix}cc";
            "CARGO_TARGET_${lib.toUpper (builtins.replaceStrings ["-"] ["_"] muslTarget)}_LINKER" =
              "${pkgs.pkgsStatic.stdenv.cc}/bin/${pkgs.pkgsStatic.stdenv.cc.targetPrefix}cc";
          }
          else {};

        staticCargoArtifacts =
          if staticCraneLib != null
          then staticCraneLib.buildDepsOnly staticCommonArgs
          else null;

        volt-static =
          if staticCraneLib != null
          then
            staticCraneLib.buildPackage (staticCommonArgs
              // {
                cargoArtifacts = staticCargoArtifacts;
              })
          else null;
      in {
        packages =
          {
            default = volt;
            volt = volt;
          }
          // lib.optionalAttrs (volt-static != null) {
            volt-static = volt-static;
          };

        checks = {
          inherit volt;

          volt-clippy = craneLib.cargoClippy (commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "-- -D warnings";
            });

          volt-fmt = craneLib.cargoFmt {
            src = commonArgs.src;
          };
        };

        devShells.default = craneLib.devShell {
          checks = {
            inherit volt;
          };

          packages = with pkgs; [
            rust-analyzer
          ];
        };
      };
    };
}
