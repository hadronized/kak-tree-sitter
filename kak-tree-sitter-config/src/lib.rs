//! Configuration for both the daemon and client.

pub mod error;
pub mod source;

use std::{
  collections::{HashMap, HashSet},
  fs, io,
  path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use source::{Source, UserSource};

/// Configuration object used in the server and controller.
///
/// User configuration being opt-in for every option, a different type is used, [`UserConfig`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
  pub highlight: HighlightConfig,

  #[serde(flatten)]
  pub languages: LanguagesConfig,
}

impl Config {
  /// Load the configuration from a given path.
  pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|err| ConfigError::CannotReadConfig {
      path: path.to_owned(),
      err,
    })?;

    toml::from_str(&content).map_err(|err| ConfigError::CannotParseConfig {
      err: err.to_string(),
    })
  }

  /// Default configuration using the `default-config.toml` file.
  const DEFAULT_CONFIG_CONTENT: &'static str = include_str!("../../default-config.toml");

  pub fn load_default_config() -> Result<Self, ConfigError> {
    log::debug!("loading default configuration");

    toml::from_str(Self::DEFAULT_CONFIG_CONTENT).map_err(|err| ConfigError::CannotParseConfig {
      err: err.to_string(),
    })
  }

  /// Load the default configuration, the user configuration, and merge both.
  pub fn load_default_user() -> Result<Self, ConfigError> {
    let mut config = Self::load_default_config()?;
    match UserConfig::load_from_xdg() {
      Ok(user_config) => {
        config.merge_user_config(user_config)?;
      }

      Err(err) => {
        log::warn!("cannot load user config: {err}");
      }
    }

    Ok(config)
  }

  /// Merge the config with a user-provided one.
  pub fn merge_user_config(&mut self, user_config: UserConfig) -> Result<(), ConfigError> {
    if let Some(user_highlight) = user_config.highlight {
      self.highlight.merge_user_config(user_highlight);
    }

    if let Some(language) = user_config.language {
      self.languages.merge_user_config(language)?;
    }

    Ok(())
  }
}

/// Highlight configuration.
///
/// This is a set of capture groups that can be found in various queries.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HighlightConfig {
  pub groups: HashSet<String>,
}

impl HighlightConfig {
  fn merge_user_config(&mut self, user_config: UserHighlightConfig) {
    self.groups.extend(user_config.groups);
  }
}

/// Languages configuration.
///
/// This is akin to a map from the language name and the language config ([`LanguageConfig`]).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LanguagesConfig {
  pub language: HashMap<String, LanguageConfig>,
}

impl LanguagesConfig {
  fn merge_user_config(
    &mut self,
    user_config: HashMap<String, UserLanguageConfig>,
  ) -> Result<(), ConfigError> {
    for (lang, user_config) in user_config {
      if let Some(config) = self.language.get_mut(&lang) {
        // if we already have a config, everything is optional so we can merge
        config.merge_user_config(user_config)?;
      } else {
        // if we do not have a config, we take it from the user configuration, which can fail
        self
          .language
          .insert(lang, LanguageConfig::try_from(user_config)?);
      }
    }

    Ok(())
  }

  /// Get the configuration for `lang`.
  pub fn get_lang_conf(&self, lang: impl AsRef<str>) -> Option<&LanguageConfig> {
    self.language.get(lang.as_ref())
  }

  /// Get the directory where all grammars live in.
  pub fn get_grammars_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("kak-tree-sitter/grammars"))
  }

  /// Get the grammar path for a given language.
  pub fn get_grammar_path(lang_config: &LanguageConfig, lang: impl AsRef<str>) -> Option<PathBuf> {
    match lang_config.grammar.source {
      Source::Local { ref path } => Some(path.clone()),

      Source::Git { ref pin, .. } => {
        let lang = lang.as_ref();
        dirs::data_dir().map(|dir| dir.join(format!("kak-tree-sitter/grammars/{lang}/{pin}.so")))
      }
    }
  }

  /// Get the queries directory for a given language.
  pub fn get_queries_dir(lang_config: &LanguageConfig, lang: impl AsRef<str>) -> Option<PathBuf> {
    match lang_config.queries.source {
      Some(Source::Local { ref path }) => Some(path.clone()),

      Some(Source::Git { ref pin, .. }) => {
        let lang = lang.as_ref();
        dirs::data_dir().map(|dir| dir.join(format!("kak-tree-sitter/queries/{lang}/{pin}")))
      }

      None => {
        // FIXME: not sure this is wanted?
        None
      }
    }
  }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RemoveDefaultHighlighter(pub bool);

impl Default for RemoveDefaultHighlighter {
  fn default() -> Self {
    Self(true)
  }
}

impl From<bool> for RemoveDefaultHighlighter {
  fn from(value: bool) -> Self {
    Self(value)
  }
}

impl From<RemoveDefaultHighlighter> for bool {
  fn from(RemoveDefaultHighlighter(value): RemoveDefaultHighlighter) -> Self {
    value
  }
}

