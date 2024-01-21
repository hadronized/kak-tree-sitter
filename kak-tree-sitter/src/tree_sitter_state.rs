//! Tree-sitter state (i.e. highlighting, tree walking, etc.)

use tree_sitter::{Language as TSLanguage, Parser, Query, QueryCursor};
use tree_sitter_highlight::Highlighter;

use crate::error::OhNo;
use crate::highlighting::KakHighlightRange;
use crate::languages::Language;
use crate::languages::Languages;

/// State around a tree.
///
/// A tree-sitter tree represents a parsed buffer in a given state. It can be walked with queries and updated.
pub struct TreeState {
  highlighter: Highlighter,
  parser: Parser,
  tree: tree_sitter::Tree,
}

impl TreeState {
  pub fn new(lang: &Language) -> Result<Self, OhNo> {
    let mut parser = Parser::new();
    parser.set_language(lang.lang());
    parser.set_timeout_micros(1000);

    let highlighter = Highlighter::new();

    let tree = parser.parse("", None).unwrap();

    Ok(Self {
      highlighter,
      parser,
      tree,
    })
  }

  pub fn parser_mut(&mut self) -> &mut Parser {
    // TODO: Can this be merged with the highlighters parser?
    &mut self.parser
  }

  pub fn parser(&self) -> &Parser {
    // TODO: Can this be merged with the highlighters parser?
    &self.parser
  }

  pub fn lang(&self) -> TSLanguage {
    self.parser.language().unwrap() // Is set in constructor
  }

  pub fn update(&mut self, buf: &str) -> Result<(), OhNo> {
    self.tree = self
      .parser
      .parse(buf.as_bytes(), None)
      .ok_or(OhNo::CannotParseBuffer)?;
    Ok(())
  }

  pub fn highlight(
    &mut self,
    lang: &Language,
    langs: &Languages,
    source: &str,
  ) -> Result<Vec<KakHighlightRange>, OhNo> {
    // TODO: Merge this logic with the one parser
    crate::highlighting::highlight(&mut self.highlighter, lang, langs, source)
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
