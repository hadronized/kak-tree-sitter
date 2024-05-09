//! Response sent from the daemon to Kakoune.

use std::path::PathBuf;

use itertools::Itertools;

use crate::{kakoune::selection::Sel, tree_sitter::highlighting::KakHighlightRange};

/// Response sent by the daemon to Kakoune.
#[derive(Debug, Eq, PartialEq)]
pub enum Response {
  /// Status change.
  Info { status: String },

  /// Initial response when a session starts.
  Init {
    cmd_fifo_path: PathBuf,
    buf_fifo_path: PathBuf,
    with_highlighting: bool,
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

  /// Selections.
  ///
  /// These selections are typically returned when the user asked to perform text-objects queries.
  Selections { sels: Vec<Sel> },
}

impl Response {
  pub fn info(status: impl Into<String>) -> Self {
    Response::Info {
      status: status.into(),
    }
  }

  /// Turn the [`Response`] into a Kakoune command that can be executed remotely.
  pub fn to_kak_cmd(&self, client: Option<&str>) -> Option<String> {
    let kak_cmd = match self {
      Response::Info { status, .. } => {
        format!("info %{{{status}}}",)
      }

      Response::Init {
        cmd_fifo_path,
        buf_fifo_path,
        with_highlighting,
      } => {
        let mut resp = format!(
          "set-option global kts_cmd_fifo_path {cmd}\n
           set-option global kts_buf_fifo_path {buf}",
          cmd = cmd_fifo_path.display(),
          buf = buf_fifo_path.display(),
        );

        if *with_highlighting {
          resp.push_str(
            "\nkak-tree-sitter-enable-highlighting\n
               kak-tree-sitter-req-enable",
          );
        }

        resp
      }

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

      Response::Selections { sels } => {
        let sels_str = sels.iter().map(|sel| sel.to_kak_str()).join(" ");
        format!("select {sels_str}")
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
