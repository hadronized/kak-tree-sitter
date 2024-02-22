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
