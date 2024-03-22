use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Tree-sitter runtime resources sources.
///
/// Sources can be local or remote. In the case of remote sources, we only support git repositories for now.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
  Path { dir: PathBuf },
  Git { url: String, pin: Option<String> },
}

impl Source {
  pub fn path(dir: impl Into<PathBuf>) -> Self {
    let dir = dir.into();
    Self::Path { dir }
  }

  pub fn git(url: impl Into<String>, pin: impl Into<Option<String>>) -> Self {
    let url = url.into();
    let pin = pin.into();
    Self::Git { url, pin }
  }
}
