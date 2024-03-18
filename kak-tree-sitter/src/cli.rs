use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
  author = "Dimitri Sabadie <dimitri.sabadie@gmail.com>",
  name = "kak-tree-sitter",
  version = concat!(env!("CARGO_PKG_VERSION"), "-", env!("GIT_HEAD")),
  about = "A client/server interface between Kakoune and tree-sitter."
)]
pub struct Cli {
  /// Whether we start from Kakoune and then we should issue an initial request for setup.
  #[clap(short, long)]
  pub kakoune: bool,

  /// Start the server, if not already started.
  #[clap(short, long)]
  pub server: bool,

  /// Try to daemonize, if not already done.
  #[clap(short, long)]
  pub daemonize: bool,

  /// Kakoune session to connect to.
  #[clap(long)]
  pub session: Option<String>,

  /// Kakoune client to connect with, if any.
  #[clap(short, long)]
  pub client: Option<String>,

  /// JSON-serialized request.
  #[clap(short, long)]
  pub request: Option<String>,

  /// Verbosity.
  ///
  /// Can be accumulated to get more verbosity. Without this flag, logging is disabled. Then, for each applicaton of the
  /// flag, the obtained verbosity follows this order: error, warn, info, debug, trace. Thus, if you use -v, you will
  /// only get error messages. If you use -vv, you will also see warnings. The maximum verbosity is achieved with -vvvvv
  /// for trace logs.
  #[arg(short, long, action = clap::ArgAction::Count)]
  pub verbose: u8,

  /// Insert Kakoune code related to highlighting.
  ///
  /// Highlighting is supported mainly via hooks and commands that ping-ping between Kakoune and KTS.
  #[arg(long)]
  pub with_highlighting: bool,

  /// Insert Kakoune commands, user modes and mappings related to text-objects.
  ///
  /// Those are default and completely optional. It is advised to start with those and if further customization is
  /// needed, you shall not use this flag and craft your own user modes and mappings.
  ///
  /// Text-objects user-modes will be available via the 'tree-sitter' user-mode.
  #[arg(long)]
  pub with_text_objects: bool,
}
