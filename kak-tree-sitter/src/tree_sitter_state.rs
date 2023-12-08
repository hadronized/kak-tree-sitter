//! Tree-sitter state (i.e. highlighting, tree walking, etc.)

use tree_sitter::{Language, Parser, Query, QueryCursor};

use crate::{error::OhNo, queries::Queries};

/// State around a tree.
///
/// A tree-sitter tree represents a parsed buffer in a given state. It can be walked with queries and updated.
pub struct TreeState {
  tree: tree_sitter::Tree,

  // TODO: for now, we don’t support custom highligthing, and hence have to use tree-sitter-highlight; see
  // #26 for further information
  highlighter: tree_sitter_highlight::Highlighter,
  highlight_conf: tree_sitter_highlight::HighlightConfiguration,
}

impl TreeState {
  pub fn new(lang: Language, queries: &Queries, buf: &str) -> Result<Self, OhNo> {
    let mut parser = Parser::new();
    parser.set_language(lang)?;
    parser.set_timeout_micros(1000);

    let tree = parser
      .parse(buf.as_bytes(), None)
      .ok_or(OhNo::CannotParseBuffer)?;

    let highlighter = tree_sitter_highlight::Highlighter::new();
    let highlight_conf = tree_sitter_highlight::HighlightConfiguration::new(
      lang,
      queries.highlights.as_deref().unwrap_or_default(),
      queries.injections.as_deref().unwrap_or_default(),
      queries.locals.as_deref().unwrap_or_default(),
    )
    .map_err(|err| OhNo::HighlightError {
      err: err.to_string(),
    })?;

    Ok(Self {
      tree,
      highlighter,
      highlight_conf,
    })
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
