//! Supported languages.
//!
//! Languages have different objects (grammars, queries, etc.) living at runtime and must be loaded beforehand.

use std::{
  collections::{HashMap, HashSet},
  fs,
  path::Path,
};

use kak_tree_sitter_config::Config;
use libloading::Symbol;
use tree_sitter_highlight::HighlightConfiguration;

use crate::queries::Queries;

pub struct Language {
  pub hl_config: HighlightConfiguration,
  pub hl_names: Vec<String>,

  // NOTE: we need to keep that alive *probably*; better be safe than sorry
  _ts_lang: tree_sitter::Language,
  _ts_lib: libloading::Library,
}

pub struct Languages {
  /// Map a `filetype` to the tree-sitter [`Language`] and its queries.
  langs: HashMap<String, Language>,
}

impl Languages {
  /// Load a grammar.
  fn load_grammar(lang: &str, path: &Path) -> Option<(libloading::Library, tree_sitter::Language)> {
    let lib = unsafe { libloading::Library::new(path) };
    match lib {
      Ok(lib) => {
        let fn_sym = format!("tree_sitter_{}", lang);

        let sym: Result<Symbol<fn() -> tree_sitter::Language>, _> =
          unsafe { lib.get(fn_sym.as_bytes()) };
        match sym {
          Ok(sym) => {
            let ffi_lang = sym();
            Some((lib, ffi_lang))
          }

          Err(err) => {
            eprintln!("cannot find {lang}: {err}");
            None
          }
        }
      }

      Err(err) => {
        eprintln!("cannot load grammar {}: {err}", path.display());
        None
      }
    }
  }

  /// Load languages.
  ///
  /// This function will scan the directory and extract / map all the languages.
  pub fn load_from_dir(config: &Config) -> Self {
    let mut langs = HashMap::new();

    let langs_from_grammars = fs::read_dir(config.languages.get_grammars_dir().unwrap())
      .unwrap()
      .flatten()
      .map(|x| {
        x.file_name()
          .to_str()
          .unwrap()
          .trim_end_matches(".so")
          .to_owned()
      });
    let known_langs: HashSet<_> = config
      .languages
      .language
      .keys()
      .cloned()
      .chain(langs_from_grammars)
      .collect();
    // iterate over all known languages in the configuration
    for lang_name in &known_langs {
      if let Some(grammar_path) = config.languages.get_grammar_path(lang_name) {
        if let Some((ts_lib, ts_lang)) = Self::load_grammar(lang_name, &grammar_path) {
          if let Some(queries_dir) = config.languages.get_queries_dir(lang_name) {
            let queries = Queries::load_from_dir(queries_dir);
            let mut hl_config = HighlightConfiguration::new(
              ts_lang,
              queries.highlights.as_deref().unwrap_or(""),
              queries.injections.as_deref().unwrap_or(""),
              queries.locals.as_deref().unwrap_or(""),
            )
            .unwrap();

            let hl_names = config.languages.get_lang_conf(lang_name).highlight.hl_names;
            hl_config.configure(&hl_names);

            let lang = Language {
              hl_config,
              hl_names,
              _ts_lang: ts_lang,
              _ts_lib: ts_lib,
            };
            langs.insert(lang_name.to_owned(), lang);
          }
        }
      }
    }

    Self { langs }
  }

  pub fn get(&self, filetype: impl AsRef<str>) -> Option<&Language> {
    self.langs.get(filetype.as_ref())
  }
}
