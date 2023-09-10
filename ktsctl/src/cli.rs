use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
  author = "Dimitri Sabadie <dimitri.sabadie@gmail.com>",
  name = "ktsctl",
  version = concat!(env!("CARGO_PKG_VERSION"), "-", env!("GIT_HEAD")),
  about = "CLI controler of kak-tree-sitter"
)]
pub struct Cli {
  /// Fetch resources.
  #[clap(short, long)]
  pub fetch: bool,

  /// Whether we should compile fetched grammars.
  #[clap(short, long)]
  pub compile: bool,

  /// Whether we should install compiled grammars/queries to the kak-tree-sitter data location.
  ///
  /// Implies --compile for grammars.
  #[clap(short, long)]
  pub install: bool,

  /// Check whether a given language is currently installed and supported.
  #[clap(long)]
  pub has: bool,

  /// Grammar to fetch.
  ///
  /// Grammars are currently fetched from https://github.com/tree-sitter/.
  pub lang: String,
}
