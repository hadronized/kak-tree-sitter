//! Requests that can be sent to the server from Kakoune.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::session::KakSession;

/// Unidentified request (i.e. not linked to a given session).
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnidentifiedRequest {
  /// Inform KTS that a new session exists and that we should be sending back the Kakoune commands to get KTS features.
  NewSession { name: String },

  /// Inform KTS that a session has exited.
  SessionExit { name: String },

  /// Ask KTS to shutdown.
  Shutdown,
}

impl UnidentifiedRequest {
  /// Add a session name to a [`UnidentifiedRequest`], replacing it if one was already provided.
  pub fn with_session(self, name: impl Into<String>) -> Self {
    let name = name.into();

    match self {
      UnidentifiedRequest::NewSession { .. } => UnidentifiedRequest::NewSession { name },
      UnidentifiedRequest::SessionExit { .. } => UnidentifiedRequest::SessionExit { name },
      _ => self,
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
  pub session: KakSession,
  pub payload: RequestPayload,
}

/// Request payload.
///
/// Request payload are parameterized with the « origin » at which requests are expected.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestPayload {
  /// Try enabling highlighting for a given filetype.
  ///
  /// This request starts a “highlighting session.” The response will not replay with « supports highlighting » or
  /// « does not support highlighting », but instead will insert the Kakoune commands to ask for highlights only if the
  /// filetype is supported.
  TryEnableHighlight { lang: String },

  /// Ask to highlight the given buffer.
  ///
  /// The content of the buffer is streamed right after in the same command FIFO file the request was sent in.
  Highlight {
    buffer: String,
    lang: String,
    timestamp: u64,
  },
}

#[cfg(test)]
mod tests {
  use super::RequestPayload;

  #[test]
  fn serialization() {
    let req = RequestPayload::Highlight {
      buffer: "/tmp/a.rs".to_owned(),
      lang: "rust".to_owned(),
      timestamp: 0,
    };
    let expected = r#"{"type":"highlight","buffer":"/tmp/a.rs","lang":"rust","timestamp":0}"#;
    let serialized = serde_json::to_string(&req);

    assert_eq!(serialized.unwrap(), expected);
  }
}
