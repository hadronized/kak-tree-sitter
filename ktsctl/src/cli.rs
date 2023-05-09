use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
  /// Grammar to fetch.
  ///
  /// Grammars are currently fetched from https://github.com/tree-sitter/.
  #[clap(short, long)]
  pub grammar: Option<String>,

  /// Whether we should fetch queries.
  ///
  /// Queries are currently fetched from https://github.com/helix-editor/helix/.
  #[clap(short, long)]
  pub queries: bool,

  /// Whether we should compile fetched grammars.
  #[clap(short, long)]
  pub compile: bool,

  /// Whether we should install compiled grammars/queries to the kak-tree-sitter data location.
  ///
  /// Implies --compile for grammars.
  #[clap(short, long)]
  pub install: bool,
}
