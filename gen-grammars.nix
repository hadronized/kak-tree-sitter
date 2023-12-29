# Fetch and build grammars from https://github.com/helix-editor/helix/languages.toml.
# The code is borrowed and reworked from: https://github.com/helix-editor/helix/blob/85fce2f5b6c9f35ab9d3361f3933288a28db83d4/grammars.nix.
{helix}: {
  stdenv,
  lib,
  fetchFromGitHub,
  ...
}: let
  buildGrammar = grammar:
    stdenv.mkDerivation {
      # See: https://github.com/NixOS/nixpkgs/blob/fbdd1a7c0bc29af5325e0d7dd70e804a972eb465/pkgs/development/tools/parsing/tree-sitter/grammar.nix.
      pname = "tree-sitter-${grammar.name}";
      version = grammar.source.rev;

      src = let
        isGitHubGrammar = grammar: lib.hasPrefix "https://github.com" grammar.source.git;
        sourceGitHub = let
          match = builtins.match "https://github\.com/([^/]*)/([^/]*)/?" grammar.source.git;
          owner = builtins.elemAt match 0;
          repo = builtins.elemAt match 1;
        in
          builtins.fetchTree {
            inherit (grammar.source) rev;
            inherit owner repo;
            type = "github";
          };
        sourceGit = builtins.fetchTree {
          type = "git";
          url = grammar.source.git;
          rev = grammar.source.rev;
          ref = grammar.source.ref or "HEAD";
          shallow = true;
        };
      in
        if isGitHubGrammar grammar
        then sourceGitHub
        else sourceGit;

      sourceRoot =
        if builtins.hasAttr "subpath" grammar.source
        then "source/${grammar.source.subpath}"
        else "source";

      dontConfigure = true;

      FLAGS = [
        "-Isrc"
        "-g"
        "-O3"
        "-fPIC"
        "-fno-exceptions"
        "-Wl,-z,relro,-z,now"
      ];

      NAME = grammar.name;

      buildPhase = ''
        runHook preBuild

        if [[ -e src/scanner.cc ]]; then
          $CXX -c src/scanner.cc -o scanner.o $FLAGS
        elif [[ -e src/scanner.c ]]; then
          $CC -c src/scanner.c -o scanner.o $FLAGS
        fi

        $CC -c src/parser.c -o parser.o $FLAGS
        $CXX -shared -o $NAME.so *.o

        ls -al

        runHook postBuild
      '';

      installPhase = ''
        runHook preInstall
        mkdir $out
        mv $NAME.so $out/
        runHook postInstall
      '';

      # Strip failed on darwin: strip: error: symbols referenced by indirect symbol table entries that can't be stripped
      fixupPhase = lib.optionalString stdenv.isLinux ''
        runHook preFixup
        $STRIP $out/$NAME.so
        runHook postFixup
      '';
    };
in
  stdenv.mkDerivation rec {
    pname = "consolidated-grammars";
    version = helix.version;

    src = helix.repo;

    dontUnpack = true;
    dontPatch = true;
    dontConfigure = true;

    buildPhase = let
      languagesConfig = builtins.fromTOML (builtins.readFile "${src}/languages.toml");
      isGitGrammar = grammar:
        builtins.hasAttr "source" grammar
        && builtins.hasAttr "git" grammar.source
        && builtins.hasAttr "rev" grammar.source;
      gitGrammars = builtins.filter isGitGrammar languagesConfig.grammar;
      builtGrammars =
        builtins.map (grammar: {
          inherit (grammar) name;
          value = buildGrammar grammar;
        })
        gitGrammars;
      grammarLinks =
        lib.mapAttrsToList
        (name: artifact: "ln -s ${artifact}/${name}.so $out/grammars/${name}.so")
        (lib.filterAttrs (n: v: lib.isDerivation v) (builtins.listToAttrs builtGrammars));
    in ''
      mkdir -p $out/grammars
      ${builtins.concatStringsSep "\n" grammarLinks}
      ln -s ${src}/runtime/queries $out/queries
    '';
  }