/// Specific language configuration.
///
/// It is possible to configure the grammar and queries part of a language, as well as some specific Kakoune options.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LanguageConfig {
  pub grammar: LanguageGrammarConfig,
  pub queries: LanguageQueriesConfig,

  #[serde(default)]
  pub remove_default_highlighter: RemoveDefaultHighlighter,
}

impl LanguageConfig {
  fn merge_user_config(&mut self, user_config: UserLanguageConfig) -> Result<(), ConfigError> {
    if let Some(user_grammar) = user_config.grammar {
      self.grammar.merge_user_config(user_grammar)?;
    }
    if let Some(user_queries) = user_config.queries {
      self.queries.merge_user_config(user_queries)?;
    }

    self.remove_default_highlighter = user_config
      .remove_default_highlighter
      .unwrap_or(self.remove_default_highlighter.0)
      .into();

    Ok(())
  }
}

impl TryFrom<UserLanguageConfig> for LanguageConfig {
  type Error = ConfigError;

  fn try_from(user_config: UserLanguageConfig) -> Result<Self, Self::Error> {
    let Some(grammar) = user_config.grammar else {
      return Err(ConfigError::missing_opt("grammar"));
    };
    let Some(queries) = user_config.queries else {
      return Err(ConfigError::missing_opt("queries"));
    };

    Ok(Self {
      grammar: LanguageGrammarConfig::try_from(grammar)?,
      queries: LanguageQueriesConfig::try_from(queries)?,
      remove_default_highlighter: user_config
        .remove_default_highlighter
        .unwrap_or(true)
        .into(),
    })
  }
}

/// Grammar configuration.
///
/// Most of the options are used by the controller only.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LanguageGrammarConfig {
  /// Source from where to get the grammar.
  pub source: Source,

  /// Path to find the grammar source inside the downloaded content.
  pub path: PathBuf,

  /// Compile command to run to compile the grammar.
  ///
  /// Should always be `cc` but who knows.
  pub compile: String,

  /// Compiler arguments.
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
  pub link_args: Vec<String>,

  /// Linker extra arguments.
  ///
  /// Should be used to pass optimization and debug flags, mainly.
  pub link_flags: Vec<String>,
}

impl LanguageGrammarConfig {
  fn merge_user_config(
    &mut self,
    user_config: UserLanguageGrammarConfig,
  ) -> Result<(), ConfigError> {
    if let Some(source) = user_config.source {
      self.source.merge_user_config(source);
    }

    if let Some(path) = user_config.path {
      self.path = path;
    }

    if let Some(compile) = user_config.compile {
      self.compile = compile;
    }

    if let Some(compile_args) = user_config.compile_args {
      self.compile_args = compile_args;
    }

    if let Some(compile_flags) = user_config.compile_flags {
      self.compile_flags = compile_flags;
    }

    if let Some(link) = user_config.link {
      self.link = link;
    }

    if let Some(link_args) = user_config.link_args {
      self.link_args = link_args;
    }

    if let Some(link_flags) = user_config.link_flags {
      self.link_flags = link_flags;
    }

    Ok(())
  }
}

impl TryFrom<UserLanguageGrammarConfig> for LanguageGrammarConfig {
  type Error = ConfigError;

  fn try_from(user_config: UserLanguageGrammarConfig) -> Result<Self, Self::Error> {
    let Some(source) = user_config.source else {
      return Err(ConfigError::missing_opt("source"));
    };
    let Some(link_args) = user_config.link_args else {
      return Err(ConfigError::missing_opt("link_args"));
    };

    Ok(Self {
      source: Source::try_from(source)?,
      path: user_config.path.unwrap_or_else(|| PathBuf::from("src")),
      compile: user_config.compile.unwrap_or_else(|| "cc".to_owned()),
      compile_args: user_config.compile_args.unwrap_or_else(|| {
        vec![
          "-c".to_owned(),
          "-fpic".to_owned(),
          "../parser.c".to_owned(),
          "../scanner.c".to_owned(),
          "-I".to_owned(),
          "..".to_owned(),
        ]
      }),
      compile_flags: user_config
        .compile_flags
        .unwrap_or_else(|| vec!["-O3".to_owned()]),
      link: user_config.link.unwrap_or_else(|| "cc".to_owned()),
      link_args,
      link_flags: user_config
        .link_flags
        .unwrap_or_else(|| vec!["-O3".to_owned()]),
    })
  }
}

/// Queries configuration.
///
/// Most of the options are used by the controller only.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LanguageQueriesConfig {
  /// Source from where to get the queries.
  pub source: Option<Source>,

  /// Path to go to where to find the queries directory.
  pub path: PathBuf,
}

impl LanguageQueriesConfig {
  fn merge_user_config(
    &mut self,
    user_config: UserLanguageQueriesConfig,
  ) -> Result<(), ConfigError> {
    match (&mut self.source, user_config.source) {
      (_, None) => (),

      (None, Some(source)) => self.source = Some(Source::try_from(source)?),

      (Some(source), Some(user_source)) => {
        source.merge_user_config(user_source);
      }
    }

    if let Some(path) = user_config.path {
      self.path = path;
    }

    Ok(())
  }
}

impl TryFrom<UserLanguageQueriesConfig> for LanguageQueriesConfig {
  type Error = ConfigError;

