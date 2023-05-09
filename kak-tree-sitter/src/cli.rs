use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about = "A client/server interface between Kakoune and tree-sitter.")]
pub struct Cli {
  /// Whether we start from Kakoune and then we should inject the rc/.
  #[clap(short, long)]
  pub kakoune: bool,

  /// Try to daemonize, if not already done.
  #[clap(short, long)]
  pub daemonize: bool,

  /// Kakoune session to connect to.
  #[clap(short, long)]
  pub session: Option<String>,

  /// Kakoune client to connect with, if any.
  #[clap(short, long)]
  pub client: Option<String>,

  /// JSON-serialized request.
  #[clap(short, long)]
  pub request: Option<String>,
}
