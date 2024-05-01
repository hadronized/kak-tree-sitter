//! Convert from tree-sitter-highlight events to Kakoune ranges highlighter.

use tree_sitter_highlight::{Highlight, HighlightEvent};
use unicode_segmentation::UnicodeSegmentation;

/// A convenient representation of a single highlight range for Kakoune.
///
/// `:doc highlighters`, `ranges`, for further documentation.
#[derive(Debug, Eq, PartialEq)]
pub struct KakHighlightRange {
  line_start: usize,
  col_byte_start: usize,
  line_end: usize,
  col_byte_end: usize,
  face: String,
}

impl KakHighlightRange {
  pub fn new(
    line_start: usize,
    col_byte_start: usize,
    line_end: usize,
    col_byte_end: usize,
    face: impl Into<String>,
  ) -> Self {
    Self {
      line_start,
      col_byte_start,
      line_end,
      col_byte_end,
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
    let mut mapper = ByteLineColMapper::new(source.graphemes(true));

    // iterate on the highlight event
    for event in hl_events {
      match event {
        HighlightEvent::Source { start, end } => {
          if start == end {
            continue;
          }

          mapper.advance(start);
          let line_start = mapper.line();
          let col_byte_start = mapper.col_byte();

          mapper.advance(end - 1);
          let line_end = mapper.line();
          let col_byte_end = mapper.col_byte();

          let face = faces.last().copied().unwrap_or("unknown");

          kak_hls.push(KakHighlightRange::new(
            line_start,
            col_byte_start,
            line_end,
            col_byte_end,
            format!("ts_{}", face.replace('.', "_")),
          ));
        }

        HighlightEvent::HighlightStart(Highlight(idx)) => {
          if idx >= hl_names.len() {
            log::error!(
              "unrecognized highlight group index: {idx} (len: {len}), groups = {hl_names:?}",
              len = hl_names.len()
            );
          } else {
            faces.push(&hl_names[idx]);
          }
        }

        HighlightEvent::HighlightEnd => {
          faces.pop();
        }
      }
    }

    kak_hls
  }

  /// Display as a string recognized by the `ranges` Kakoune highlighter.
  pub fn to_kak_range_str(&self) -> String {
    format!(
      "{}.{},{}.{}|{}",
      self.line_start,
      self.col_byte_start + 1, // range-specs is 1-indexed
      self.line_end,
      self.col_byte_end + 1, // ditto
      self.face
    )
  }
}

/// Map byte indices to line and column.
#[derive(Debug)]
struct ByteLineColMapper<C> {
  chars: C,
  byte_idx: usize,
  line: usize,
  col_byte: usize,
}

impl<'a, C> ByteLineColMapper<C>
where
  C: Iterator<Item = &'a str>,
{
  fn new(chars: C) -> Self {
    Self {
      chars,
      byte_idx: 0,
      line: 1,
      col_byte: 0,
    }
  }

  fn line(&self) -> usize {
    self.line
  }

  fn col_byte(&self) -> usize {
    self.col_byte
  }

  fn should_change_line(s: &str) -> bool {
    ["\n", "\r\n"].contains(&s)
  }

  /// Advance the mapper until the given byte is read (or just passed over).
  fn advance(&mut self, til: usize) {
    loop {
      if self.byte_idx >= til {
        break;
      }

      if let Some(grapheme) = self.chars.next() {
        let bytes = grapheme.as_bytes().len();
        self.byte_idx += bytes;

        if Self::should_change_line(grapheme) {
          self.line += 1;
          self.col_byte = 0;
        } else {
          self.col_byte += bytes;
        }
      } else {
        break;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};
  use unicode_segmentation::UnicodeSegmentation;

  use super::ByteLineColMapper;

  #[test]
  fn idempotent_mapper() {
    let source = "Hello, world!";
    let mut mapper = ByteLineColMapper::new(source.graphemes(true));

    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 0);

    mapper.advance(1);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 1);
    mapper.advance(1);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 1);

    mapper.advance(4);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 4);
    mapper.advance(4);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 4);
  }

  #[test]
  fn lines_mapper() {
    let source = "const x: &'str = \"Hello, world!\";\nconst y = 3;";
    let mut mapper = ByteLineColMapper::new(source.graphemes(true));

    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 0);

    mapper.advance(4);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 4);

    mapper.advance(33);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 33);

    mapper.advance(34);
    assert_eq!(mapper.line(), 2);
    assert_eq!(mapper.col_byte(), 0);

    mapper.advance(35);
    assert_eq!(mapper.line(), 2);
    assert_eq!(mapper.col_byte(), 1);
  }

  #[test]
  fn unicode_mapper() {
    let source = "const ᾩ = 1"; // the unicode symbol is 3-bytes
    let mut mapper = ByteLineColMapper::new(source.graphemes(true));

    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 0);

    mapper.advance(1);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 1);

    mapper.advance(5);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 5);

    mapper.advance(6);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 6);

    mapper.advance(7);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 9);

    mapper.advance(8);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 9);

    mapper.advance(9);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 9);
  }

  #[test]
  fn unicode_mapper_more() {
    let source = "× a"; // 2 bytes
    let mut mapper = ByteLineColMapper::new(source.graphemes(true));

    mapper.advance(1);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 2);
  }

  #[test]
  fn newline_mapper() {
    let source = "×\na"; // 2 bytes, 1 byte, 1 byte
    let mut mapper = ByteLineColMapper::new(source.graphemes(true));

    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 0);

    mapper.advance(1);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 2);

    mapper.advance(2);
    assert_eq!(mapper.line(), 1);
    assert_eq!(mapper.col_byte(), 2);

    mapper.advance(3);
    assert_eq!(mapper.line(), 2);
    assert_eq!(mapper.col_byte(), 0);
  }

  #[test]
  fn kak_hl_ranges_from_iter() {
    let source = "fn foo(a: i32, b: /* ® */ impl Into<Option<String>>) {}";
    let hl_names = vec![
      "constant",
      "function",
      "keyword",
      "variable",
      "punctuation",
      "type",
      "comment",
    ];

    let mut hl_conf = HighlightConfiguration::new(
      tree_sitter_rust::language(),
      tree_sitter_rust::HIGHLIGHT_QUERY,
      tree_sitter_rust::INJECTIONS_QUERY,
      "",
    )
    .unwrap();
    hl_conf.configure(&hl_names);

    let mut hl = Highlighter::new();
    let events: Vec<_> = hl
      .highlight(&hl_conf, source.as_bytes(), None, |_| None)
      .unwrap()
      .flatten()
      .collect();

    assert_eq!(events.len(), 70);

    assert!(matches!(
      events[..],
      [
        HighlightEvent::HighlightStart(Highlight(2)),
        HighlightEvent::Source { start: 0, end: 2 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::Source { start: 2, end: 3 },
        HighlightEvent::HighlightStart(Highlight(1)),
        HighlightEvent::Source { start: 3, end: 6 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 6, end: 7 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(3)),
        HighlightEvent::Source { start: 7, end: 8 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 8, end: 9 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::Source { start: 9, end: 10 },
        HighlightEvent::HighlightStart(Highlight(5)),
        HighlightEvent::Source { start: 10, end: 13 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 13, end: 14 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::Source { start: 14, end: 15 },
        HighlightEvent::HighlightStart(Highlight(3)),
        HighlightEvent::Source { start: 15, end: 16 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 16, end: 17 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::Source { start: 17, end: 18 },
        HighlightEvent::HighlightStart(Highlight(6)),
        HighlightEvent::Source { start: 18, end: 26 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::Source { start: 26, end: 27 },
        HighlightEvent::HighlightStart(Highlight(2)),
        HighlightEvent::Source { start: 27, end: 31 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::Source { start: 31, end: 32 },
        HighlightEvent::HighlightStart(Highlight(5)),
        HighlightEvent::Source { start: 32, end: 36 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 36, end: 37 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(5)),
        HighlightEvent::Source { start: 37, end: 43 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 43, end: 44 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(5)),
        HighlightEvent::Source { start: 44, end: 50 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 50, end: 51 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 51, end: 52 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 52, end: 53 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::Source { start: 53, end: 54 },
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 54, end: 55 },
        HighlightEvent::HighlightEnd,
        HighlightEvent::HighlightStart(Highlight(4)),
        HighlightEvent::Source { start: 55, end: 56 },
        HighlightEvent::HighlightEnd
      ]
    ));
  }
}
