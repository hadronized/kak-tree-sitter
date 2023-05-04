mod cli;
mod config;
mod daemon;
mod handler;
mod highlighting;
mod languages;
mod queries;
mod rc;
mod request;
mod response;
mod session;

use std::process::exit;

use clap::Parser;
use cli::Cli;
use config::Config;
use daemon::Daemon;
use request::Request;
use session::KakSession;

fn main() {
  let cli = Cli::parse();
  let config = Config::load_from_xdg();

  if cli.kakoune {
    // inject the rc/ and daemon-based settings
    println!("{}", rc::rc_commands());
  }

  // server logic
  if cli.daemonize {
    Daemon::start(config);
    exit(0);
  }

  // client logic
  if let Some(session) = cli.session {
    let kak_sess = KakSession::new(session, cli.client);

    if let Some(request) = cli.request {
      // parse the request payload and embed it in a request
      let payload = serde_json::from_str(&request).unwrap(); // FIXME: unwrap()
      let req = Request::new(kak_sess, payload);
      Daemon::send_request(req);
    } else {
      eprintln!("no request");
      exit(1);
    }
  } else {
    eprintln!("missing session");
    exit(1);
  }
}
