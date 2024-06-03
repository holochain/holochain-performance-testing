{
  description = "Flake for Holochain testing";

  inputs = {
    versions.url = "github:holochain/holochain?dir=versions/0_3_rc";

    holochain = {
      url = "github:holochain/holochain";
      inputs.versions.follows = "versions";
    };

    tryorama.url = "github:holochain/tryorama/main";

    flake-utils.url = "github:numtide/flake-utils";

    nixpkgs.follows = "holochain/nixpkgs";
  };

  outputs = inputs:
    inputs.holochain.inputs.flake-parts.lib.mkFlake { inherit inputs; }
      {
        systems = builtins.attrNames inputs.holochain.devShells;
        perSystem = { lib, config, pkgs, system, self', ... }:
          {
            formatter = pkgs.nixpkgs-fmt;
            devShells.default = pkgs.mkShell {
              inputsFrom = [
                inputs.holochain.devShells.${system}.holonix
              ];

              packages = [
                inputs.tryorama.packages.${system}.trycp-server
                pkgs.shellcheck
                pkgs.statix
              ];
            };

            devShells.ci = pkgs.mkShell {
              inputsFrom = [
                inputs.holochain.devShells.${system}.holochainBinaries
              ];

              packages = [
                inputs.tryorama.packages.${system}.trycp-server
                pkgs.shellcheck
                pkgs.statix
              ];
            };
          };
      };
}
