mod cli;
mod client;
mod error;
mod kakoune;
mod logging;
mod protocol;
mod server;
mod tree_sitter;

use clap::Parser;
use cli::Cli;
use error::OhNo;
use kak_tree_sitter_config::Config;
use logging::Verbosity;
use protocol::request::UnixRequest;
use server::{resources::ServerResources, Server};

use crate::{kakoune::rc, logging::KakouneLogger};

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

  // whatever we do, we will need to know about where the resources are
  let resources = ServerResources::new()?;

  if cli.server {
    if resources.is_server_running() {
        return Ok(())
    }
    let config = Config::load_default_user()?;

    log::trace!("running with configuration:\n{config:#?}");
    let server = Server::new(&config, &cli, resources)?;
    server.prepare(cli.daemonize)?;
    return server.start();
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

    return req.send(&resources);
  }

  Err(OhNo::NothingToDo)
}
