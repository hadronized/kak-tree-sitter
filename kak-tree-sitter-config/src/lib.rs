//! Configuration for both the daemon and client.

use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
  pub grammars: GrammarsConfig,
  pub queries: QueriesConfig,
  pub highlight: HighlightConfig,
  #[serde(flatten)]
  pub languages: LanguagesConfig,
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HighlightConfig {
  /// List of highlight names to detect in grammars.
  pub hl_names: Vec<String>,
}

impl Default for HighlightConfig {
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
      "variable.other_member",
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
#[derive(Clone, Debug, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct GrammarsConfig {
  pub grammar_libs_dir: PathBuf,
}

impl Default for GrammarsConfig {
  fn default() -> Self {
    GrammarsConfig {
      grammar_libs_dir: dirs::data_dir()
        .map(|dir| dir.join("kak-tree-sitter/grammars"))
        .unwrap(), // FIXME: yikes again?
    }
  }
}

/// Languages configuration.
///
/// It is possible to set the URI and path where to fetch grammars, as well as queries.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LanguagesConfig {
  pub language: HashMap<String, LanguageConfig>,
}

impl Default for LanguagesConfig {
  fn default() -> Self {
    Self {
      language: HashMap::new(),
    }
  }
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
}

/// Specific language configuration.
///
/// Not providing one will default to the default language configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LanguageConfig {
  pub grammar: LanguageGrammarConfig,
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

  /// Link command to run to link the grammar.
  ///
  /// Should always be `cc`, but, still, who knows.
  pub link: String,

  /// Linker arguments.
  ///
  /// Wherever the language must appear, you can use `{lang} as placeholder.
  pub link_args: Vec<String>,
}

impl Default for LanguageGrammarConfig {
  fn default() -> Self {
    Self {
      uri_fmt: "https://github.com/tree-sitter/tree-sitter-{lang}".to_owned(),
      path: ".".into(),
      compile: "cc".to_owned(),
      compile_args: vec![
        "-c".to_owned(),
        "-O3".to_owned(),
        "-fpic".to_owned(),
        "../src/scanner.c".to_owned(),
        "../src/parser.c".to_owned(),
        "-I".to_owned(),
        "../src".to_owned(),
      ],
      link: "cc".to_owned(),
      link_args: vec![
        "-shared".to_owned(),
        "-O3".to_owned(),
        "-fpic".to_owned(),
        "scanner.o".to_owned(),
        "parser.o".to_owned(),
        "-o".to_owned(),
        "{lang}.so".to_owned(),
      ],
    }
  }
}
