mod cli;
mod daemon;
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
use daemon::Daemon;
use kak_tree_sitter_config::Config;
use request::Request;
use session::KakSession;

fn main() {
  let cli = Cli::parse();
  let config = Config::load_from_xdg();

  if cli.kakoune {
    // inject the rc/ and daemon-based settings
    println!("{}", rc::rc_commands());
  }

  if let (Some(session), Some(request)) = (cli.session, cli.request) {
    // client logic
    let kak_sess = KakSession::new(session, cli.client);

    // parse the request payload and embed it in a request
    let payload = serde_json::from_str(&request).unwrap(); // FIXME: unwrap()
    let req = Request::new(kak_sess, payload);
    Daemon::send_request(req);
  } else {
    // server logic
    let async_rt = tokio::runtime::Runtime::new().unwrap(); // FIXME: unwrap
    async_rt.block_on(Daemon::bootstrap(config, cli.daemonize));
  }
}
