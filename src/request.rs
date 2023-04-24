//! Requests that can be sent to the server from Kakoune.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
  /// Ask to highlight the given buffer.
  Highlight {
    session_name: String,
    buffer_name: String,
    lang: String,
    content: String,
  },
}

#[cfg(test)]
mod tests {
  use super::Request;

  #[test]
  fn serialization() {
    let req = Request::Highlight {
      session_name: "session".to_owned(),
      buffer_name: "foo".to_owned(),
      lang: "rust".to_owned(),
      content: String::new(),
    };
    let expected = r#"{"type":"highlight","session_name":"session","buffer_name":"foo","lang":"rust","content":""}"#;
    let serialized = serde_json::to_string(&req);

    assert_eq!(serialized.unwrap(), expected);
  }
}
