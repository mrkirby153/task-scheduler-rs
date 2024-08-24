{
  description = "A task sscheduler in Rust";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in rec {
      packages = rec {
        task-scheduler = pkgs.callPackage ./task-scheduler.nix {};
        default = task-scheduler;
      };
      devShell = pkgs.mkShell {
        buildInpts = with pkgs; [];
      };
      formatter = pkgs.alejandra;
    });
}
