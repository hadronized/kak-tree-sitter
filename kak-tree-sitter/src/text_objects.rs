//! Text-object support.
//!
//! Requests use strings like `"function.inside"`, `"function.around"`, etc. to target specific capture groups. We do
//! not provide a type for this, as the patterns are free and varies on the languages / grammars.
//!
//! However, operation modes are fixed and represent what to do with the matched parts of the buffer. For instance,
//! [`OperationMode::Next`] will move each selection to the next matched capture group, etc.

use serde::{Deserialize, Serialize};

/// Text-objects can be manipulated in two different ways:
///
/// - In object mode, to expand selections or replace them.
/// - To shrink selections via selecting or splitting, as in `s`, `S`, etc.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationMode {
  /// Search for the next text-object.
  ///
  /// Similar to `/`.
  SearchNext,

  /// Search for the previous text-object.
  ///
  /// Similar to `<a-/>`.
  SearchPrev,

  /// Search-extend for the next text-object.
  ///
  /// Similar to `?`.
  SearchExtendNext,

  /// Search-extend for the previous text-object.
  ///
  /// Similar to `<a-?>`.
  SearchExtendPrev,

  /// Find the next text-object.
  ///
  /// Similar to `f`.
  FindNext,

  /// Find the previous text-object.
  ///
  /// Similar to `<a-f>`.
  FindPrev,
}
