//! Tree-sitter navigation.

use serde::{Deserialize, Serialize};

/// Every possible navigation directions.
///
/// Navigation within a tree-sitter tree is provided via several directions.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dir {
  /// Parent node.
  Parent,

  /// First child of the current node, if any.
  FirstChild,

  // NOTE: currently not available due to tree-sitter version
  // /// Last child of the current node, if any.
  // LastChild,
  /// First sibling of the current node, if any.
  FirstSibling,

  // NOTE: currently not available due to tree-sitter version
  // /// Last sibling of the current node if any.
  // LastSibling,
  /// Previous sibiling of the current node, if any.
  PrevSibling {
    /// Should we take cousins into account?
    #[serde(default)]
    cousin: bool,
  },

  /// Next sibling of the current node, if any.
  NextSibling {
    /// Should we take cousins into account?
    cousin: bool,
  },
}
