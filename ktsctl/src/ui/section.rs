//! Typography based utilities.

use std::fmt::Display;

use colored::{ColoredString, Colorize};

use super::status_icon::StatusIcon;

#[derive(Debug)]
pub struct Section {
  name: String,
  fields: Vec<Field>,
}

impl Section {
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      fields: Vec::default(),
    }
  }

  pub fn push(&mut self, field: Field) -> &mut Self {
    self.fields.push(field);
    self
  }
}

impl Display for Section {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let name = format!("· {}", self.name).bold();
    writeln!(f, "{name}")?;

    for fld in &self.fields {
      writeln!(f, "  {fld}")?;
    }

    Ok(())
  }
}

impl Extend<Field> for Section {
  fn extend<T: IntoIterator<Item = Field>>(&mut self, iter: T) {
    self.fields.extend(iter);
  }
}

#[derive(Debug)]
pub struct SectionBuilder {
  section: Section,
}

impl SectionBuilder {
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      section: Section::new(name),
    }
  }

  pub fn push(mut self, field: Field) -> Self {
    self.section.push(field);
    self
  }

  pub fn build(self) -> Section {
    self.section
  }
}

/// Section field.
#[derive(Debug)]
pub enum Field {
  KeyValue {
    key: ColoredString,
    value: FieldValue,
    indent: usize,
  },

  StatusLine {
    status: StatusIcon,
    value: FieldValue,
    indent: usize,
  },
}

impl Field {
  pub fn kv(key: ColoredString, value: impl Into<FieldValue>) -> Self {
    Self::KeyValue {
      key,
      value: value.into(),
      indent: 0,
    }
  }

  pub fn status_line(status: StatusIcon, value: impl Into<FieldValue>) -> Self {
    Self::StatusLine {
      status,
      value: value.into(),
      indent: 0,
    }
  }

  pub fn indent(&mut self) -> &mut Self {
    match self {
      Self::KeyValue { indent, .. } => *indent += 1,
      Self::StatusLine { indent, .. } => *indent += 1,
    }

    self
  }
}

impl Display for Field {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Field::KeyValue { key, value, indent } => write!(
        f,
        "{indent}{key}{delim} {value}",
        delim = ":".black(),
        indent = " ".repeat(*indent * 2)
      ),
      Field::StatusLine {
        status,
        value,
        indent,
      } => write!(
        f,
        "{indent}{status} {value}",
        indent = " ".repeat(*indent * 2)
      ),
    }
  }
}

#[derive(Debug)]
pub enum FieldValue {
  String(ColoredString),
  List(Vec<ColoredString>),
}

impl FieldValue {
  pub fn list(v: impl Into<Vec<ColoredString>>) -> Self {
    Self::List(v.into())
  }
}

impl Display for FieldValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FieldValue::String(s) => s.fmt(f),
      FieldValue::List(ss) => {
        write!(f, "{} ", "[".black())?;

        if !ss.is_empty() {
          write!(f, "{}", ss[0])?;

          for s in &ss[1..] {
            write!(f, "{} {}", ",".black(), s)?;
          }
        }

        write!(f, " {}", "]".black())
      }
    }
  }
}

impl<T> From<T> for FieldValue
where
  T: Into<ColoredString>,
{
  fn from(value: T) -> Self {
    FieldValue::String(value.into())
  }
}
