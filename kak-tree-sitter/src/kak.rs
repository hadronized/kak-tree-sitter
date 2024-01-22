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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocRange {
  pub anchor: Loc,
  pub cursor: Loc,
}

impl<T: Into<Loc>> From<Range<T>> for LocRange {
  fn from(value: Range<T>) -> Self {
    Self {
      anchor: value.start.into(),
      cursor: value.end.into(),
    }
  }
}

impl LocRange {
  pub fn new(anchor: Loc, cursor: Loc) -> Self {
    Self { anchor, cursor }
  }

  pub fn contains(&self, loc: Loc) -> bool {
    loc >= self.start() && loc <= self.end()
  }

  pub fn contains_range(&self, range: &LocRange) -> bool {
    self.contains(range.cursor) && self.contains(range.anchor)
  }

  /// Parse a location from kakoune in the format `a.b,c.d`, where start.line = a, start.col = b, end.line = c, end.col = d
  pub fn parse_selection(s: &str) -> Option<Self> {
    let (anchor, cursor) = s.split_once(',')?;
    Some(Self {
      anchor: Loc::parse(anchor)?,
      cursor: Loc::parse(cursor)?,
    })
  }

  pub fn start(&self) -> Loc {
    self.cursor.min(self.anchor)
  }

  pub fn end(&self) -> Loc {
    self.cursor.max(self.anchor)
  }

  /// Merge two selections, keeping the order of the cursor/anchor. i.e. if the cursor was at the start of the
  /// selection before, keep it at the start of the selection after
  pub fn extend(&self, other: LocRange) -> LocRange {
    if self.anchor < self.cursor {
      LocRange::new(self.start().min(other.start()), self.end().max(other.end()))
    } else {
      LocRange::new(self.end().max(other.end()), self.start().min(other.start()))
    }
  }
}

impl std::fmt::Display for LocRange {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{},{}", self.anchor, self.cursor)
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

/// The flags stored in kakoune's %val{object_flags}
#[derive(Debug, Default, Clone, Copy, serde::Serialize)]
pub struct ObjectFlags {
  pub to_begin: bool,
  pub to_end: bool,
  pub inner: bool,
}

impl ObjectFlags {
  pub fn parse(s: &str) -> Self {
    let mut res = Self::default();
    for flag in s.split('|') {
      match flag {
        "inner" => res.inner = true,
        "to_begin" => res.to_begin = true,
        "to_end" => res.to_end = true,
        _ => log::warn!("Unexpected object flag from kakoune: {flag}"),
      }
    }
    res
  }
}

impl<'de> Deserialize<'de> for ObjectFlags {
  fn deserialize<D>(deserializer: D) -> Result<ObjectFlags, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = <&str>::deserialize(deserializer)?;
    Ok(ObjectFlags::parse(s))
  }
}

/// The selection mode stored in %val{select_mode}
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum SelectMode {
  #[default]
  #[serde(rename = "replace")]
  Replace,
  #[serde(rename = "extend")]
  Extend,
}
