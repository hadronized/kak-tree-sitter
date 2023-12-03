//! Tree-sitter state (i.e. highlighting, tree walking, etc.)

use tree_sitter::{Query, QueryCursor};

/// State around a tree.
///
/// A tree-sitter tree represents a parsed buffer in a given state. It can walked with queries and updated.
#[derive(Debug)]
pub struct TreeState {
  tree: tree_sitter::Tree,
}

impl TreeState {
  pub fn new(tree: tree_sitter::Tree) -> Self {
    Self { tree }
  }

  pub fn query(&self, query: &Query, code: &str) {
    let mut cursor = QueryCursor::new();
    let captures = cursor.captures(query, self.tree.root_node(), code.as_bytes());

    for (query_match, size) in captures {
      for capture in query_match.captures {
        log::info!("--> {:?}", capture);
      }
    }
  }
}
