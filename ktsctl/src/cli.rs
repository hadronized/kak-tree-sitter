use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(
  author = "Dimitri Sabadie <dimitri.sabadie@gmail.com>",
  name = "ktsctl",
  version = concat!(env!("CARGO_PKG_VERSION"), "-", env!("GIT_HEAD")),
  about = "CLI controler for kak-tree-sitter"
)]
pub struct Cli {
  #[clap(long)]
  pub verbose: bool,

  #[clap(subcommand)]
  pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
  /// Manage tree-sitter resources (grammers, queries).
  Manage {
    /// Fetch resources.
    #[clap(short, long)]
    fetch: bool,

    /// Whether we should compile fetched grammars.
    #[clap(short, long)]
    compile: bool,

    /// Whether we should install compiled grammars/queries to the Kakoune tree-sitter data location.
    ///
    /// Implies --compile for grammars.
    #[clap(short, long)]
    install: bool,

    /// Synchronize resources.
    ///
    /// This command uses the `pin` configuration option of the language to synchronize. This command doesn’t remove
    /// previous pinned resources; see --clear.
    #[clap(short, long)]
    sync: bool,

    /// Language to manage.
    lang: String,
  },

  /// Get information on installed resources.
  Info {
    /// Get information about a specific language.
    #[clap(long)]
    lang: Option<String>,

    /// List all known languages and display information about them.
    #[clap(short, long)]
    all: bool,
  },
}
