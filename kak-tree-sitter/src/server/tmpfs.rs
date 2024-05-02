//! Temporary file system module.
//!
//! This module is used to expose files that can be written to by Kakoune and
//! read by our server. The purpose is mainly for streaming buffer content in a
//! fast and reliable way.
//!
//! Once a file is read, it’s automatically deleted by the server.

use std::path::{Path, PathBuf};

use crate::error::OhNo;

#[derive(Debug)]
pub struct TmpFile {
  path: PathBuf,
}

impl TmpFile {
  pub fn new(path: impl Into<PathBuf>) -> Self {
    Self { path: path.into() }
  }

  pub fn path(&self) -> &Path {
    &self.path
  }

  /// Read the temporary file and drop it.
  ///
  /// Once the file is dropped, it is removed from the tmpfs.
  pub fn into_string(self) -> Result<String, OhNo> {
    let s = std::fs::read_to_string(&self.path).map_err(|err| OhNo::CannotReadTmpFile { err })?;
    Ok(s)
  }
}

impl Drop for TmpFile {
  fn drop(&mut self) {
    if let Err(err) = std::fs::remove_file(&self.path) {
      log::error!("cannot delete tmpfs file {}: {err}", self.path.display());
    }
  }
}
