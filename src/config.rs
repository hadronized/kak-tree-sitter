//! Configuration for both the daemon and client.

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
  pub grammars: GrammarsConfig,
  pub queries: QueriesConfig,
  pub highlight: HighlightConfig,
}

impl Config {
  /// Load the config from the default user location (XDG).
  pub fn load_from_xdg() -> Config {
    dirs::config_dir()
      .and_then(|dir| {
        let path = dir.join("kak-tree-sitter/config.toml");
        let content = fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
      })
      .unwrap_or_default()
  }
}

/// Configuration for highlighting.
///
/// Highlighting configuration consists of a default set of settings, and per-language overrides.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HighlightConfig {
  /// List of highlight names to detect in grammars.
  pub hl_names: Vec<String>,
}

impl Default for HighlightConfig {
  fn default() -> Self {
    let hl_names = [
      "attribute",
      "constant",
      "constructor",
      "function.builtin",
      "function",
      "function.macro",
      "function.method",
      "keyword",
      "label",
      "operator",
      "property",
      "punctuation",
      "punctuation.bracket",
      "punctuation.delimiter",
      "string",
      "string.special",
      "tag",
      "type",
      "type.builtin",
      "variable",
      "variable.builtin",
      "variable.parameter",
    ]
    .into_iter()
    .map(|n| n.to_owned())
    .collect();
    HighlightConfig { hl_names }
  }
}

/// Tree-sitter queries configuration.
///
/// We currently support three kind of queries:
///
/// - Highlights.
/// - Injections.
/// - Locals.
/// - Text-objects.
///
/// All of those properties are set per-language and thus require runtime files. Each language has a specific directory
/// (same name as the language), which must contain `highlights.scm`, `injections.scm`, `locals.scm` and
/// `text-objects.scm`. The absence of one of those files disable the linked feature.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct QueriesConfig {
  pub path: PathBuf,
}

impl Default for QueriesConfig {
  fn default() -> Self {
    QueriesConfig {
      path: dirs::data_dir()
        .map(|dir| dir.join("kak-tree-sitter/queries"))
        .unwrap(), // FIXME: yikes?
    }
  }
}

/// Tree-sitter grammars configuration.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct GrammarsConfig {
  pub path: PathBuf,
}

impl Default for GrammarsConfig {
  fn default() -> Self {
    GrammarsConfig {
      path: dirs::data_dir()
        .map(|dir| dir.join("kak-tree-sitter/grammars"))
        .unwrap(), // FIXME: yikes again?
    }
  }
}
