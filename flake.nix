{
  description = "outofbounds";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    systems.url = "github:nix-systems/default";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      systems,
      gitignore,
    }:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
      gitignoreSource = gitignore.lib.gitignoreSource;
    in
    {
      packages = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        rec {
          outofbounds = pkgs.rustPlatform.buildRustPackage {
            pname = "outofbounds";
            version = "0.1.0";
            src = gitignoreSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };
          default = outofbounds;
        }
      );

      devShells = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustc
              cargo
              rust-analyzer
              clippy
              rustfmt
            ];
          };
        }
      );
    };
}
