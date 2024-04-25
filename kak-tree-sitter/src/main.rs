mod buffer;
mod cli;
mod error;
mod fifo;
mod handler;
mod highlighting;
mod languages;
mod logging;
mod nav;
mod queries;
mod rc;
mod request;
mod response;
mod selection;
mod server;
mod session;
mod text_objects;
mod tree_sitter_state;

use clap::Parser;
use cli::Cli;
use error::OhNo;
use kak_tree_sitter_config::Config;
use logging::Verbosity;
use request::UnixRequest;
use server::Server;

use crate::logging::KakouneLogger;

fn main() {
  if let Err(err) = start() {
    log::error!("{err}");
    std::process::exit(1);
  }
}

fn start() -> Result<(), OhNo> {
  let cli = Cli::parse();

  if let Some(level) = Verbosity::from_count(cli.verbose).to_level() {
    if cli.kakoune {
      KakouneLogger::new(level).register()?;
    } else {
      simple_logger::init_with_level(level)?;
    }
  }

  if cli.kakoune {
    println!("{}", rc::static_kak());
  }

  if cli.with_text_objects {
    println!("{}", rc::text_objects_kak());
  }

  if cli.server {
    let config = Config::load_default_user()?;
    log::trace!("running with configuration:\n{config:#?}");
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
