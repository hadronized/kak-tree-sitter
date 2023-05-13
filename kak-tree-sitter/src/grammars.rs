//! Supported grammars.
//!
//! Grammars live at runtime and must be loaded beforehand.

use std::{collections::HashMap, fs, path::Path};

use libloading::Symbol;
use tree_sitter::Language;

use crate::queries::Queries;

#[derive(Debug)]
pub struct Grammars {
  /// Map a `filetype` to the tree-sitter [`Language`] and its queries.
  langs: HashMap<String, (libloading::Library, Language, Queries)>,
}

impl Grammars {
  /// Load grammars from a directory.
  ///
  /// Grammar files must be named according to the pattern `<filetype>.so` in that directory. For instance, for a
  /// `rust` grammar, it must be named `rust.so`.
  ///
  /// This function will scan the directory and extract / map all the grammars.
  pub fn load_from_dir(dir: impl AsRef<Path>) -> Option<Grammars> {
    let mut langs = HashMap::new();

    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
      let lib = unsafe { libloading::Library::new(entry.path()) };
      match lib {
        Ok(lib) => {
          let file_name = entry.file_name();
          let file_name = file_name.to_str().unwrap();

          // check that this is what we expect
          if !file_name.ends_with(".so") {
            continue;
          }

          let lang = file_name.trim_end_matches(".so");
          let fn_sym = format!("tree_sitter_{}", lang);

          let sym: Result<Symbol<fn() -> Language>, _> = unsafe { lib.get(fn_sym.as_bytes()) };
          match sym {
            Ok(sym) => {
              let ffi_lang = sym();

              // get the queries
              let queries = Queries::load_from_dir()

              langs.insert(lang.to_owned(), (lib, ffi_lang));
            }

            Err(err) => eprintln!("cannot find {lang}: {err}"),
          }
        }

        Err(err) => eprintln!("cannot load grammar {}: {err}", entry.path().display()),
      }
    }

    Some(Grammars { langs })
  }

  /// Get a grammar by looking up a filetype.
  pub fn get(&self, filetype: impl AsRef<str>) -> Option<&Language> {
    self.langs.get(filetype.as_ref()).map(|(_, lang)| lang)
  }
}
