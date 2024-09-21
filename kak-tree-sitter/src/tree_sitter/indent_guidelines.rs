//! Indent guidelines support in Kakoune.

use std::fmt::Write as _;

const INDENT_GUIDELINE_CHAR: char = '│';

/// Indent guidelines for a whole buffer.
///
/// The guidelines are sorted by lines and optimized to generate a single
/// highlighter item with spaces for several guidelines annotation on the
/// same line, instead of having several highlight items.
///
/// This is not a contiguous array of indent guidelines, as
#[derive(Debug)]
pub struct IndentGuidelines {
  lines: Vec<IndentGuideline>,
}

impl IndentGuidelines {
  pub fn new(lines: impl Into<Vec<IndentGuideline>>) -> Self {
    Self {
      lines: lines.into(),
    }
  }
}

/// Indent guideline for a given line.
#[derive(Debug)]
pub struct IndentGuideline {
  start_line: usize,
  end_line: usize,
  col: usize,
}

impl IndentGuideline {
  pub fn new(start_line: usize, end_line: usize, col: usize) -> Self {
    Self {
      start_line,
      end_line,
      col,
    }
  }

  /// Display as a string recognized by the `replace-ranges` Kakoune
  /// highlighter.
  pub fn to_kak_replace_ranges_str(&self) -> String {
    let mut output = String::new();
    let col = self.col;

    // we aways ignore the first and last line
    for line in self.start_line + 1..self.end_line {
      write!(&mut output, "{line}.{col}+1|{INDENT_GUIDELINE_CHAR} ").unwrap();
    }

    output
  }

  /// Display as a string recognized by the `ranges` Kakoune highlighter.
  pub fn to_kak_ranges_str(&self) -> String {
    let mut output = String::new();
    let col = self.col;

    // we aways ignore the first and last line
    for line in self.start_line + 1..self.end_line {
      write!(&mut output, "{line}.{col}+1|ts_indent_guideline ").unwrap();
    }

    output
  }
}
