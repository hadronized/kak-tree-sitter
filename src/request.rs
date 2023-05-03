//! Requests that can be sent to the server from Kakoune.

use serde::{Deserialize, Serialize};

use crate::session::KakSession;

/// A unique way to identify a buffer.
///
/// Currently tagged by the session name and the buffer name.
#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct BufferId {
  session: String,
  buffer: String,
}

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

  /// Ask to highlight the given buffer.
  Highlight {
    buffer_id: BufferId,
    lang: String,
    timestamp: u64,
    stream_name: String,
  },
}

#[cfg(test)]
mod tests {
  use super::{BufferId, RequestPayload};

  #[test]
  fn serialization() {
    let req = RequestPayload::Highlight {
      buffer_id: BufferId {
        session: "session".to_owned(),
        buffer: "foo".to_owned(),
      },
      lang: "rust".to_owned(),
      timestamp: 0,
      stream_name: "foo.rs".to_owned(),
    };
    let expected = r#"{"type":"highlight","buffer_id":{"session":"session","buffer":"foo"},"lang":"rust","timestamp":0,"stream_name":"foo.rs"}"#;
    let serialized = serde_json::to_string(&req);

    assert_eq!(serialized.unwrap(), expected);
  }
}
