//! Text-object support.

use std::fmt::Display;

/// A text-object type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Type {
  pub pattern: Pattern,
  pub level: Level,
}

impl Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_query_name())
  }
}

impl Type {
  /// Return the query name representation of this text-object type.
  pub fn as_query_name(&self) -> &'static str {
    match (self.pattern, self.level) {
      (Pattern::Function, Level::Inside) => "function.inside",
      (Pattern::Function, Level::Around) => "function.around",
      (Pattern::Parameter, Level::Inside) => "parameter.inside",
      (Pattern::Parameter, Level::Around) => "parameter.around",
      (Pattern::Class, Level::Inside) => "class.inside",
      (Pattern::Class, Level::Around) => "class.around",
      (Pattern::Comment, Level::Inside) => "comment.inside",
      (Pattern::Comment, Level::Around) => "comment.around",
      (Pattern::Test, Level::Inside) => "test.inside",
      (Pattern::Test, Level::Around) => "test.around",
    }
  }
}

/// Text-object flavor as currently supported.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Pattern {
  Function,
  Parameter,
  Class,
  Comment,
  Test,
}

/// Level of the text-object.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Level {
  Inside,
  Around,
}
