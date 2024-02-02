{
  description = "Build a cargo project without extra checks";
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
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      rustNightly = pkgs.rust-bin.selectLatestNightlyWith (t: t.default);
      craneLib = (crane.mkLib pkgs).overrideToolchain rustNightly;
      my-crate = craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        buildInputs = []
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
      };
    in {
      checks = {
        inherit my-crate;
      };

      packages.default = my-crate;

      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.checks.${system};

        RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";

        nativeBuildInputs = with pkgs; [
          rustNightly
          clippy
          rustfmt
          rust-analyzer
          sccache
        ];
      };
    });
}
