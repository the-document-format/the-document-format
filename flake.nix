{
  description = "The Document Format";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    treefmt.url = "github:numtide/treefmt-nix";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      treefmt,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        nightly = fenix.packages.${system}.toolchainOf {
          channel = "nightly";
          date = "2026-01-30";
          sha256 = "sha256-sgb5WP79bNLEaZ4IygKSu3zv0LzP1G+7dVx9XdZeRoE=";
        };
        toolchain = fenix.packages.${system}.combine [
          nightly.toolchain
          (fenix.packages.${system}.targets.wasm32-unknown-unknown.toolchainOf {
            channel = "nightly";
            date = "2026-01-30";
            sha256 = "sha256-sgb5WP79bNLEaZ4IygKSu3zv0LzP1G+7dVx9XdZeRoE=";
          }).rust-std
        ];
        treefmtEval = treefmt.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs.nixfmt.enable = true;
          programs.yamlfmt.enable = true;
          programs.toml-sort.enable = true;
          programs.rustfmt.enable = true;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages =
            (with pkgs; [
              nil
              nixd
              cargo-release
              just
            ])
            ++ [
              toolchain
            ];
        };

        formatter = treefmtEval.config.build.wrapper;

        checks = {
          formatting = treefmtEval.config.build.check self;
        };
      }
    );
}
