//! Convert from tree-sitter-highlight events to Kakoune ranges highlighter.

use tree_sitter_highlight::{Highlight, HighlightEvent};

/// A convenient representation of a single highlight range for Kakoune.
///
/// `:doc highlighters`, `ranges`, for further documentation.
#[derive(Debug, Eq, PartialEq)]
pub struct KakHighlight {
  line_start: usize,
  col_start: usize,
  line_end: usize,
  col_end: usize,
  face: String,
}

impl KakHighlight {
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

    let mut advance_til = |line: &mut usize, col: &mut usize, til_byte: usize| {
      while let Some((byte, c)) = chars.next() {
        println!("read {c} at position {byte}; line = {line}, col = {col}");

        if byte == til_byte {
          break;
        }

        if c == '\n' {
          *line += 1;
          *col = 1;
        } else {
          *col += 1;
        }
      }
    };

    // iterate on the highlight event
    for event in hl_events {
      match event {
        HighlightEvent::Source { start, end } => {
          println!("event: {start}-{end}");

          if start == end {
            continue;
          }

          advance_til(&mut line, &mut col, start);
          let line_start = line;
          let col_start = col;

          advance_til(&mut line, &mut col, end);
          let line_end = line;
          let col_end = col;

          let face = faces.last().cloned().unwrap_or_default();

          kak_hls.push(KakHighlight::new(
            line_start,
            col_start,
            line_end,
            col_end,
            face.to_owned(),
          ));
        }

        HighlightEvent::HighlightStart(Highlight(idx)) => {
          println!("event: push {idx}");
          faces.push(hl_names[idx]);
        }

        HighlightEvent::HighlightEnd => {
          println!("event: pop");
          faces.pop();
        }
      }
    }

    kak_hls
  }

  /// Display as a string recognized by the `ranges` Kakoune highlighter.
  pub fn as_ranges_str(&self) -> String {
    format!(
      "{}.{},{}.{}|{}",
      self.line_start, self.col_start, self.line_end, self.col_end, self.face
    )
  }
}
