//! Configuration for both the daemon and client.

use std::{
  collections::{HashMap, HashSet},
  fs, io,
  path::PathBuf,
};

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
  pub highlight: HighlightConfig,

  #[serde(flatten)]
  pub languages: LanguagesConfig,
}

impl Config {
  pub fn load() -> Result<Config, ConfigError> {
    Self::load_from_xdg().or(Self::load_builtin())
  }

  /// Load the config from the default user location (XDG).
  fn load_from_xdg() -> Result<Config, ConfigError> {
    let dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
    let path = dir.join("kak-tree-sitter/config.toml");
    let content = fs::read_to_string(path).map_err(|err| ConfigError::CannotReadConfig { err })?;

    toml::from_str(&content).map_err(|err| ConfigError::CannotParseConfig {
      err: err.to_string(),
    })
  }

  /// Load the built-in config which may be shipped with the binary.
  fn load_builtin() -> Result<Config, ConfigError> {
    let exec = std::env::current_exe().map_err(|err| ConfigError::CannotReadConfig { err })?;
    let root = exec
      .parent()
      .and_then(|bin| bin.parent())
      .ok_or(ConfigError::NoConfigDir)?;
    let path = root.join("share/kak-tree-sitter/config.toml");
    let content = fs::read_to_string(path).map_err(|err| ConfigError::CannotReadConfig { err })?;

    toml::from_str(&content).map_err(|err| ConfigError::CannotParseConfig {
      err: err.to_string(),
    })
  }
}

/// Highlight configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HighlightConfig {
  pub groups: HashSet<String>,
}

/// Languages configuration.
///
/// It is possible to set the URI and path where to fetch grammars, as well as queries.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LanguagesConfig {
  pub language: HashMap<String, LanguageConfig>,
}

impl LanguagesConfig {
  /// Get the directory with built-in grammars and queries relative to the binary.
  fn get_builtin_dir() -> Option<PathBuf> {
    std::env::current_exe()
      .ok()?
      .parent()?
      .parent()
      .map(|dir| dir.join("share/kak-tree-sitter"))
  }

  fn fallback(from_xdg: Option<PathBuf>, builtin: Option<PathBuf>) -> Option<PathBuf> {
    match from_xdg {
      Some(path) => match path.try_exists() {
        Ok(true) => Some(path),
        Ok(false) => builtin,
        Err(_) => builtin,
      },
      None => builtin,
    }
  }

  /// Get the configuration for `lang`.
  pub fn get_lang_conf(&self, lang: impl AsRef<str>) -> Option<&LanguageConfig> {
    self.language.get(lang.as_ref())
  }

  /// Get the directory where all grammars live in.
  pub fn get_grammars_dir() -> Option<PathBuf> {
    let builtin = Self::get_builtin_dir().map(|dir| dir.join("grammars"));
    let from_xdg = dirs::data_dir().map(|dir| dir.join("kak-tree-sitter/grammars"));
    Self::fallback(from_xdg, builtin)
  }

  /// Get the grammar path for a given language.
  pub fn get_grammar_path(lang: impl AsRef<str>) -> Option<PathBuf> {
    let lang = lang.as_ref();
    let builtin = Self::get_grammars_dir().map(|dir| dir.join(format!("{lang}.so")));
    let from_xdg =
      dirs::data_dir().map(|dir| dir.join(format!("kak-tree-sitter/grammars/{lang}.so")));
    Self::fallback(from_xdg, builtin)
  }

  /// Get the queries directory for a given language.
  pub fn get_queries_dir(lang: impl AsRef<str>) -> Option<PathBuf> {
    let lang = lang.as_ref();
    let builtin = Self::get_builtin_dir().map(|dir| dir.join(format!("queries/{lang}")));
    let from_xdg = dirs::data_dir().map(|dir| dir.join(format!("kak-tree-sitter/queries/{lang}")));
    Self::fallback(from_xdg, builtin)
  }
}

/// Specific language configuration.
///
/// Not providing one will default to the default language configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LanguageConfig {
  pub grammar: LanguageGrammarConfig,
  pub queries: LanguageQueriesConfig,
  #[serde(default)]
  pub remove_default_highlighter: RemoveDefaultHighlighter,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct RemoveDefaultHighlighter(pub bool);

impl Default for RemoveDefaultHighlighter {
  fn default() -> Self {
    RemoveDefaultHighlighter(true)
  }
}

impl RemoveDefaultHighlighter {
  pub fn to_bool(self) -> bool {
    self.0
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LanguageGrammarConfig {
  /// URL to fetch the language grammar from.
  pub url: String,

  /// Pin to use. Can be a commit, a branch, etc.
  pub pin: Option<String>,

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

  /// Pin to use. Can be a commit, a branch, etc.
  pub pin: Option<String>,

  /// Path to go to where to find the queries directory.
  pub path: PathBuf,
}
