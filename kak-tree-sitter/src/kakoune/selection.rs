//! Selections as recognized by Kakoune, as well as associated types and functions.

use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Point};

/// A single position in a buffer.
///
/// Kakoune position _1-based_, while tree-sitter selections are _0-based_.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Pos {
  pub line: usize,
  pub col: usize,
}

impl From<Point> for Pos {
  fn from(p: Point) -> Self {
    Self {
      line: p.row + 1,
      col: p.column + 1,
    }
  }
}

impl From<Pos> for Point {
  fn from(p: Pos) -> Self {
    Self {
      row: p.line - 1,
      column: p.col - 1,
    }
  }
}

impl Pos {
  /// Read a [`Pos`] from Kakoune-formatted string; i.e. <line>.<col>.
  ///
  /// Return [`None`] if parsing failed.
  pub fn parse_kak_str(s: &str) -> Option<Self> {
    let mut parts = s.split('.').flat_map(|s| s.parse().ok());
    let line = parts.next()?;
    let col = parts.next()?;

    Some(Self { line, col })
  }
}

/// A single Kakoune selection, containing an anchor and a cursor.
///
/// Note: there is no rule about anchors and cursors. One can come before the other; do not assume anything about their
/// position.
///
/// Kakoune selections are always inclusive, while tree-sitter ranges are exclusive on their end boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sel {
  pub anchor: Pos,
  pub cursor: Pos,
}

impl Sel {
  /// Read a [`Sel`] from Kakoune-formatted string; i.e. <anchor_line>.<anchor_col>,<cursor_line>.<cursor_col>.
  ///
  /// Return [`None`] if parsing failed.
  pub fn parse_kak_str(s: &str) -> Option<Self> {
    let mut parts = s.split(',').flat_map(Pos::parse_kak_str);
    let anchor = parts.next()?;
    let cursor = parts.next()?;

    Some(Self { anchor, cursor })
  }

  /// Parse many [`Sel`] from a space-separated list of selection.
  pub fn parse_many(s: &str) -> Vec<Self> {
    s.split_whitespace().flat_map(Self::parse_kak_str).collect()
  }

  /// Kakoune string representation.
  ///
  /// The anchor always come first; then the cursor.
  pub fn to_kak_str(&self) -> String {
    format!(
      "{anchor_line}.{anchor_col},{cursor_line}.{cursor_col}",
      anchor_line = self.anchor.line,
      anchor_col = self.anchor.col,
      cursor_line = self.cursor.line,
      cursor_col = self.cursor.col
    )
  }

  /// Replace a selection with two other points.
  ///
  /// This function replaces the selection with two other points by keeping the order anchor / cursor; if the anchor is
  /// before, the new anchor will be before; if the cursor is before the anchor, the new cursor will still be before the
  /// new anchor.
  pub fn replace(&self, a: &Pos, b: &Pos) -> Self {
    if self.anchor <= self.cursor {
      Self {
        anchor: *a,
        cursor: *b,
      }
    } else {
      Self {
        anchor: *b,
        cursor: *a,
      }
    }
  }

  /// Same as [`Sel::replace`], but with a node’s content.
  pub fn replace_with_node(&self, node: &Node) -> Self {
    let mut b: Pos = node.end_position().into();
    b.col -= 1; // kakoune selections are inclusive
    self.replace(&node.start_position().into(), &b)
  }

  /// Check whether a selection selects a node.
  pub fn selects(&self, node: &Node) -> bool {
    let start: Pos = node.start_position().into();
    let mut end: Pos = node.end_position().into();
    end.col -= 1;

    if self.anchor <= self.cursor {
      self.anchor <= start && self.cursor >= end
    } else {
      self.cursor <= start && self.anchor >= end
    }
  }

  /// Check whether a selection fully selects a node.
  pub fn fully_selects(&self, node: &Node) -> bool {
    let start: Pos = node.start_position().into();
    let mut end: Pos = node.end_position().into();
    end.col -= 1;

    if self.anchor <= self.cursor {
      self.anchor == start && self.cursor == end
    } else {
      self.cursor == start && self.anchor == end
    }
  }
}

/// Object flags; used in object mode.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ObjectFlags {
  pub to_begin: bool,
  pub to_end: bool,
  pub inner: bool,
}

impl ObjectFlags {
  pub fn parse_kak_str(s: &str) -> Self {
    s.split('|').fold(Self::default(), |flags, s| match s {
      "to_begin" => flags.set_to_begin(),
      "to_end" => flags.set_to_end(),
      "inner" => flags.set_inner(),
      _ => flags,
    })
  }

  pub fn set_to_begin(mut self) -> Self {
    self.to_begin = true;
    self
  }

  pub fn set_to_end(mut self) -> Self {
    self.to_end = true;
    self
  }

  pub fn set_inner(mut self) -> Self {
    self.inner = true;
    self
  }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectMode {
  Replace,
  Extend,
}

#[cfg(test)]
mod tests {
  use super::{Pos, Sel};

  #[test]
  fn pos_parsing() {
    let s = "123.456";
    let parsed = Pos::parse_kak_str(s);
    assert_eq!(
      parsed,
      Some(Pos {
        line: 123,
        col: 456
      })
    );
  }

  #[test]
  fn sel_parsing() {
    let s = "123.456,124.789";
    let parsed = Sel::parse_kak_str(s);
    assert_eq!(
      parsed,
      Some(Sel {
        anchor: Pos {
          line: 123,
          col: 456,
        },

        cursor: Pos {
          line: 124,
          col: 789
        }
      })
    );
  }

  #[test]
  fn replace_sel() {
    let anchor_cursor = Sel {
      anchor: Pos {
        line: 123,
        col: 456,
      },

      cursor: Pos {
        line: 124,
        col: 789,
      },
    };
    let cursor_anchor = Sel {
      anchor: Pos {
        line: 124,
        col: 789,
      },

      cursor: Pos {
        line: 123,
        col: 456,
      },
    };
    let a = Pos { line: 1, col: 1 };
    let b = Pos {
      line: 1000,
      col: 1000,
    };

    assert_eq!(
      anchor_cursor.replace(&a, &b),
      Sel {
        anchor: Pos {
          line: a.line,
          col: a.col,
        },

        cursor: Pos {
          line: b.line,
          col: b.col
        }
      }
    );

    assert_eq!(
      cursor_anchor.replace(&a, &b),
      Sel {
        anchor: Pos {
          line: b.line,
          col: b.col,
        },

        cursor: Pos {
          line: a.line,
          col: a.col
        }
      }
    );
  }
}
