//! Supported queries.

use std::{fs, path::Path};

#[derive(Debug)]
pub struct Queries {
  pub highlights: Option<String>,
  pub injections: Option<String>,
  pub locals: Option<String>,
  pub textobjects: Option<String>,
  pub indents: Option<String>,
}

impl Queries {
  pub fn load_from_dir(dir: impl AsRef<Path>) -> Self {
    let dir = dir.as_ref();

    let highlights = fs::read_to_string(dir.join("highlights.scm")).ok();
    let injections = fs::read_to_string(dir.join("injections.scm")).ok();
    let locals = fs::read_to_string(dir.join("locals.scm")).ok();
    let textobjects = fs::read_to_string(dir.join("textobjects.scm")).ok();
    let indents = fs::read_to_string(dir.join("indents.scm")).ok();

    Queries {
      highlights,
      injections,
      locals,
      textobjects,
      indents,
    }
  }
}
