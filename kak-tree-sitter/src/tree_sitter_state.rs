//! Tree-sitter state (i.e. highlighting, tree walking, etc.)

use tree_sitter::{Parser, Query, QueryCursor};

use crate::{error::OhNo, highlighting::KakHighlightRange, languages::Language, text_objects};

/// State around a tree.
///
/// A tree-sitter tree represents a parsed buffer in a given state. It can be walked with queries and updated.
pub struct TreeState {
  parser: Parser,
  tree: tree_sitter::Tree,

  // TODO: for now, we don’t support custom highligthing, and hence have to use tree-sitter-highlight; see
  // #26 for further information
  highlighter: tree_sitter_highlight::Highlighter,
}

impl TreeState {
  pub fn new(lang: &Language, buf: &str) -> Result<Self, OhNo> {
    let mut parser = Parser::new();
    parser.set_language(lang.lang())?;

    let tree = parser
      .parse(buf.as_bytes(), None)
      .ok_or(OhNo::CannotParseBuffer)?;

    let highlighter = tree_sitter_highlight::Highlighter::new();

    Ok(Self {
      parser,
      tree,
      highlighter,
    })
  }

  pub fn highlight<'a>(
    &'a mut self,
    lang: &'a Language,
    buf: &'a str,
    injection_callback: impl FnMut(&str) -> Option<&'a tree_sitter_highlight::HighlightConfiguration>
      + 'a,
  ) -> Result<Vec<KakHighlightRange>, OhNo> {
    self.text_objects(
      lang,
      buf,
      text_objects::Type {
        pattern: text_objects::Pattern::Function,
        level: text_objects::Level::Inside,
      },
    )?;

    let events = self
      .highlighter
      .highlight(&lang.hl_config, buf.as_bytes(), None, injection_callback)
      .map_err(|err| OhNo::HighlightError {
        err: err.to_string(),
      })?;

    Ok(KakHighlightRange::from_iter(
      buf,
      &lang.hl_names,
      events.flatten(),
    ))
  }

  /// Get the text-objects for the given type.
  pub fn text_objects(
    &self,
    lang: &Language,
    buf: &str,
    ty: text_objects::Type,
  ) -> Result<(), OhNo> {
    // first, check whether the language supports text-objects, and also check whether it has the text-object type in
    // its capture names
    let query = lang
      .textobject_query
      .as_ref()
      .ok_or_else(|| OhNo::UnsupportedTextObjects)?;
    let capture_index = query
      .capture_index_for_name(ty.as_query_name())
      .ok_or_else(|| OhNo::UnknownTextObjectQuery { ty })?;

    // run the query via a query cursor
    let mut cursor = QueryCursor::new();
    let mut captures = cursor
      .captures(query, self.tree.root_node(), buf.as_bytes())
      .flat_map(|(cm, _)| cm.captures.iter())
      .filter(|cq| cq.index == capture_index)
      .collect::<Vec<_>>();

    log::info!("text_objects: {captures:#?}");

    Ok(())
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
