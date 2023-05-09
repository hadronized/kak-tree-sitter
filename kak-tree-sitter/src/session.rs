use std::{io::Write, process::Stdio};

use serde::{Deserialize, Serialize};

use crate::response::Response;

#[derive(Debug, Deserialize, Serialize)]
pub struct KakSession {
  pub session_name: String,
  pub client_name: Option<String>,
}

impl KakSession {
  pub fn new(session_name: impl Into<String>, client_name: impl Into<Option<String>>) -> Self {
    Self {
      session_name: session_name.into(),
      client_name: client_name.into(),
    }
  }

  pub fn send_response(&mut self, resp: &Response) {
    let resp = resp.to_kak_cmd(self.client_name.as_deref());

    match resp {
      Some(resp) if !resp.is_empty() => self.send_response_raw(resp),

      _ => (),
    }
  }

  // FIXME: I’m not entirely sure why but something is off with UnixStream. It’s like we’re not correctly connected with the right address?!
  pub fn send_response_raw(&mut self, resp: impl AsRef<str>) {
    let resp = resp.as_ref();

    let child = std::process::Command::new("kak")
      .args(["-p", self.session_name.as_str()])
      .stdin(Stdio::piped())
      .spawn()
      .unwrap(); // FIXME: unwrap()
    let mut child_stdin = child.stdin.unwrap(); // FIXME: unwrap()
    child_stdin.write_all(resp.as_bytes()).unwrap(); // FIXME: unwrap
    child_stdin.flush().unwrap(); // FIXME: unwrap
  }
}
