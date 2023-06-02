//! Configuration for both the daemon and client.

use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
  #[serde(flatten)]
  pub languages: LanguagesConfig,
}

impl Config {
  /// Load the config from the default user location (XDG), and if not found, default to the system location (FHS).
  /// If not found, a default, empty configuration is used.
  pub fn load() -> Config {
    dirs::config_dir()
      .and_then(|dir| {
        let path = dir.join("kak-tree-sitter/config.toml");
        let content = fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
      })
      .or_else(|| {
        let path = Path::new(option_env!("SHARE_PREFIX").unwrap_or("/usr/local/share"))
          .join("kak-tree-sitter/config.toml");
        let content = fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
      })
      .unwrap_or_default()
  }
}

/// Languages configuration.
///
/// It is possible to set the URI and path where to fetch grammars, as well as queries.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LanguagesConfig {
  pub language: HashMap<String, LanguageConfig>,
}

impl LanguagesConfig {
  /// Get the configuration for `lang`.
  ///
  /// If there is no specific configuration for `lang`, this function tries to get the overridden default configuration.
  /// If there is none, the default configuration is returned.
  pub fn get_lang_conf(&self, lang: impl AsRef<str>) -> LanguageConfig {
    self
      .language
      .get(lang.as_ref())
      .or_else(|| self.language.get("default"))
      .cloned()
      .unwrap_or_default()
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
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LanguageConfig {
  pub grammar: LanguageGrammarConfig,
  pub queries: LanguageQueriesConfig,
  pub highlight: LanguageHighlightConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LanguageGrammarConfig {
  /// A format string to form the final URI to fetch the language grammar from.
  ///
  /// The language is inserted via the `{lang}` placeholder.
  pub uri_fmt: String,

  /// Path to go to where to find the grammar source.
  ///
  /// This is not the src/ directory; it is its parent.
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

impl Default for LanguageGrammarConfig {
  fn default() -> Self {
    Self {
      uri_fmt: "https://github.com/tree-sitter/tree-sitter-{lang}".to_owned(),
      path: ".".into(),
      compile: "cc".to_owned(),
      compile_args: vec![
        "-c".to_owned(),
        "-fpic".to_owned(),
        "../src/scanner.c".to_owned(),
        "../src/parser.c".to_owned(),
        "-I".to_owned(),
        "../src".to_owned(),
      ],
      compile_flags: vec!["-O3".to_owned()],
      link: "cc".to_owned(),
      link_args: vec![
        "-shared".to_owned(),
        "-fpic".to_owned(),
        "scanner.o".to_owned(),
        "parser.o".to_owned(),
        "-o".to_owned(),
        "{lang}.so".to_owned(),
      ],
      link_flags: vec!["-O3".to_owned()],
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LanguageQueriesConfig {
  /// A format string to form the final URI to fetch the language queries from.
  ///
  /// The language is inserted via the `{lang}` placeholder.
  pub uri_fmt: String,

  /// Path to go to where to find the queries directory.
  pub path: PathBuf,
}

impl Default for LanguageQueriesConfig {
  fn default() -> Self {
    LanguageQueriesConfig {
      uri_fmt: "https://github.com/tree-sitter/tree-sitter-{lang}".to_owned(),
      path: PathBuf::from("queries"),
    }
  }
}

/// Configuration for highlighting.
///
/// Highlighting configuration consists of a default set of settings, and per-language overrides.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LanguageHighlightConfig {
  /// List of highlight names to detect in grammars.
  pub hl_names: Vec<String>,
}

impl Default for LanguageHighlightConfig {
  fn default() -> Self {
    let hl_names = [
      "attribute",
      "comment",
      "conceal",
      "constant",
      "constructor",
      "function.builtin",
      "function",
      "function.macro",
      "function.method",
      "keyword",
      "keyword.control.conditional",
      "keyword.function",
      "label",
      "namespace",
      "operator",
      "property",
      "punctuation",
      "punctuation.bracket",
      "punctuation.delimiter",
      "punctuation.special",
      "special",
      "spell",
      "string",
      "string.escape",
      "string.special",
      "tag",
      "text",
      "text.literal",
      "text.reference",
      "text.title",
      "text.quote",
      "text.uri",
      "type",
      "type.builtin",
      "variable",
      "variable.builtin",
      "variable.other.member",
      "variable.parameter",
    ]
    .into_iter()
    .map(|n| n.to_owned())
    .collect();
    LanguageHighlightConfig { hl_names }
  }
}
