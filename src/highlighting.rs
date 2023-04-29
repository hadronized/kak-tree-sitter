//! Convert from tree-sitter-highlight events to Kakoune ranges highlighter.

use std::{collections::HashMap, fs, path::Path};

use tree_sitter::Language;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

use crate::{queries::Queries, request::BufferId, response::Response};

/// Session/buffer highlighters.
///
/// This type maps a [`BufferId`] with a tree-sitter highlighter.
pub struct Highlighters {
  highlighters: HashMap<BufferId, Highlighter>,
  hl_names: Vec<String>,
}

impl Highlighters {
  pub fn new(hl_names: impl Into<Vec<String>>) -> Self {
    Highlighters {
      highlighters: HashMap::new(),
      hl_names: hl_names.into(),
    }
  }
}

impl Highlighters {
  pub fn highlight(
    &mut self,
    lang: Language,
    queries: &Queries,
    buffer_id: BufferId,
    path: impl AsRef<Path>,
  ) -> Response {
    println!("parsing {buffer_id:?}");

    let source = fs::read_to_string(path).unwrap(); // FIXME: unwrap()

    let highlighter = self
      .highlighters
      .entry(buffer_id)
      .or_insert(Highlighter::new());
    // re-parse
    let mut hl_config = HighlightConfiguration::new(lang, &queries.highlights, "", "").unwrap();
    hl_config.configure(&self.hl_names); // FIXME: config

    let events = highlighter
      .highlight(&hl_config, source.as_bytes(), None, |_| None)
      .unwrap();

    let ranges = KakHighlightRange::from_iter(&source, &self.hl_names, events.flatten());

    Response::Highlights { ranges }
  }
}

/// A convenient representation of a single highlight range for Kakoune.
///
/// `:doc highlighters`, `ranges`, for further documentation.
#[derive(Debug, Eq, PartialEq)]
pub struct KakHighlightRange {
  line_start: usize,
  col_start: usize,
  line_end: usize,
  col_end: usize,
  face: String,
}

impl KakHighlightRange {
  pub fn new(
    line_start: usize,
    col_start: usize,
    line_end: usize,
    col_end: usize,
    face: impl Into<String>,
  ) -> Self {
    Self {
      line_start,
      col_start,
      line_end,
      col_end,
      face: face.into(),
    }
  }

  /// Given an iterator of [`HighlightEvent`], generate a list of Kakoune highlights.
  pub fn from_iter(
    source: &str,
    hl_names: &[String],
    hl_events: impl Iterator<Item = HighlightEvent>,
  ) -> Vec<Self> {
    let mut kak_hls = Vec::new();
    let mut faces: Vec<&str> = Vec::new();
    let mut mapper = ByteLineColMapper::new(source.char_indices());

    // iterate on the highlight event
    for event in hl_events {
      match event {
        HighlightEvent::Source { start, end } => {
          if start == end {
            continue;
          }

          println!("{start}-{end}");

          mapper.advance(start);
          let line_start = mapper.line();
          let col_start = mapper.col();

          mapper.advance(end - 1);
          let line_end = mapper.line();
          let col_end = mapper.col();

          let face = faces.last().copied().unwrap_or("unknown");

          kak_hls.push(KakHighlightRange::new(
            line_start,
            col_start,
            line_end,
            col_end,
            face.replace('.', "_"),
          ));
        }

        HighlightEvent::HighlightStart(Highlight(idx)) => {
          faces.push(&hl_names[idx]);
        }

        HighlightEvent::HighlightEnd => {
          faces.pop();
        }
      }
    }

    println!("{kak_hls:#?}");
    kak_hls
  }

  /// Display as a string recognized by the `ranges` Kakoune highlighter.
  pub fn to_kak_range_str(&self) -> String {
    format!(
      "{}.{},{}.{}|ts_{}",
      self.line_start, self.col_start, self.line_end, self.col_end, self.face
    )
  }
}

/// Map byte indices to line and column.
#[derive(Debug)]
struct ByteLineColMapper<C> {
  chars: C,
  byte_idx: usize,
  line: usize,
  col: usize,
  change_line: bool,
}

impl<C> ByteLineColMapper<C>
where
  C: Iterator<Item = (usize, char)>,
{
  fn new(mut chars: C) -> Self {
    chars.next();

    Self {
      chars,
      byte_idx: 0,
      line: 1,
      col: 1,
      change_line: false,
    }
  }

  fn line(&self) -> usize {
    self.line
  }

  fn col(&self) -> usize {
    self.col
  }

  fn advance(&mut self, til: usize) {
    loop {
      if self.byte_idx >= til {
        break;
      }

      if let Some((idx, c)) = self.chars.next() {
        println!("read {c}");
        self.byte_idx = idx;

        if self.change_line {
          self.line += 1;
          self.col = 0;
        }

        self.change_line = c == '\n';

        // TODO: we probably want to compute the « display width » of `c` here instead
        self.col += 1;
      } else {
        break;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::ByteLineColMapper;

  #[test]
  fn byte_line_col_mapper() {
    let source = "const x: &'str = \"Hello, world!\";\nconst y = 3;";
    let mut mapper = ByteLineColMapper::new(source.char_indices());

    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col(), 1);

    mapper.advance(4);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col(), 5);

    mapper.advance(33);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col(), 34);

    mapper.advance(34);
    assert_eq!(mapper.line(), 2);
    assert_eq!(mapper.col(), 1);
  }
}
