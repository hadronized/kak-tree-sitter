use std::{io, path::PathBuf};

use kak_tree_sitter_config::error::ConfigError;
use thiserror::Error;

/// Hell no!
#[derive(Debug, Error)]
pub enum HellNo {
  #[error("logger failed to initialize: {err}")]
  LoggerError {
    #[from]
    err: log::SetLoggerError,
  },

  #[error("no runtime directory available")]
  NoRuntimeDir,

  #[error("no data directory to hold grammars / queries")]
  NoDataDir,

  #[error("bad path")]
  BadPath,

  #[error("cannot create directory {dir}: {err}")]
  CannotCreateDir { dir: PathBuf, err: io::Error },

  #[error("configuration error: {err}")]
  ConfigError {
    #[from]
    err: ConfigError,
  },

  #[error("no configuration for language {lang}")]
  MissingLangConfig { lang: String },

  #[error("{process} failed to run: {err}")]
  ProcessRunError { process: String, err: io::Error },

  #[error("{process} exited with error: {err}")]
  ProcessExitedWithError { process: String, err: String },

  #[error("cannot copy {src} to {dest}: {err}")]
  CannotCopyFile {
    src: PathBuf,
    dest: PathBuf,
    err: io::Error,
  },

  #[error("cannot recursively copy from {src} to {dest}: {err}")]
  CannotCopyDir {
    src: PathBuf,
    dest: PathBuf,
    err: io::Error,
  },
}
