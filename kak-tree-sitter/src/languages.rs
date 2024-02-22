//! Supported languages.
//!
//! Languages have different objects (grammars, queries, etc.) living at runtime and must be loaded beforehand.

use std::{collections::HashMap, path::Path};

use kak_tree_sitter_config::{Config, LanguagesConfig};
use libloading::Symbol;
use tree_sitter_highlight::HighlightConfiguration;

use crate::{error::OhNo, queries::Queries};

pub struct Language {
  pub hl_config: HighlightConfiguration,
  pub hl_names: Vec<String>,
  // whether we should remove the default highlighter when highlighting a buffer with this language
  pub remove_default_highlighter: bool,

  // NOTE: we need to keep that alive *probably*; better be safe than sorry
  _ts_lang: tree_sitter::Language,
  _ts_lib: libloading::Library,
}

impl Language {
  pub fn lang(&self) -> tree_sitter::Language {
    self.ts_lang
  }
  
  pub fn configure_hl_names(&mut self, hl_names: Vec<String>) {
     self.hl_config.configure(&hl_names);
     self.hl_names = hl_names;
  }
}

pub struct Languages {
  /// Map a `kts_lang` to the tree-sitter [`Language`] and its queries.
  pub langs: HashMap<String, Language>,
}

impl Languages {
  /// Load a grammar.
  fn load_grammar(
    lang: &str,
    path: &Path,
  ) -> Result<(libloading::Library, tree_sitter::Language), OhNo> {
    let lib = unsafe { libloading::Library::new(path) };
    let lib = lib.map_err(|err| OhNo::CannotLoadGrammar {
      lang: lang.to_owned(),
      err: err.to_string(),
    })?;
    let fn_sym = format!("tree_sitter_{}", lang.replace(['.', '-'], "_"));

    let sym: Result<Symbol<fn() -> tree_sitter::Language>, _> =
      unsafe { lib.get(fn_sym.as_bytes()) };
    let sym = sym.map_err(|err| OhNo::CannotLoadGrammar {
      lang: lang.to_owned(),
      err: format!("cannot find language: {err}"),
    })?;
    let sym = sym();

    Ok((lib, sym))
  }

  /// Load languages.
  ///
  /// This function will scan the directory and extract / map all the languages.
  pub fn load_from_dir(config: &Config) -> Result<Self, OhNo> {
    let mut langs = HashMap::new();

    // iterate over all known languages in the configuration
    for (lang_name, lang_config) in &config.languages.language {
      log::info!("loading configuration for {lang_name}");

      if let Some(grammar_path) = LanguagesConfig::get_grammar_path(lang_name) {
        log::info!("  grammar path: {}", grammar_path.display());

        let (ts_lib, ts_lang) = match Self::load_grammar(lang_name, &grammar_path) {
          Ok(x) => x,
          Err(err) => {
            log::warn!("{err}");
            continue;
          }
        };

        if let Some(queries_dir) = LanguagesConfig::get_queries_dir(lang_name) {
          log::info!("  queries directory: {}", queries_dir.display());

          let queries = Queries::load_from_dir(queries_dir);
          let mut hl_config = HighlightConfiguration::new(
            ts_lang,
            queries.highlights.as_deref().unwrap_or(""),
            queries.injections.as_deref().unwrap_or(""),
            queries.locals.as_deref().unwrap_or(""),
          )
          .map_err(|err| OhNo::HighlightError {
            err: err.to_string(),
          })?;

          let hl_names: Vec<_> = config.highlight.groups.iter().cloned().collect();
          hl_config.configure(&hl_names);

          let remove_default_highlighter = lang_config.remove_default_highlighter.to_bool();

          let lang = Language {
            hl_config,
            hl_names,
            remove_default_highlighter,
            _ts_lang: ts_lang,
            _ts_lib: ts_lib,
          };
          langs.insert(lang_name.to_owned(), lang);
        }
      }
    }

    Ok(Self { langs })
  }

  pub fn get(&self, filetype: impl AsRef<str>) -> Option<&Language> {
    self.langs.get(filetype.as_ref())
  }
}
