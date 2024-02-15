{
  description = "tree-sitter meets Kakoune";

  inputs = {
    devshell.url = "github:numtide/devshell";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    devshell,
    fenix,
    flake-utils,
    nixpkgs,
    ...
  }:
  # Get Linux x86_64, Linux aarch64, macOS x86_64 and macOS aarch64 for free.
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          devshell.overlays.default
          fenix.overlays.default
        ];
      };

      # Read the kak-tree-sitter version from Cargo.toml.
      version = (builtins.fromTOML (builtins.readFile ./kak-tree-sitter/Cargo.toml)).package.version;

      # We use Helix sources to fetch and build grammars and queries.
      helix = {
        version = "23.10-dev";
        repo = pkgs.fetchFromGitHub {
          owner = "helix-editor";
          repo = "helix";
          rev = "85fce2f5b6c9f35ab9d3361f3933288a28db83d4";
          hash = "sha256-TNEsdsaCG1+PvGINrV/zw7emzwpfWiml4b77l2n5UEI=";
        };
      };

      # Description of how to build the kak-tree-sitter.
      derivation = {
        rustPlatform,
        git,
      }:
        rustPlatform.buildRustPackage {
          inherit version;
          pname = "kak-tree-sitter";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          # We only need to build kak-tree-sitter, no need to waste time on ktsctl.
          cargoBuildFlags = ["--package=kak-tree-sitter"];
          nativeBuildInputs = [git];
        };

      # Build or "realize" the derivation.
      package = pkgs.callPackage derivation {};

      # Fetch and build grammars and queries from Helix.
      grammars = pkgs.callPackage (import ./gen-grammars.nix {inherit helix;}) {};

      # Generate kak-tree-sitter config.toml using tree-sitter queries from Helix.
      config = pkgs.callPackage (import ./gen-config.nix {inherit helix;}) {};

      # Final package to assemble everything.
      final = pkgs.runCommandLocal "kak-tree-sitter-full" {} ''
        mkdir -p $out/share/kak-tree-sitter

        cp --dereference             ${config}/config.toml $out/share/kak-tree-sitter/config.toml
        cp --dereference --recursive ${grammars}/grammars  $out/share/kak-tree-sitter/grammars
        cp --dereference --recursive ${grammars}/queries   $out/share/kak-tree-sitter/queries
        cp --dereference --recursive ${package}/bin        $out/bin
      '';
    in
      # See: https://nixos.wiki/wiki/Flakes#Output_schema
      {
        formatter = pkgs.alejandra;

        packages.default = final;

        apps.default = {
          type = "app";
          program = "${final}/bin/kak-tree-sitter";
        };

        devShell = pkgs.devshell.mkShell {
          name = "kak-tree-sitter";
          motd = "Entered the kak-tree-sitter development environment";
          packages = [(pkgs.fenix.stable.withComponents ["cargo" "clippy" "rust-analyzer" "rustfmt" "rust-src"])];
        };
      });
}
