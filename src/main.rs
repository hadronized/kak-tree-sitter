use std::{io::Write, process::Stdio, time::Duration};

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about = "A client/server interface between Kakoune and tree-sitter.")]
pub struct Cli {
  /// Kakoune session to connect to.
  #[clap(short, long)]
  session: String,

  /// Kakoune client to connect with, if any.
  #[clap(short, long)]
  client: Option<String>,
}

fn main() {
  let cli = Cli::parse();

  if cli.session.is_empty() {
    std::process::exit(1);
  }

  let kak_sess = KakSession::new(cli.session, cli.client);

  kak_sess.send("info Connected!");
  std::thread::sleep(Duration::from_secs(5));
  kak_sess.send("info Disconnected!");
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
      format!("eval -try-client {client_name} {cmd}")
    } else {
      format!("{cmd}")
    }
  }

  fn send(&self, cmd: impl AsRef<str>) {
    let child = std::process::Command::new("kak")
      .args(["-p", self.session_name.as_str()])
      .stdin(Stdio::piped())
      .spawn()
      .unwrap(); // FIXME: unwrap()
    let mut child_stdin = child.stdin.unwrap(); // FIXME: unwrap()
    child_stdin.write_all(self.fmt_cmd(cmd).as_bytes()).unwrap(); // FIXME: unwrap
  }
}
