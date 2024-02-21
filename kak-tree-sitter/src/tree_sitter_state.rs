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

  pub fn tree(&self) -> &tree_sitter::Tree {
    &self.tree
  }

  pub fn query<'a>(
    &'a self,
    cursor: &'a mut QueryCursor,
    query: &'a Query,
    code: &'a str,
  ) -> impl Iterator<Item = &'a tree_sitter::QueryCapture<'a>> {
    let captures = cursor.captures(query, self.tree.root_node(), code.as_bytes());

    captures.flat_map(|(qm, _size)| qm.captures)
  }
}
