//! Requests that can be sent to the server from Kakoune.

use std::fmt::Debug;
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::session::KakSession;

/// Unidentified request (i.e. not linked to a given session).
#[derive(Debug, Deserialize, Serialize)]
pub enum UnidentifiedRequest {
  /// Inform KTS that a new session exists and that we should be sending back the Kakoune commands to get KTS features.
  NewSession { name: String },

  /// Inform KTS that a session has exited.
  SessionExit { name: String },
}

impl UnidentifiedRequest {
  /// Add a session name to a [`UnidentifiedRequest`], replacing it if one was already provided.
  pub fn with_session(self, name: impl Into<String>) -> Self {
    let name = name.into();

    match self {
      UnidentifiedRequest::NewSession { .. } => UnidentifiedRequest::NewSession { name },
      UnidentifiedRequest::SessionExit { .. } => UnidentifiedRequest::SessionExit { name },
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Request<Origin>
where
  Origin: RequestOrigin,
{
  pub session: KakSession,
  pub payload: RequestPayload<Origin>,
}

/// Origin of a request.
///
/// Used to reinterpret request payloads.
pub trait RequestOrigin {
  /// Payload type for the [`Request::Highlight`] variant.
  type HighlightPayload: Debug + DeserializeOwned + Serialize;
}

#[derive(Debug, Deserialize, Serialize)]
pub enum KakouneOrigin {}

impl RequestOrigin for KakouneOrigin {
  /// This is a FIFO to read from.
  type HighlightPayload = PathBuf;
}

#[derive(Debug, Deserialize, Serialize)]
pub enum KakTreeSitterOrigin {}

impl RequestOrigin for KakTreeSitterOrigin {
  /// This is buffer content.
  type HighlightPayload = String;
}

/// Request payload.
///
/// Request payload are parameterized with the « origin » at which requests are expected.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestPayload<Origin>
where
  Origin: RequestOrigin,
{
  /// A session just ended.
  ///
  /// This request is useful to track which sessions are still alive, and eventually make the daemon quit by itself.
  SessionEnd,

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
    payload: Origin::HighlightPayload,
  },
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::request::{KakTreeSitterOrigin, KakouneOrigin};

  use super::RequestPayload;

  #[test]
  fn serialization() {
    let kak_req = RequestPayload::<KakouneOrigin>::Highlight {
      buffer: "/tmp/a.rs".to_owned(),
      lang: "rust".to_owned(),
      timestamp: 0,
      payload: PathBuf::from("/tmp/a.fifo"),
    };
    let expected = r#"{"type":"highlight","buffer":"/tmp/a.rs","lang":"rust","timestamp":0,"payload":"/tmp/a.fifo"}"#;
    let serialized = serde_json::to_string(&kak_req);

    assert_eq!(serialized.unwrap(), expected);

    let kts_req = RequestPayload::<KakTreeSitterOrigin>::Highlight {
      buffer: "/tmp/a.rs".to_owned(),
      lang: "rust".to_owned(),
      timestamp: 0,
      payload: "lol".to_owned(),
    };
    let expected =
      r#"{"type":"highlight","buffer":"/tmp/a.rs","lang":"rust","timestamp":0,"payload":"lol"}"#;
    let serialized = serde_json::to_string(&kts_req);

    assert_eq!(serialized.unwrap(), expected);
  }
}
