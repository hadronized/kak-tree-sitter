//! Client implementation.
//!
//! This module exports the [`Client`] type that is used to send requests to the
//! server.

use std::{io::Write, os::unix::net::UnixStream};

use crate::{error::OhNo, protocol::request::Request, server::resources::Paths};

/// Connected client (UNIX socket).
#[derive(Debug)]
pub struct Client {
  stream: UnixStream,
}

impl Client {
  pub fn connect(paths: &Paths) -> Result<Self, OhNo> {
    let stream = UnixStream::connect(paths.socket_path())
      .map_err(|err| OhNo::CannotConnectToServer { err })?;

    Ok(Self { stream })
  }

  /// Convenient method to connect to the server and initiate a session.
  pub fn init_session(paths: &Paths, session: impl Into<String>) -> Result<(), OhNo> {
    let mut client = Self::connect(paths)?;
    client.send(&Request::init_session(session))
  }

  /// Asynchronously send a request.
  pub fn send(&mut self, req: &Request) -> Result<(), OhNo> {
    let bytes = serde_json::to_string(req).map_err(|err| OhNo::CannotSendRequest {
      err: err.to_string(),
    })?;

    self
      .stream
      .write_all(bytes.as_bytes())
      .map_err(|err| OhNo::CannotSendRequest {
        err: err.to_string(),
      })
  }
}
