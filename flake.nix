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
        toolchain = fenix.packages.${system}.default.toolchain;
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
              nixfmt
              typst
              mermaid-cli
              fira-mono
              cargo-release
              stdenv.cc.cc.lib
              bun
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