  fn try_from(user_config: UserLanguageQueriesConfig) -> Result<Self, Self::Error> {
    let Some(path) = user_config.path else {
      return Err(ConfigError::missing_opt("path"));
    };

    let source = if let Some(source) = user_config.source {
      Some(Source::try_from(source)?)
    } else {
      None
    };

    Ok(Self { source, path })
  }
}

/// User version of configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserConfig {
  pub highlight: Option<UserHighlightConfig>,
  pub language: Option<HashMap<String, UserLanguageConfig>>,
}

impl UserConfig {
  /// Load the config from the default user location (XDG).
  pub fn load_from_xdg() -> Result<Self, ConfigError> {
    log::debug!("loading user configuration");

    let dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
    let path = dir.join("kak-tree-sitter/config.toml");
    Self::load(path)
  }

  /// Load the configuration from a given path.
  fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
    let path = path.as_ref();

    log::debug!("loading configuration at {path}", path = path.display());

    let content = fs::read_to_string(path).map_err(|err| ConfigError::CannotReadConfig {
      path: path.to_owned(),
      err,
    })?;

    toml::from_str(&content).map_err(|err| ConfigError::CannotParseConfig {
      err: err.to_string(),
    })
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserHighlightConfig {
  pub groups: HashSet<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserLanguageConfig {
  pub grammar: Option<UserLanguageGrammarConfig>,
  pub queries: Option<UserLanguageQueriesConfig>,
  pub remove_default_highlighter: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserLanguageGrammarConfig {
  pub source: Option<UserSource>,
  pub path: Option<PathBuf>,
  pub compile: Option<String>,
  pub compile_args: Option<Vec<String>>,
  pub compile_flags: Option<Vec<String>>,
  pub link: Option<String>,
  pub link_args: Option<Vec<String>>,
  pub link_flags: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserLanguageQueriesConfig {
  pub source: Option<UserSource>,
  pub path: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::{
    source::{Source, UserSource},
    Config, ConfigError, HighlightConfig, LanguageConfig, LanguageGrammarConfig,
    LanguageQueriesConfig, LanguagesConfig, UserConfig, UserLanguageConfig,
    UserLanguageGrammarConfig,
  };

  #[test]
  fn user_merge() {
    // we have a config and see that we can alter it by merging with a user config
    let main_config = Config {
      highlight: HighlightConfig {
        groups: ["foo".to_owned(), "bar".to_owned(), "zoo".to_owned()]
          .into_iter()
          .collect(),
      },
      languages: LanguagesConfig {
        language: [(
          "rust".to_owned(),
          LanguageConfig {
            grammar: LanguageGrammarConfig {
              source: Source::local("file://hello"),
              path: PathBuf::from("src"),
              compile: "".to_owned(),
              compile_args: Vec::default(),
              compile_flags: Vec::default(),
              link: "".to_owned(),
              link_args: Vec::default(),
              link_flags: Vec::default(),
            },
            queries: LanguageQueriesConfig {
              source: None,
              path: PathBuf::from("runtime/queries/rust"),
            },
            remove_default_highlighter: true.into(),
          },
        )]
        .into_iter()
        .collect(),
      },
    };

    // merging a default user config to a config shouldn’t change anything
    {
      let mut config = main_config.clone();
      let user_config = UserConfig::default();
      assert!(config.merge_user_config(user_config).is_ok());
      assert_eq!(main_config, config);
    }

    // deeply changing some config for Rust
    {
      let mut config = main_config.clone();
      let user_config = UserConfig {
        highlight: None,
        language: Some(
          [(
            "rust".to_owned(),
            UserLanguageConfig {
              grammar: Some(UserLanguageGrammarConfig {
                source: Some(UserSource::git("git_source".to_owned(), "pin".to_owned())),
                link_args: Some(vec!["link".to_owned(), "args".to_owned()]),
                ..Default::default()
              }),
              ..Default::default()
            },
          )]
          .into_iter()
          .collect(),
        ),
      };

      assert!(config.merge_user_config(user_config).is_ok());

      let prev_rust_config = main_config.languages.get_lang_conf("rust").unwrap();
      let new_rust_config = config.languages.get_lang_conf("rust").unwrap();

      assert_eq!(prev_rust_config.queries, new_rust_config.queries);

      assert_eq!(
        new_rust_config.grammar.source,
        Source::git("git_source", "pin".to_owned())
      );
      assert_eq!(
        new_rust_config.grammar.link_args,
        vec!["link".to_owned(), "args".to_owned()]
      );
    }
  }

  #[test]
  fn user_config() -> Result<(), ConfigError> {
    let toml = r#"[language.rust.grammar.source.git]
      pin = "foo""#;
    let config = toml::from_str::<UserConfig>(toml).unwrap();
    let source = config
      .language
      .as_ref()
      .unwrap()
      .get("rust")
      .as_ref()
      .unwrap()
      .grammar
      .as_ref()
      .unwrap()
      .source
      .as_ref()
      .unwrap();

    assert!(matches!(source, UserSource::Git { pin, .. } if pin.as_deref() == Some("foo")));

    Ok(())
  }
}
