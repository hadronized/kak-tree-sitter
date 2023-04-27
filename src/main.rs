mod cli;
mod config;
mod daemon;
mod handler;
mod highlighting;
mod languages;
mod rc;
mod request;
mod response;

use clap::Parser;
use cli::Cli;
use daemon::Daemon;

use std::{io::Write, process::Stdio};

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
      kak_sess.send(rc::rc_commands());
    }

    if let Some(request) = cli.request {
      Daemon::send_request(request);
    } else {
      eprintln!("no request");
      std::process::exit(1);
    }
  } else {
    eprintln!("missing session");
    std::process::exit(1);
  }
}

#[derive(Debug)]
struct KakSession {
  session_name: String,
  client_name: Option<String>,
}

impl KakSession {
  fn new(session_name: impl Into<String>, client_name: impl Into<Option<String>>) -> Self {
    Self {
      session_name: session_name.into(),
      client_name: client_name.into(),
    }
  }

  /// Format a command to send to Kakoune.
  ///
  /// If `client_name` exists, it will be added to provide additional context and more commands (like info, etc.).
  fn fmt_cmd(&self, cmd: impl AsRef<str>) -> String {
    let cmd = cmd.as_ref();

    if let Some(ref client_name) = self.client_name {
      format!("eval -client {client_name} '{cmd}'\n")
    } else {
      format!("{}\n", cmd)
    }
  }

  /// FIXME: I’m not entirely sure why but something is off with UnixStream. It’s like we’re not correctly connected with the right address?!
  fn send(&mut self, cmd: impl AsRef<str>) {
    let child = std::process::Command::new("kak")
      .args(["-p", self.session_name.as_str()])
      .stdin(Stdio::piped())
      .spawn()
      .unwrap(); // FIXME: unwrap()
    let mut child_stdin = child.stdin.unwrap(); // FIXME: unwrap()
    child_stdin.write_all(self.fmt_cmd(cmd).as_bytes()).unwrap(); // FIXME: unwrap
    child_stdin.flush().unwrap(); // FIXME: unwrap
  }
}
