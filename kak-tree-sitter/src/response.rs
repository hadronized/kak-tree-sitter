//! Response sent from the daemon to Kakoune, typically via the socket interface (kak -p, etc.).

use std::path::PathBuf;

use itertools::Itertools;

use crate::{highlighting::KakHighlightRange, rc};

/// Response sent by the daemon to Kakoune.
#[derive(Debug, Eq, PartialEq)]
pub enum Response {
  /// Status change.
  StatusChanged { status: String },

  /// Initial response when a session starts.
  Init { cmd_fifo_path: PathBuf },

  /// Whether a filetype is supported.
  FiletypeSupported { supported: bool },

  /// Highlights.
  ///
  /// This response is generated when new highlights are asked.
  Highlights {
    timestamp: u64,
    ranges: Vec<KakHighlightRange>,
  },
}

impl Response {
  pub fn status(status: impl Into<String>) -> Self {
    Response::StatusChanged {
      status: status.into(),
    }
  }

  pub fn to_kak_cmd<'a>(&self, client_name: impl Into<Option<&'a str>>) -> Option<String> {
    let kak_cmd = match self {
      Response::StatusChanged { status, .. } => format!("info %{{{}}}", status),

      Response::Init { cmd_fifo_path } => {
        format!(
          "{rc}\nset-option global kts_cmd_fifo_path {cmd_fifo_path}",
          rc = rc::rc_commands(),
          cmd_fifo_path = cmd_fifo_path.display()
        )
      }

      Response::FiletypeSupported { supported } => {
        if *supported {
          "kak-tree-sitter-highlight-enable".to_owned()
        } else {
          "".to_owned()
        }
      }

      Response::Highlights { timestamp, ranges } => {
        let ranges_str = ranges
          .iter()
          .map(KakHighlightRange::to_kak_range_str)
          .join(" ");

        format!(
          "{range_specs} {timestamp} {ranges_str}",
          range_specs = "set buffer kak_tree_sitter_highlighter_ranges",
        )
      }
    };

    // empty command means no response
    if kak_cmd.is_empty() {
      return Some(kak_cmd);
    }

    // check if we need to build a command prefix
    let cmd_prefix = if let Some(client_name) = client_name.into() {
      format!("-client {client_name} ")
    } else {
      String::new()
    };

    Some(format!("eval -no-hooks {cmd_prefix}%{{{kak_cmd}}}"))
  }
}
