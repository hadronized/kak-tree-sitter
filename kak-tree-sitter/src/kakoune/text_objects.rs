//! Text-object support.
//!
//! Requests use strings like `"function.inside"`, `"function.around"`, etc. to target specific capture groups. We do
//! not provide a type for this, as the patterns are free and varies on the languages / grammars.
//!
//! However, operation modes are fixed and represent what to do with the matched parts of the buffer. For instance,
//! [`OperationMode::Next`] will move each selection to the next matched capture group, etc.

use serde::{Deserialize, Serialize};

use crate::kakoune::selection::SelectMode;

/// Text-objects can be manipulated in two different ways:
///
/// - In object mode, to expand selections or replace them.
/// - To shrink selections via selecting or splitting, as in `s`, `S`, etc.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

  /// Extend onto the next text-object.
  ///
  /// Similar to `F`.
  ExtendNext,

  /// Extend onto the previous text-object.
  ///
  /// Similar to `<a-F>`.
  ExtendPrev,
  /// Object mode.
  ///
  /// This combines select mode with object flags.
  Object { mode: SelectMode, flags: String },
}

#[cfg(test)]
mod tests {
  use crate::kakoune::{selection::SelectMode, text_objects::OperationMode};

  #[test]
  fn deser() {
    assert_eq!(
      serde_json::from_str::<OperationMode>("\"search_next\"").unwrap(),
      OperationMode::SearchNext
    );
    assert_eq!(
      serde_json::from_str::<OperationMode>("\"search_prev\"").unwrap(),
      OperationMode::SearchPrev
    );
    assert_eq!(
      serde_json::from_str::<OperationMode>("\"search_extend_next\"").unwrap(),
      OperationMode::SearchExtendNext
    );
    assert_eq!(
      serde_json::from_str::<OperationMode>("\"search_extend_prev\"").unwrap(),
      OperationMode::SearchExtendPrev
    );
    assert_eq!(
      serde_json::from_str::<OperationMode>("\"find_next\"").unwrap(),
      OperationMode::FindNext
    );
    assert_eq!(
      serde_json::from_str::<OperationMode>("\"find_prev\"").unwrap(),
      OperationMode::FindPrev
    );
    assert_eq!(
      serde_json::from_str::<OperationMode>("\"extend_next\"").unwrap(),
      OperationMode::ExtendNext
    );
    assert_eq!(
      serde_json::from_str::<OperationMode>("\"extend_prev\"").unwrap(),
      OperationMode::ExtendPrev
    );

    assert_eq!(
      serde_json::from_str::<OperationMode>(
        r#"{ "object": { "mode": "replace", "flags": "to_begin|to_end|inner" }}"#
      )
      .unwrap(),
      OperationMode::Object {
        mode: SelectMode::Replace,
        flags: "to_begin|to_end|inner".to_owned()
      }
    );
  }
}
