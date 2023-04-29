//! Supported queries.

use std::{fs, path::Path};

#[derive(Debug)]
pub struct Queries {
  pub highlights: String,
  pub injections: String,
  pub locals: String,
  pub text_objects: String,
}

impl Queries {
  pub fn load_from_dir(dir: impl AsRef<Path>) -> Option<Self> {
    let dir = dir.as_ref();

    let highlights = fs::read_to_string(dir.join("highlights.scm")).ok()?;
    let injections = fs::read_to_string(dir.join("injections.scm")).ok()?;
    let locals = fs::read_to_string(dir.join("locals.scm")).ok()?;
    let text_objects = fs::read_to_string(dir.join("text_objects.scm")).ok()?;

    Some(Queries {
      highlights,
      injections,
      locals,
      text_objects,
    })
  }
}
