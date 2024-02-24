//! Text-object support.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

///
/// Text-objects can be manipulated in two different ways:
///
/// - In object mode, to expand selections or replace them.
/// - To shrink selections via selecting or splitting, as in `s`, `S`, etc.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationMode {
  /// Find the next text-object.
  Next,

  /// Find the previous text-object.
  Prev,

  /// Select the enclosing text-object (inside).
  Inside,

  /// Select the enclosing text-object (around)
  Around,

  /// Select text-objects inside the selection.
  Select,

  /// Split with text-objects inside the selection.
  Split,
}
