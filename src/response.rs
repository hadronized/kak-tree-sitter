//! Response sent from the daemon to Kakoune, typically via the socket interface (kak -p, etc.).

use itertools::Itertools;

use crate::highlighting::KakHighlightRange;

/// Response sent by the daemon to Kakoune.
#[derive(Debug, Eq, PartialEq)]
pub enum Response {
  /// Status change.
  StatusChanged { status: String, shutdown: bool },

  /// Highlights.
  ///
  /// This response is generated when new highlights are asked.
  Highlights { ranges: Vec<KakHighlightRange> },
}

impl Response {
  pub fn should_shutdown(&self) -> bool {
    match self {
      Response::StatusChanged { shutdown, .. } => *shutdown,
      Response::Highlights { .. } => false,
    }
  }

  pub fn status(status: impl Into<String>, shutdown: bool) -> Self {
    Response::StatusChanged {
      status: status.into(),
      shutdown,
    }
  }

  pub fn to_kak_cmd<'a>(&self, client_name: impl Into<Option<&'a str>>) -> String {
    let kak_cmd = match self {
      Response::StatusChanged { status, .. } => format!("info '{}'\n", status),
      Response::Highlights { ranges } => {
        let ranges_str = ranges
          .iter()
          .map(KakHighlightRange::to_kak_range_str)
          .join(" ");

        format!(
          "{range_specs_decl} {ranges_str};{highlighter_decl}",
          range_specs_decl = "decl range-specs kak_tree_sitter_highlighter_ranges %val{timestamp}",
          highlighter_decl = "add-highlighter -override buffer/kak-tree-sitter-highlighter ranges kak_tree_sitter_highlighter_ranges"
        )
      }
    };

    if let Some(client_name) = client_name.into() {
      format!("eval -client {client_name} '{kak_cmd}'\n")
    } else {
      kak_cmd
    }
  }
}
