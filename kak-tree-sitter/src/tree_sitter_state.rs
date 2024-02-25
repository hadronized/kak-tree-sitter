//! Tree-sitter state (i.e. highlighting, tree walking, etc.)

use std::cmp::Reverse;

use tree_sitter::{Parser, QueryCapture, QueryCursor};

use crate::{
  error::OhNo,
  highlighting::KakHighlightRange,
  languages::Language,
  selection::{Pos, Sel},
  text_objects,
};

/// State around a tree.
///
/// A tree-sitter tree represents a parsed buffer in a given state. It can be walked with queries and updated.
pub struct TreeState {
  // this will be useful for #26
  _parser: Parser,
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
      _parser: parser,
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

  /// Get the text-objects for the given pattern.
  ///
  /// This function takes in a list of selections and a mode of operation, and return new selections, depending on the
  /// mode.
  pub fn text_objects(
    &self,
    lang: &Language,
    buf: &str,
    pattern: &str,
    selections: &[Sel],
    mode: &text_objects::OperationMode,
  ) -> Result<Vec<Sel>, OhNo> {
    // first, check whether the language supports text-objects, and also check whether it has the text-object type in
    // its capture names
    let query = lang
      .textobject_query
      .as_ref()
      .ok_or_else(|| OhNo::UnsupportedTextObjects)?;
    let capture_index =
      query
        .capture_index_for_name(pattern)
        .ok_or(OhNo::UnknownTextObjectQuery {
          pattern: pattern.to_owned(),
        })?;

    // run the query via a query cursor
    let mut cursor = QueryCursor::new();
    let captures = cursor
      .captures(query, self.tree.root_node(), buf.as_bytes())
      .flat_map(|(cm, _)| cm.captures.iter())
      .filter(|cq| cq.index == capture_index)
      .collect::<Vec<_>>();

    let sels = match mode {
      text_objects::OperationMode::Next => selections
        .iter()
        .flat_map(|sel| Self::find_next_text_object(sel, &captures[..]))
        .collect(),

      text_objects::OperationMode::Prev => selections
        .iter()
        .flat_map(|sel| Self::find_prev_text_object(sel, &captures[..]))
        .collect(),

      text_objects::OperationMode::Inside => todo!(),
      text_objects::OperationMode::Around => todo!(),
      text_objects::OperationMode::Select => todo!(),
      text_objects::OperationMode::Split => todo!(),
    };

    Ok(sels)
  }

  /// Find the next text-object for a given selection. If found, return a new [`Sel`].
  fn find_next_text_object(sel: &Sel, captures: &[&QueryCapture]) -> Option<Sel> {
    let mut candidates = captures
      .iter()
      .filter(|c| Pos::from(c.node.start_position()) >= sel.cursor)
      .collect::<Vec<_>>();
    candidates.sort_by_key(|c| c.node.start_byte());
    let candidate = candidates.first()?;
    let start = Pos::from(candidate.node.start_position());
    let mut end = Pos::from(candidate.node.end_position());
    end.col -= 1;

    Some(sel.replace(&start, &end))
  }

  /// Find the prev text-object for a given selection. If found, return a new [`Sel`].
  fn find_prev_text_object(sel: &Sel, captures: &[&QueryCapture]) -> Option<Sel> {
    let mut candidates = captures
      .iter()
      .filter(|c| Pos::from(c.node.start_position()) <= sel.cursor)
      .collect::<Vec<_>>();
    candidates.sort_by_key(|c| Reverse(c.node.start_byte()));
    let candidate = candidates.first()?;
    let start = Pos::from(candidate.node.start_position());
    let mut end = Pos::from(candidate.node.end_position());
    end.col -= 1;

    Some(sel.replace(&start, &end))
  }
}
