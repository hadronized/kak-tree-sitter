mod cli;
mod config;
mod daemon;
mod handler;
mod highlighting;
mod languages;
mod rc;
mod request;
mod response;
mod session;

use clap::Parser;
use cli::Cli;
use daemon::Daemon;
use request::Request;
use session::KakSession;

fn main() {
  let cli = Cli::parse();

  // server logic
  if cli.daemonize {
    Daemon::start();
    std::process::exit(0);
  }

  // client logic
  if let Some(session) = cli.session {
    let mut kak_sess = KakSession::new(session, cli.client);

    if cli.kakoune {
      // inject the rc/
      kak_sess.send_response_raw(rc::rc_commands());
    }

    if let Some(request) = cli.request {
      // parse the request payload and embed it in a request
      let payload = serde_json::from_str(&request).unwrap(); // FIXME: unwrap()
      let req = Request::new(kak_sess, payload);
      Daemon::send_request(req);
    } else {
      eprintln!("no request");
      std::process::exit(1);
    }
  } else {
    eprintln!("missing session");
    std::process::exit(1);
  }
}
