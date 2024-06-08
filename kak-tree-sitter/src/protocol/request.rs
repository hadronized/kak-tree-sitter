//! Requests that can be sent to the server from Kakoune.

use serde::{Deserialize, Serialize};

use crate::{error::OhNo, kakoune::text_objects::OperationMode, tree_sitter::nav};

use super::response::{self, Response};

/// Request.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct Request {
  session: String,
  client: Option<String>,
  buffer: Option<String>,
  payload: Payload,
}

impl Request {
  /// Parse a [`Request`] from a JSON string.
  pub fn from_json(s: impl AsRef<str>) -> Result<Self, OhNo> {
    let s = s.as_ref();
    serde_json::from_str(s).map_err(|err| OhNo::InvalidRequest {
      req: s.to_owned(),
      err: err.to_string(),
    })
  }

  pub fn init_session(session: impl Into<String>) -> Self {
    Self {
      session: session.into(),
      client: None,
      buffer: None,
      payload: Payload::SessionBegin,
    }
  }

  pub fn session(&self) -> &str {
    &self.session
  }

  pub fn buffer(&self) -> Option<&str> {
    self.buffer.as_deref()
  }

  pub fn payload(&self) -> &Payload {
    &self.payload
  }

  pub fn reply(&self, payload: response::Payload) -> Response {
    Response::new(
      self.session.clone(),
      self.client.clone(),
      self.buffer.clone(),
      payload,
    )
  }
}

/// Request payload.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
  /// Inform the server that a session exists and that we should be sending back
  /// the Kakoune commands to get the server features.
  SessionBegin,

  /// Inform the server that a session has exited.
  SessionEnd,

  /// Ask the server to reload its configuration and reload grammars / queries.
  Reload,

  /// Ask the server to shutdown.
  Shutdown,

  /// Buffer metadata.
  ///
  /// This should be sent every time the buffer changes (lang, mostly).
  BufferMetadata { lang: String },

  /// Buffer close.
  BufferClose,

  /// Request to apply text-objects on selections.
  TextObjects {
    buffer: String,
    pattern: String,
    selections: String,
    mode: OperationMode,
  },

  /// Request to navigate the tree-sitter tree on selections.
  Nav {
    buffer: String,
    selections: String,
    dir: nav::Dir,
  },
}

/// Possible way of updating a buffer.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BufferUpdate {
  /// The full buffer is sent over the buffer FIFO.
  Full,
}
