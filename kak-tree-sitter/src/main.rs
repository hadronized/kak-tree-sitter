mod cli;
mod client;
mod error;
mod kakoune;
mod logging;
mod protocol;
mod server;
mod tree_sitter;

use std::sync::Arc;

use clap::Parser;
use cli::Cli;
use error::OhNo;
use kak_tree_sitter_config::Config;
use logging::Verbosity;
use mio::Poll;
use protocol::request::Request;
use server::{
  resources::{Paths, ServerResources},
  Server,
};

use crate::{kakoune::rc, logging::KakouneLogger};

fn main() {
  if let Err(err) = start() {
    log::error!("{err}");
    std::process::exit(1);
  }
}

fn start() -> Result<(), OhNo> {
  let cli = Cli::parse();

  // setup logger; if we start from kakoune, the logger implementation sends to
  // the *debug* buffer
  if let Some(level) = Verbosity::from_count(cli.verbose).to_level() {
    if cli.kakoune {
      KakouneLogger::new(level).register()?;
    } else {
      simple_logger::init_with_level(level)?;
    }
  }

  let config = Config::load_default_user()?;
  log::trace!("running with configuration:\n{config:#?}");

  // inject rc if we start from Kakoune
  if cli.kakoune && cli.init.is_some() {
    println!("{}", rc::static_kak());

    if cli.with_text_objects || config.features.text_objects {
      println!("{}", rc::text_objects_kak());
    }
  }

  let paths = Paths::new()?;

  if let Some(request) = cli.request {
    // otherwise, regular client
    let req = serde_json::from_str::<Request>(&request).map_err(|err| OhNo::InvalidRequest {
      req: request,
      err: err.to_string(),
    })?;

    let mut client = client::Client::connect(&paths)?;
    client.send(&req)?;

    // if we sent the request from within Kakoune, we return nop command so that we can call the commands and print
    // errors along the way
    if cli.kakoune {
      println!("nop");
    }

    return Ok(());
  }

  if cli.server {
    let poll = Poll::new().map_err(|err| OhNo::PollError { err })?;

    // whatever we do, we will need to know about where the resources are; this
    // object ensures the resources are created and expose interfaces to them
    let registry = Arc::new(
      poll
        .registry()
        .try_clone()
        .map_err(|err| OhNo::PollError { err })?,
    );

    if Server::is_server_running(&paths) {
      if let Some(session) = cli.init {
        log::debug!("server already running, but initiating first session {session}");
        client::Client::init_session(&paths, session)?;
      }

      return Ok(());
    }

    let resources = ServerResources::new(paths, registry)?;
    resources.persist_process(cli.daemonize)?;

    let mut server = Server::new(&config, &cli, resources, poll)?;

    if let Some(session) = cli.init {
      server.init_first_session(session)?;
    }

    return server.start();
  }

  Err(OhNo::NothingToDo)
}
