//! Tree-sitter state (i.e. highlighting, tree walking, etc.)

use tree_sitter::{Language, Parser, Query, QueryCursor};

use crate::error::OhNo;

/// State around a tree.
///
/// A tree-sitter tree represents a parsed buffer in a given state. It can be walked with queries and updated.
#[derive(Debug)]
pub struct TreeState {
  tree: tree_sitter::Tree,
}

impl TreeState {
  pub fn new(lang: Language, buf: &str) -> Result<Self, OhNo> {
    let mut parser = Parser::new();
    parser.set_language(lang);
    parser.set_timeout_micros(1000);

    parser
      .parse(buf.as_bytes(), None)
      .ok_or(OhNo::CannotParseBuffer)
      .map(|tree| Self { tree })
  }

  pub fn query(&self, query: &Query, code: &str) {
    let mut cursor = QueryCursor::new();
    let captures = cursor.captures(query, self.tree.root_node(), code.as_bytes());
    let names = query.capture_names();

    for (query_match, size) in captures {
      for capture in query_match.captures {
        log::info!(
          "--> {}: {:#?} {:?} // {:?}",
          &code[capture.node.byte_range()],
          capture.node.kind(),
          names.get(capture.index as usize),
          capture
        );
      }
    }
  }
}
