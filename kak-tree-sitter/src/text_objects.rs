//! Text-object support.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Level of the text-object.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Level {
  Inside,
  Around,
}

impl Display for Level {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      Level::Inside => "inside",
      Level::Around => "around",
    };

    f.write_str(s)
  }
}

/// Operation mode.
///
/// Text-objects can be manipulated in two different ways:
///
/// - In object mode, to expand selections or replace them.
/// - To shrink selections via selecting or splitting, as in `s`, `S`, etc.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum OperationMode {
  Object,
  Shrink,
}
