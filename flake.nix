{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        inherit (pkgs)
          lib
          mkShell
          pkg-config
          rust-bin
          stdenv
          ;

        rustToolchain =
          target:
          rust-bin.nightly."2026-09-21".default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "clippy"
              "rustc-dev"
            ];
            targets = [ target ];
          };

        linuxBuildInputs = [
          (rustToolchain "x86_64-unknown-linux-gnu")
        ];

        darwinBuildInputs = [
          (rustToolchain "aarch64-apple-darwin")
        ];
      in
      {
        devShells.default = mkShell {
          buildInputs = [
            pkg-config
          ]
          ++ lib.optionals stdenv.hostPlatform.isDarwin darwinBuildInputs
          ++ lib.optionals stdenv.hostPlatform.isLinux linuxBuildInputs;
        };
      }
    );
}
