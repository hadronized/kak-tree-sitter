mod cli;
mod error;
mod handler;
mod highlighting;
mod languages;
mod queries;
mod rc;
mod request;
mod response;
mod server;
mod session;

use clap::Parser;
use cli::Cli;
use colored::Colorize;
use error::OhNo;
use kak_tree_sitter_config::Config;
use request::Request;
use server::Server;
use session::KakSession;

fn main() {
  if let Err(err) = start() {
    eprintln!("{}", err.to_string().red());
    std::process::exit(1);
  }
}

fn start() -> Result<(), OhNo> {
  let cli = Cli::parse();
  let config = match Config::load_from_xdg() {
    Ok(config) => config,
    Err(err) => {
      eprintln!("configuration error; will be using empty configuration: {err}");
      Config::default()
    }
  };

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
    Server::send_request(req)
  } else {
    // server logic
    Server::bootstrap(&config, cli.daemonize)
  }
}
