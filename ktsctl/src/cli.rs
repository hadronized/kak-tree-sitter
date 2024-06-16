use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(
  author = "Dimitri Sabadie <hadronized@strongly-typed-thoughts.net>",
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
  /// Fetch resources.
  Fetch {
    /// Execute commands for all known languages.
    ///
    /// The list of languages can be seen with `ktsctl query -a`.
    #[clap(short, long)]
    all: bool,

    /// Language to manage.
    lang: Option<String>,
  },

  /// Compile resources.
  Compile {
    /// Execute commands for all known languages.
    ///
    /// The list of languages can be seen with `ktsctl info --all`.
    #[clap(short, long)]
    all: bool,

    /// Language to manage.
    lang: Option<String>,
  },

  /// Install resources.
  Install {
    /// Execute commands for all known languages.
    ///
    /// The list of languages can be seen with `ktsctl query -a`.
    #[clap(short, long)]
    all: bool,

    /// Language to manage.
    lang: Option<String>,
  },

  /// Synchronize resources (implies fetch, compile and install).
  ///
  /// This command also checks whether pinned version are already there; if so,
  /// nothing is performed.
  Sync {
    /// Execute commands for all known languages.
    ///
    /// The list of languages can be seen with `ktsctl query -a`.
    #[clap(short, long)]
    all: bool,

    /// Language to manage.
    lang: Option<String>,
  },

  /// Get information on installed resources.
  Query {
    /// List all known languages and display information about them.
    #[clap(short, long)]
    all: bool,

    /// Get information about a specific language.
    lang: Option<String>,
  },

  /// Remove resources.
  ///
  /// If no flag is passed, -g and -q are assumed. Passing -p will also prune
  /// out-of-sync pins; so to completely remove everything for a given language:
  ///
  ///     ktsctl rm -p <LANG>
  #[clap(aliases = &["rm"])]
  Remove {
    /// Remove grammar.
    #[clap(short, long)]
    grammar: bool,

    /// Remove queries.
    #[clap(short, long)]
    queries: bool,

    /// Prune resources.
    ///
    /// Pruning resources implies resources deletion for which the pin is out
    /// of date.
    #[clap(short, long)]
    prune: bool,

    /// Remove resources for the specific language.
    lang: String,
  },
}
