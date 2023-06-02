mod cli;
mod daemon;
mod error;
mod handler;
mod highlighting;
mod languages;
mod queries;
mod rc;
mod request;
mod response;
mod session;

use clap::Parser;
use cli::Cli;
use colored::Colorize;
use daemon::Daemon;
use error::OhNo;
use kak_tree_sitter_config::Config;
use request::Request;
use session::KakSession;

fn main() {
  if let Err(err) = start() {
    eprintln!("{}", err.to_string().red());
    std::process::exit(1);
  }
}

fn start() -> Result<(), OhNo> {
  let cli = Cli::parse();
  let config = Config::load();

  if cli.kakoune {
    // inject the rc/ and daemon-based settings
    println!("{}", rc::rc_commands());
  }

  if let (Some(session), Some(request)) = (cli.session, cli.request) {
    // client logic
    let kak_sess = KakSession::new(session, cli.client);

    // parse the request payload and embed it in a request
    let payload = serde_json::from_str(&request).map_err(|err| OhNo::InvalidRequest {
      err: err.to_string(),
    })?;
    let req = Request::new(kak_sess, payload);
    Daemon::send_request(req)
  } else {
    // server logic
    Daemon::bootstrap(config, cli.daemonize)
  }
}
