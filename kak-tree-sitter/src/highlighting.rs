//! Convert from tree-sitter-highlight events to Kakoune ranges highlighter.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tree_sitter_highlight::{Highlight, HighlightEvent, Highlighter};

use crate::{
  error::OhNo,
  languages::{Language, Languages},
  response::Response,
};

/// A unique way to identify a buffer.
///
/// Currently tagged by the session name and the buffer name.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct BufferId {
  session: String,
  buffer: String,
}

impl BufferId {
  pub fn new(session: impl Into<String>, buffer: impl Into<String>) -> Self {
    Self {
      session: session.into(),
      buffer: buffer.into(),
    }
  }
}

/// Session/buffer highlighters.
///
/// This type maps a [`BufferId`] with a tree-sitter highlighter.
pub struct Highlighters {
  highlighters: HashMap<BufferId, Highlighter>,
}

impl Highlighters {
  pub fn new() -> Self {
    Highlighters {
      highlighters: HashMap::new(),
    }
  }
}

impl Highlighters {
  pub fn highlight(
    &mut self,
    lang: &Language,
    langs: &Languages,
    buffer_id: BufferId,
    timestamp: u64,
    source: &str,
  ) -> Result<Response, OhNo> {
    let highlighter = self
      .highlighters
      .entry(buffer_id)
      .or_insert(Highlighter::new());

    let injection_callback = |lang_name: &str| langs.get(lang_name).map(|lang| &lang.hl_config);
    let events = highlighter
      .highlight(&lang.hl_config, source.as_bytes(), None, injection_callback)
      .map_err(|err| OhNo::HighlightError {
        err: err.to_string(),
      })?;

    let ranges = KakHighlightRange::from_iter(source, &lang.hl_names, events.flatten());

    Ok(Response::Highlights { timestamp, ranges })
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
    let change_line = matches!(chars.next(), Some((_, '\n')));

    Self {
      chars,
      byte_idx: 0,
      line: 1,
      col: 1,
      change_line,
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
        self.byte_idx = idx;

        if self.change_line {
          self.line += 1;
          self.col = 0;
        }

        self.change_line = c == '\n';

        self.col += c.len_utf8();
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
