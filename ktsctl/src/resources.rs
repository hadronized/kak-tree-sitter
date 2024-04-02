//! Resources directories, files and various related functions.

use std::path::{Path, PathBuf};

use kak_tree_sitter_config::{source::Source, LanguageConfig};

use crate::error::HellNo;

/// Resources view (paths, dirs, etc.).
#[derive(Debug)]
pub struct Resources {
  runtime_dir: PathBuf,
  data_dir: PathBuf,
}

impl Resources {
  /// Ensure paths / directories exist and generate a [`Resources`] object.
  pub fn new() -> Result<Self, HellNo> {
    let runtime_dir = dirs::runtime_dir()
      .or_else(|| std::env::var("TMPDIR").map(PathBuf::from).ok())
      .ok_or_else(|| HellNo::NoRuntimeDir)?
      .join("ktsctl");

    let data_dir = dirs::data_dir()
      .ok_or_else(|| HellNo::NoDataDir)?
      .join("kak-tree-sitter");

    Ok(Self {
      runtime_dir,
      data_dir,
    })
  }

  /// Data directory (a.k.a. install directory); where `ktsctl` moves resources.
  pub fn data_dir(&self) -> &Path {
    &self.data_dir
  }

  /// Source directory for a given URL (get a stable path for a given URL to work in).
  /// This function only supports http:// and https:// links. For instance,
  /// https://github.com/hadronized/kak-tree-sitter will get a directory created in the form of:
  ///
  ///   <resources-root>/sources/github.com/hadronized/kak-tree-sitter
  ///
  /// Note: this function doesn’t perform any cleaning of the input URL, and it doesn’t perform any IO.
  pub fn sources_dir(&self, url: &str) -> PathBuf {
    let url_dir = PathBuf::from(
      url
        .trim_start_matches("http")
        .trim_start_matches('s')
        .trim_start_matches("://"),
    );

    self.runtime_dir.join("sources").join(url_dir)
  }

  /// Build directory for building source code.
  pub fn lang_build_dir(&self, path: &Path, src: &Path) -> PathBuf {
    self.runtime_dir.join(format!(
      "{path}/{src}/build",
      path = path.display(),
      src = src.display()
    ))
  }

  /// Get a grammar path from config.
  pub fn grammar_path_from_config(&self, lang: &str, config: &LanguageConfig) -> PathBuf {
    match config.grammar.source {
      Source::Local { ref path } => path.clone(),
      Source::Git { ref pin, .. } => self.grammar_pin_path(lang, pin),
    }
  }

  /// Directory for language grammar and a pin.
  pub fn grammar_pin_path(&self, lang: &str, pin: &str) -> PathBuf {
    self.data_dir.join(format!("grammars/{lang}/{pin}.so"))
  }

  /// Get the queries directory from a language config.
  pub fn queries_dir_from_config(&self, lang: &str, config: &LanguageConfig) -> Option<PathBuf> {
    let path = match config.queries.source.as_ref()? {
      Source::Local { ref path } => path.clone(),
      Source::Git { ref pin, .. } => self.queries_pin_dir(lang, pin),
    };

    Some(path)
  }

  /// Directory for language queries.
  pub fn queries_pin_dir(&self, lang: &str, pin: &str) -> PathBuf {
    self.data_dir.join(format!("queries/{lang}/{pin}"))
  }

  /// Check if a grammar was compiled and installed for a given language and pin.
  pub fn grammar_exists(&self, lang: &str, pin: &str) -> bool {
    let path = self.grammar_pin_path(lang, pin);
    matches!(path.try_exists(), Ok(true))
  }

  /// Check if queries exist for a given language and pin.
  ///
  /// Note: this function doesn’t check for the existence of specific queries; only that a directory exists for them.
  pub fn queries_exist(&self, lang: &str, pin: &str) -> bool {
    let path = self.queries_pin_dir(lang, pin);
    matches!(path.try_exists(), Ok(true))
  }

  /// Grammar directory containing all grammars for a given language.
  pub fn grammars_dir(&self, lang: &str) -> PathBuf {
    self.data_dir.join(format!("grammars/{lang}"))
  }

  /// Queries directory containing all queries for a given language.
  pub fn queries_dir(&self, lang: &str) -> PathBuf {
    self.data_dir.join(format!("queries/{lang}"))
  }
}
