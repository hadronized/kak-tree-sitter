mod cli;
mod error;
mod handler;
mod highlighting;
mod languages;
mod logging;
mod queries;
mod rc;
mod request;
mod response;
mod server;
mod session;
mod tree_sitter_state;
mod kak;

use clap::Parser;
use cli::Cli;
use error::OhNo;
use kak_tree_sitter_config::Config;
use logging::Verbosity;
use request::UnixRequest;
use server::Server;

fn main() {
  if let Err(err) = start() {
    log::error!("{err}");
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

  // inject static.kak if we are starting from Kakoune
  if cli.kakoune {
    println!("{}", rc::static_kak());
  }

  // server logic implies short-circuiting the rest; hence why we have to pass it &cli to check some stuff once the
  // server is started, like whether we started from Kakoune / the session name / etc.
  if cli.server {
    // server code has logging enabled, so we need to enable it first
    if let Some(level) = Verbosity::from_count(cli.verbose).to_level() {
      simple_logger::init_with_level(level)?;
    }

    return Server::bootstrap(&config, &cli);
  }

  if let Some(request) = cli.request {
    // otherwise, regular client
    let req =
      serde_json::from_str::<UnixRequest>(&request).map_err(|err| OhNo::InvalidRequest {
        req: request,
        err: err.to_string(),
      })?;
    let req = if let Some(session) = cli.session {
      req.with_session(session)
    } else {
      req
    };

    return Server::send_request(req);
  }

  Err(OhNo::NothingToDo)
}
