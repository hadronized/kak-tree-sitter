//! Requests that can be sent to the server from Kakoune.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::session::KakSession;

#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
  pub session: KakSession,
  pub payload: RequestPayload,
}

impl Request {
  pub fn new(session: KakSession, payload: RequestPayload) -> Self {
    Self { session, payload }
  }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestPayload {
  /// Ask the server/daemon to close and clean up.
  Shutdown,

  /// Try enabling highlighting for a given filetype.
  ///
  /// This request starts a “highlighting session.” The response will not replay with « supports highlighting » or
  /// « does not support highlighting », but instead will insert the Kakoune commands to ask for highlights only if the
  /// filetype is supported.
  TryEnableHighlight { lang: String },

  /// Ask to highlight the given buffer.
  Highlight {
    buffer: String,
    lang: String,
    timestamp: u64,
    read_fifo: PathBuf,
  },
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::RequestPayload;

  #[test]
  fn serialization() {
    let req = RequestPayload::Highlight {
      buffer: "/tmp/a.rs".to_owned(),
      lang: "rust".to_owned(),
      timestamp: 0,
      read_fifo: PathBuf::from("/tmp/a.fifo"),
    };
    let expected = r#"{"type":"highlight","buffer":"/tmp/a.rs","lang":"rust","timestamp":0,"reda_fifo":"/tmp/a.fifo"}"#;
    let serialized = serde_json::to_string(&req);

    assert_eq!(serialized.unwrap(), expected);
  }
}
