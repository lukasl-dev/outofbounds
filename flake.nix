{
  description = "outofbounds";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    systems.url = "github:nix-systems/default";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-trusted-public-keys = "outofbounds.cachix.org-1:ToW43AWvNwShZOnh9NlgBP3BT68+5ytLyi3eZaXXi1I=";
    extra-substituters = "https://outofbounds.cachix.org";
  };

  outputs =
    {
      self,
      nixpkgs,
      systems,
      gitignore,
      crane,
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
          craneLib = crane.mkLib pkgs;
          commonArgs = {
            src = craneLib.cleanCargoSource (gitignoreSource ./.);
            pname = "outofbounds";
            version = "0.1.0";
            cargoLock = ./Cargo.lock;
            outputHashes = {
              "matrix-sdk-0.16.0" = "sha256-tI1CT9tWOAC2w24DRaC8Kw7ZvHkfE7IBeYzDpbpu9ZI=";
              "ruma-0.14.0" = "sha256-XVdEJUhrr4ehY+y3rfM637wfHwJJJchbAlHTH37NTYc=";
            };
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [
              pkgs.openssl
              pkgs.sqlite
            ];
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        in
        rec {
          outofbounds = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
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
              pkg-config
              openssl
              sqlite
            ];
          };
        }
      );

      nixosModules.default =
        { pkgs, ... }:
        {
          imports = [ ./nixos-module.nix ];
          services.outofbounds.package = nixpkgs.lib.mkDefault self.packages.${pkgs.system}.default;
        };
    };
}
