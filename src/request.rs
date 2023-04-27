//! Requests that can be sent to the server from Kakoune.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A unique way to identify a buffer.
///
/// Currently tagged by the session name and the buffer name.
#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct BufferId {
  session: String,
  buffer: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
  /// Ask the server to daemon to close and clean up.
  Shutdown,

  /// Ask to highlight the given buffer.
  Highlight {
    buffer_id: BufferId,
    lang: String,
    path: PathBuf,
  },
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::{BufferId, Request};

  #[test]
  fn serialization() {
    let req = Request::Highlight {
      buffer_id: BufferId {
        session: "session".to_owned(),
        buffer: "foo".to_owned(),
      },
      lang: "rust".to_owned(),
      path: PathBuf::from("/tmp/foo.rs"),
    };
    let expected = r#"{"type":"highlight","buffer_id":{"session":"session","buffer":"foo"},"lang":"rust","path":"/tmp/foo.rs"}"#;
    let serialized = serde_json::to_string(&req);

    assert_eq!(serialized.unwrap(), expected);
  }
}
