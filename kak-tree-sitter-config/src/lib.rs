//! Configuration for both the daemon and client.

use std::{collections::HashMap, fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
  #[error("no configuration directory known for your system; please adjust XDG_CONFIG_HOME")]
  NoConfigDir,

  #[error("cannot read configuration: {err}")]
  CannotReadConfig { err: io::Error },

  #[error("cannot parse configuration: {err}")]
  CannotParseConfig { err: String },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
  #[serde(flatten)]
  pub languages: LanguagesConfig,
}

impl Config {
  /// Load the config from the default user location (XDG).
  pub fn load_from_xdg() -> Result<Config, ConfigError> {
    let dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
    let path = dir.join("kak-tree-sitter/config.toml");
    let content = fs::read_to_string(path).map_err(|err| ConfigError::CannotReadConfig { err })?;
    toml::from_str(&content).map_err(|err| ConfigError::CannotParseConfig {
      err: err.to_string(),
    })
  }
}

/// Languages configuration.
///
/// It is possible to set the URI and path where to fetch grammars, as well as queries.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LanguagesConfig {
  pub language: HashMap<String, LanguageConfig>,
}

impl LanguagesConfig {
  /// Get the configuration for `lang`.
  pub fn get_lang_conf(&self, lang: impl AsRef<str>) -> Option<&LanguageConfig> {
    self.language.get(lang.as_ref())
  }

  /// Get the directory where all grammars live in.
  pub fn get_grammars_dir(&self) -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("kak-tree-sitter/grammars"))
  }

  /// Get the grammar path for a given language.
  pub fn get_grammar_path(&self, lang: impl AsRef<str>) -> Option<PathBuf> {
    let lang = lang.as_ref();
    dirs::data_dir().map(|dir| dir.join(format!("kak-tree-sitter/grammars/{lang}.so")))
  }

  /// Get the queries directory for a given language.
  pub fn get_queries_dir(&self, lang: impl AsRef<str>) -> Option<PathBuf> {
    let lang = lang.as_ref();
    dirs::data_dir().map(|dir| dir.join(format!("kak-tree-sitter/queries/{lang}")))
  }
}

/// Specific language configuration.
///
/// Not providing one will default to the default language configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LanguageConfig {
  pub grammar: LanguageGrammarConfig,
  pub queries: LanguageQueriesConfig,
  pub highlight: LanguageHighlightConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LanguageGrammarConfig {
  /// URL to fetch the language grammar from.
  pub url: String,

  /// Path to find the grammar source.
  pub path: PathBuf,

  /// Compile command to run to compile the grammar.
  ///
  /// Should always be `cc` but who knows.
  pub compile: String,

  /// Compiler arguments.
  ///
  /// Wherever the language must appear, you can use `{lang}` as placeholder.
  pub compile_args: Vec<String>,

  /// Compiler extra arguments.
  ///
  /// Should be used to pass optimization and debug flags, mainly.
  pub compile_flags: Vec<String>,

  /// Link command to run to link the grammar.
  ///
  /// Should always be `cc`, but, still, who knows.
  pub link: String,

  /// Linker arguments.
  ///
  /// Wherever the language must appear, you can use `{lang} as placeholder.
  pub link_args: Vec<String>,

  /// Linker extra arguments.
  ///
  /// Should be used to pass optimization and debug flags, mainly.
  pub link_flags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LanguageQueriesConfig {
  /// URL to fetch the language queries from.
  ///
  /// The language is inserted via the `{lang}` placeholder.
  ///
  /// If set to [`None`], the URL used will be the same as the one for the grammar and no fetch will be done (the
  /// grammar is required).
  pub url: Option<String>,

  /// Path to go to where to find the queries directory.
  pub path: PathBuf,
}

/// Configuration for highlighting.
///
/// Highlighting consists in mapping between highlight groups, such as `keyword.control.conditional`, and a Kakoune face
/// definition. Each highlight group is then converted to `set-face global ts_<name> <face definition>`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LanguageHighlightConfig {
  /// List of highlight names to detect in grammars.
  pub groups: HashMap<String, String>,
}
