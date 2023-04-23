mod rc;
mod request;

use clap::Parser;
use std::{io::Write, os::unix::net::UnixStream, process::Stdio};

#[derive(Debug, Parser)]
#[clap(about = "A client/server interface between Kakoune and tree-sitter.")]
pub struct Cli {
  /// Whether we start from Kakoune and then we should inject the rc/.
  #[clap(short, long)]
  kakoune: bool,

  /// Try to daemonize, if not already done.
  #[clap(short, long)]
  daemonize: bool,

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

  let mut kak_sess = KakSession::new(cli.session, cli.client);

  if cli.kakoune {
    // inject the rc/
    kak_sess.send(rc::rc_commands());
  }

  if cli.daemonize {
    // TODO: check if we are already daemonized; if not, start as a daemon; then quit
  }
}

fn daemonize() {}

#[derive(Debug)]
struct KakSession {
  session_name: String,
  client_name: Option<String>,
  unix_stream: UnixStream,
}

impl KakSession {
  fn new(session_name: impl AsRef<str>, client_name: impl Into<Option<String>>) -> Self {
    let user = std::env::var("USER").unwrap(); // FIXME: unwrap
    let tmpdir = std::env::var("TMPDIR").unwrap(); // FIXME: unwrap
    let session_name = session_name.as_ref();
    let path = format!("{tmpdir}kakoune-{user}/{session_name}");

    println!("connecting to {path}…");
    let unix_stream = UnixStream::connect(path).unwrap(); // FIXME: unwrap
    println!("connected! {unix_stream:?}");

    Self {
      session_name: session_name.into(),
      client_name: client_name.into(),
      unix_stream,
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

    //let cmd = self.fmt_cmd(cmd);
    //self.unix_stream.write_all(cmd.as_bytes()).unwrap(); // FIXME: unwrap
    //self.unix_stream.flush().unwrap(); // FIXME: unwrap
  }
}
