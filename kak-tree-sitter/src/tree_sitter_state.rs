//! Tree-sitter state (i.e. highlighting, tree walking, etc.)

use tree_sitter::{Parser, Point, QueryCapture, QueryCursor};

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
  /// This function takes in a list of selections and a mode of operation.
  pub fn text_objects(
    &self,
    lang: &Language,
    buf: &str,
    pattern: &str,
    selections: &[Sel],
    mode: &text_objects::OperationMode,
  ) -> Result<(), OhNo> {
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

    match mode {
      text_objects::OperationMode::Next => {
        for sel in selections {
          if let Some(found) = Self::find_next_text_object(sel, &captures[..]) {
            log::info!("new selection {found:?}");
          }
        }
      }

      text_objects::OperationMode::Prev => todo!(),
      text_objects::OperationMode::Inside => todo!(),
      text_objects::OperationMode::Around => todo!(),
      text_objects::OperationMode::Select => todo!(),
      text_objects::OperationMode::Split => todo!(),
    }

    Ok(())
  }

  fn point_to_pos(p: &Point) -> Pos {
    Pos {
      line: p.row,
      col: p.column,
    }
  }

  /// Find the next text-object for a given selection. If found, return a new [`Sel`].
  fn find_next_text_object(sel: &Sel, captures: &[&QueryCapture]) -> Option<Sel> {
    let p = sel.anchor.max(sel.cursor);
    let mut candidates = captures
      .iter()
      .filter(|c| Self::point_to_pos(&c.node.start_position()) >= p)
      .collect::<Vec<_>>();
    candidates.sort_by_key(|c| c.node.start_byte());
    let candidate = candidates.first()?;
    let start = Self::point_to_pos(&candidate.node.start_position());
    let end = Self::point_to_pos(&candidate.node.end_position());

    Some(sel.replace(&start, &end))
  }
}
