//! Requests that can be sent to the server from Kakoune.

use serde::{Deserialize, Serialize};

use crate::{kakoune::text_objects::OperationMode, tree_sitter::nav};

/// Request payload.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
  /// Inform the server that a session exists and that we should be sending back
  /// the Kakoune commands to get the server features.
  RegisterSession,

  /// Inform the server that a session has exited.
  SessionExit,

  /// Ask the server to reload its configuration and reload grammars / queries.
  Reload,

  /// Ask the server to shutdown.
  Shutdown,

  /// Try enabling highlighting for a given filetype.
  ///
  /// This request starts a “highlighting session.” The response will not replay with « supports highlighting » or
  /// « does not support highlighting », but instead will insert the Kakoune commands to ask for highlights only if the
  /// filetype is supported.
  TryEnableHighlight { lang: String, client: String },

  /// Ask to highlight the given buffer.
  ///
  /// The content of the buffer is streamed right after in the same command FIFO file the request was sent in.
  Highlight {
    client: String,
    buffer: String,
    lang: String,
    timestamp: u64,
  },

  /// Request to apply text-objects on selections.
  TextObjects {
    client: String,
    buffer: String,
    lang: String,
    pattern: String,
    selections: String,
    mode: OperationMode,
  },

  /// Request to navigate the tree-sitter tree on selections.
  Nav {
    client: String,
    buffer: String,
    lang: String,
    selections: String,
    dir: nav::Dir,
  },
}
