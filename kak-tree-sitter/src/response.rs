//! Response sent from the daemon to Kakoune, typically via the socket interface (kak -p, etc.).

use std::path::PathBuf;

use itertools::Itertools;

use crate::highlighting::KakHighlightRange;
use crate::kak;

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

  /// Explicit deinit response when the daemon exits.
  ///
  /// This is sent to all connected sessions to ask them to deinit when the server is going down. This is important as
  /// a KTS-enabled session will use various resources (UNIX sockets, FIFOs, etc.) to communicate with KTS, and most of
  /// those will block on Kakoune.
  Deinit,

  /// Whether a filetype is supported.
  FiletypeSupported {
    supported: bool,
    remove_default_highlighter: bool,
  },

  /// Highlights.
  ///
  /// This response is generated when new highlights are asked.
  Highlights {
    timestamp: u64,
    ranges: Vec<KakHighlightRange>,
  },

  TextObject {
    timestamp: u64,
    obj_type: String,
    range: kak::LocRange,
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

      Response::Deinit => "kak-tree-sitter-deinit".to_owned(),

      Response::FiletypeSupported {
        supported,
        remove_default_highlighter,
      } => {
        if *supported {
          [
            Some("kak-tree-sitter-highlight-enable"),
            remove_default_highlighter
              .then_some(r#"try %{ remove-highlighter "window/%opt{filetype}" }"#),
          ]
          .into_iter()
          .flatten()
          .join("\n")
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

      Response::TextObject {
        timestamp,
        obj_type,
        range,
      } => {
        format!("select -timestamp {timestamp} {range}")
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

/// Response that can be sent to a specific session.
#[derive(Debug, Eq, PartialEq)]
pub struct ConnectedResponse {
  pub session: String,
  pub client: Option<String>,
  pub resp: Response,
}

impl ConnectedResponse {
  pub fn new(
    session: impl Into<String>,
    client: impl Into<Option<String>>,
    resp: Response,
  ) -> Self {
    let session = session.into();
    let client = client.into();

    Self {
      session,
      client,
      resp,
    }
  }
}
