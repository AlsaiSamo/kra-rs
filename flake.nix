{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
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
        overlays = [(import rust-overlay)];
      };
      #toolchain = pkgs.rust-bin.selectLatestNightlyWith (t: t.default);
      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;
      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

      mkCrate = name: craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./${name});
        cargoLock = ./Cargo.lock;
        buildInputs = []
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
      };

      kra = mkCrate "kra";
    in {
      checks = {
        inherit kra;
      };

      packages = {
        kra = kra;
        default = self.packages.kra;
      };

      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.checks.${system};

        RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";

        nativeBuildInputs = with pkgs; [
          toolchain
          clippy
          rustfmt
          rust-analyzer
          sccache
          cargo-expand
        ];
      };
    });
}
