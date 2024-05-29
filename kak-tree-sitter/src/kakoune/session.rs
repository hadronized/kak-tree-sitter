use std::{
  collections::HashMap,
  io::Write,
  process::{Command, Stdio},
};

use crate::{error::OhNo, protocol::response::Response};

/// Session tracker.
///
/// Responsible for tracking sessions (by names).
#[derive(Debug, Default)]
pub struct SessionTracker {
  sessions: HashMap<String, Session>,
}

impl SessionTracker {
  pub fn is_empty(&self) -> bool {
    self.sessions.is_empty()
  }

  /// Check whether a session is already tracked.
  pub fn tracks(&self, session: &str) -> bool {
    self.sessions.contains_key(session)
  }

  pub fn track(&mut self, session: Session) {
    self.sessions.insert(session.name.clone(), session);
  }

  pub fn untrack(&mut self, session_name: impl AsRef<str>) {
    self.sessions.remove(session_name.as_ref());
  }

  pub fn sessions(&self) -> impl Iterator<Item = &str> {
    self.sessions.keys().map(String::as_str)
  }
}

/// An (active) session.
#[derive(Debug)]
pub struct Session {
  name: String,
}

impl Session {
  /// Create a new [`Session`] for the given name.
  pub fn new(name: impl Into<String>) -> Result<Self, OhNo> {
    Ok(Self { name: name.into() })
  }

  /// Send a response back to Kakoune.
  pub fn send_response(resp: Response) -> Result<(), OhNo> {
    let Some(data) = resp.to_kak() else {
      // FIXME: this is a weird situation where the [`Response`] doesn’t really
      // have any Kakoune counterpart; I plan on removing that ~soon
      return Ok(());
    };

    // spawn the kak -p process
    // TODO: we want to switch that from directly connecting to the UNIX socket
    let mut child = Command::new("kak")
      .args(["-p", resp.session()])
      .stdin(Stdio::piped())
      .spawn()
      .map_err(|err| OhNo::CannotSendRequest {
        err: err.to_string(),
      })?;
    let child_stdin = child
      .stdin
      .as_mut()
      .ok_or_else(|| OhNo::CannotSendRequest {
        err: "cannot pipe data to kak -p".to_owned(),
      })?;

    child_stdin
      .write_all(data.as_bytes())
      .map_err(|err| OhNo::CannotSendRequest {
        err: err.to_string(),
      })?;

    child_stdin.flush().map_err(|err| OhNo::CannotSendRequest {
      err: err.to_string(),
    })?;

    child.wait().map_err(|err| OhNo::CannotSendRequest {
      err: format!("error while waiting on kak -p: {err}"),
    })?;

    Ok(())
  }
}
