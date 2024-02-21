use std::ops::Range;

use serde::Deserialize;

/// Kakoune location. Line and col_byte are zero indexed here,
/// but one indexed when formatted and parsed
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Loc {
  pub line: usize,
  pub col_byte: usize,
}

impl std::fmt::Display for Loc {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}.{}", self.line + 1, self.col_byte + 1)
  }
}

impl From<tree_sitter::Point> for Loc {
  fn from(value: tree_sitter::Point) -> Self {
    // The tree_sitter::Point column is also a byte offset, so this is ok
    Self {
      line: value.row,
      col_byte: value.column,
    }
  }
}

impl From<Loc> for tree_sitter::Point {
  fn from(value: Loc) -> Self {
    Self {
      row: value.line,
      column: value.col_byte,
    }
  }
}

impl Loc {
  pub fn new(line: usize, col_byte: usize) -> Self {
    Self { line, col_byte }
  }

  /// Parse a location from kakoune in the format line.col
  pub fn parse(s: &str) -> Option<Self> {
    let (line, col) = s.split_once('.')?;
    Some(Self {
      line: line.parse::<usize>().ok()?.saturating_sub(1),
      col_byte: col.parse::<usize>().ok()?.saturating_sub(1),
    })
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocRange {
  pub start: Loc,
  pub end: Loc,
}

impl<T: Into<Loc>> From<Range<T>> for LocRange {
  fn from(value: Range<T>) -> Self {
    Self {
      start: value.start.into(),
      end: value.end.into(),
    }
  }
}

impl LocRange {
  pub fn new(start: Loc, end: Loc) -> Self {
    Self { start, end }
  }

  pub fn contains(&self, loc: Loc) -> bool {
    loc >= self.start && loc <= self.end
  }

  pub fn contains_range(&self, range: &LocRange) -> bool {
    self.contains(range.start) && self.contains(range.end)
  }

  /// Parse a location from kakoune in the format `a.b,c.d`, where start.line = a, start.col = b, end.line = c, end.col = d
  pub fn parse_selection(s: &str) -> Option<Self> {
    let (start, end) = s.split_once(',')?;
    Some(Self {
      start: Loc::parse(start)?,
      end: Loc::parse(end)?,
    })
  }
}

impl std::fmt::Display for LocRange {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{},{}", self.start, self.end)
  }
}

impl<'de> Deserialize<'de> for LocRange {
  fn deserialize<D>(deserializer: D) -> Result<LocRange, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    use serde::de::Error;
    let s = <&str>::deserialize(deserializer)?;
    LocRange::parse_selection(s).ok_or(D::Error::custom("Could not parse LocRange"))
  }
}

impl serde::Serialize for LocRange {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.to_string().serialize(serializer)
  }
}
