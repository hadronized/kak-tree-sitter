//! Output table with header and rows.

use std::fmt::Display;

use colored::ColoredString;
use unicode_segmentation::UnicodeSegmentation;

/// A table containing a header and rows.
///
/// Use the [`Display`] implementor to write it to a string.
#[derive(Debug, Default)]
pub struct Table {
  header: Row,
  rows: Vec<Row>,
}

impl Table {
  /// Compute the maximum length for each column.
  fn max_len_by_col(&self) -> impl '_ + Iterator<Item = usize> {
    (0..self.header.cells.len()).map(|i| {
      let max_len = self.header.cells[i].content.graphemes(true).count();
      self.rows.iter().fold(max_len, |max_len, row| {
        row.cells[i].content.graphemes(true).count().max(max_len)
      })
    })
  }

  /// Set the header.
  pub fn header(&mut self, header: Row) -> &mut Self {
    self.header = header;
    self
  }

  /// Add a row to the table.
  pub fn push(&mut self, row: Row) -> &mut Self {
    self.rows.push(row);
    self
  }
}

impl Display for Table {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let lengths: Vec<_> = self.max_len_by_col().collect();

    // header first
    self.header.display_with_len(&lengths, f)?;

    // delimiter
    let delim = vec![Cell::new("").fill('-'); self.header.cells.len()];
    Row { cells: delim }.display_with_len(&lengths, f)?;

    // then rows
    for row in &self.rows {
      row.display_with_len(&lengths, f)?;
    }

    Ok(())
  }
}

/// A cell inside a table.
///
/// It can be used as header or row values.
#[derive(Clone, Debug)]
pub struct Cell {
  content: ColoredString,
  filling: char,
}

impl Cell {
  pub fn new(content: impl Into<ColoredString>) -> Self {
    Self {
      content: content.into(),
      filling: ' ',
    }
  }

  pub fn fill(mut self, filling: char) -> Self {
    self.filling = filling;
    self
  }

  /// Display the cell with specific length padding.
  fn display_with_len(&self, len: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let fill = self.filling;
    // BUG: here, the problem is that we are displaying something that has a style applied to it; I don’t get why it’s
    // fucked up like that, maybe we should be using something more “display” like?
    let padding_len = len - self.content.graphemes(true).count();
    let content = &self.content;

    write!(f, "{content}",)?;

    for _ in 0..padding_len {
      write!(f, "{fill}")?;
    }

    Ok(())
  }
}

impl<'a> From<&'a str> for Cell {
  fn from(s: &'a str) -> Self {
    Self::new(s)
  }
}

/// A row of cells.
#[derive(Debug, Default)]
pub struct Row {
  cells: Vec<Cell>,
}

impl Row {
  pub fn push(&mut self, cell: impl Into<Cell>) -> &mut Self {
    self.cells.push(cell.into());
    self
  }

  fn display_with_len(&self, lens: &[usize], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.cells.is_empty() {
      return Ok(());
    }

    // first cell first
    self.cells[0].display_with_len(lens[0], f)?;

    // then the rest
    for (cell, &len) in self.cells[1..].iter().zip(&lens[1..]) {
      f.write_str(" | ")?;
      cell.display_with_len(len, f)?;
    }

    f.write_str("\n")
  }
}

#[derive(Debug, Default)]
pub struct RowBuilder {
  row: Row,
}

impl RowBuilder {
  pub fn push(mut self, cell: impl Into<Cell>) -> Self {
    self.row.push(cell);
    self
  }

  pub fn build(self) -> Row {
    self.row
  }
}
