{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        rustToolchain = fenix.packages.${system}.stable.toolchain;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource ./.;
        commonArgs = { inherit src; };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        fenixRustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        # Provides `cargo bench --bench <name>_gungraun -- --save-baseline=X`
        # for the deterministic instruction-count CI regression gate set up
        # per the perf-verification skill. Drop this (and the gungraun
        # dev-dependency in Cargo.toml) if the port doesn't need a
        # perf-regression gate yet.
        gungraunRunner = fenixRustPlatform.buildRustPackage rec {
          pname = "gungraun-runner";
          version = "0.19.3";
          src = pkgs.fetchCrate {
            inherit pname version;
            hash = "sha256-YubsNYBJuENEvaOGXh9yGg8DWqzA92lujsHthAwypRw=";
          };
          cargoHash = "sha256-KMA5v7ogN1fKlM6Jz4iXgYl5NDk0XdB/4KIbihKxs84=";
          doCheck = false;
        };
      in {
        packages.gungraun-runner = gungraunRunner;

        checks = {
          fmt = craneLib.cargoFmt { inherit src; };

          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-features -- -D warnings";
          });

          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--all-features";
          });
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = [ pkgs.valgrind gungraunRunner ];
        };
      });
}
