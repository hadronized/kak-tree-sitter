//! Response sent from the daemon to Kakoune, typically via the socket interface (kak -p, etc.).

use std::path::PathBuf;

use itertools::Itertools;

use crate::highlighting::KakHighlightRange;

/// Response sent by the daemon to Kakoune.
#[derive(Debug, Eq, PartialEq)]
pub enum Response {
  /// Status change.
  StatusChanged { status: String },

  /// Initial response when a session starts.
  Init {
    cmd_fifo_path: PathBuf,
    buf_fifo_path: PathBuf,
  },

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

  pub fn to_kak_cmd(&self, client: Option<&str>) -> Option<String> {
    let kak_cmd = match self {
      Response::StatusChanged { status, .. } => {
        format!("info %{{{status}}}",)
      }

      Response::Init {
        cmd_fifo_path,
        buf_fifo_path,
      } => [
        format!(
          "set-option global kts_cmd_fifo_path {path}",
          path = cmd_fifo_path.display()
        ),
        format!(
          "set-option global kts_buf_fifo_path {path}",
          path = buf_fifo_path.display()
        ),
        "kak-tree-sitter-req-enable".to_owned(),
      ]
      .join("\n"),

      Response::FiletypeSupported { supported } => {
        if *supported {
          "kak-tree-sitter-highlight-enable".to_owned()
        } else {
          String::new()
        }
      }

      Response::Highlights { timestamp, ranges } => {
        let ranges_str = ranges
          .iter()
          .map(KakHighlightRange::to_kak_range_str)
          .join(" ");

        format!(
          "{range_specs} {timestamp} {ranges_str}",
          range_specs = "set buffer kts_highlighter_ranges",
        )
      }
    };

    // empty command means no response
    if kak_cmd.is_empty() {
      return None;
    }

    let prefix = if let Some(client) = client {
      format!("-try-client {client} ")
    } else {
      String::new()
    };

    Some(format!("eval -no-hooks {prefix}%{{{kak_cmd}}}"))
  }
}
