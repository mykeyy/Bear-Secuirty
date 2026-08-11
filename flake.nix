{
  description = "Development shell for Bear Sec Bot";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        python = pkgs.python312.withPackages (pythonPackages:
          with pythonPackages; [
            discordpy
            python-dotenv
          ]);
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            python
            pkgs.ruff
            pkgs.basedpyright
          ];
        };

        formatter = pkgs.nixfmt-tree;
      });
}
