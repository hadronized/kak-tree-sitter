use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about = "A client/server interface between Kakoune and tree-sitter.")]
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
}
