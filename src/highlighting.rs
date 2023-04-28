//! Convert from tree-sitter-highlight events to Kakoune ranges highlighter.

use tree_sitter_highlight::{Highlight, HighlightEvent};

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
    hl_names: &[&str],
    hl_events: impl Iterator<Item = HighlightEvent>,
  ) -> Vec<Self> {
    let mut kak_hls = Vec::new();
    let mut faces: Vec<&str> = Vec::new();
    let mut chars = source.char_indices();
    let mut line = 1;
    let mut col = 1;
    let mut byte_i = 0;

    let mut advance_til = |line: &mut usize, col: &mut usize, til_byte: usize| {
      while byte_i != til_byte {
        if let Some((byte, c)) = chars.next() {
          byte_i = byte;

          if c == '\n' {
            *line += 1;
            *col = 1;
          } else {
            *col += 1;
          }

          if byte_i == til_byte {
            break;
          }
        } else {
          break;
        }
      }
    };

    // iterate on the highlight event
    for event in hl_events {
      match event {
        HighlightEvent::Source { start, end } => {
          if start == end {
            continue;
          }

          advance_til(&mut line, &mut col, start);
          let line_start = line;
          let col_start = col;

          advance_til(&mut line, &mut col, end - 1);
          let line_end = line;
          let col_end = col;

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
          faces.push(hl_names[idx]);
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
      "{}.{},{}.{}|ts_{}",
      self.line_start, self.col_start, self.line_end, self.col_end, self.face
    )
  }
}
