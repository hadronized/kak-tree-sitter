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
use request::UnidentifiedRequest;
use server::Server;

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

  // server logic; basically a no-op if the server is already started, and should quickly return if cli.daemonize is
  // set
  Server::bootstrap(&config, cli.daemonize)?;

  if cli.kakoune {
    // when starting from Kakoune, we manually issue a first request to setup the Kakoune session
    if let Some(name) = cli.session {
      let req = UnidentifiedRequest::NewSession { name };
      Server::send_request(req)?;
    } else {
      return Err(OhNo::InvalidRequest {
        err: "missing session name; start with --session -s <session-name>".to_owned(),
      });
    }
  } else if let Some(request) = cli.request {
    // otherwise, regular client
    let req = serde_json::from_str::<UnidentifiedRequest>(&request).map_err(|err| {
      OhNo::InvalidRequest {
        err: err.to_string(),
      }
    })?;
    let req = if let Some(session) = cli.session {
      req.with_session(session)
    } else {
      req
    };

    Server::send_request(req)?;
  }

  Ok(())
}
