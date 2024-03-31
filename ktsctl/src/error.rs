use std::{io, path::PathBuf};

use kak_tree_sitter_config::error::ConfigError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
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

  #[error("error while waiting for process {process} to end: {err}")]
  ErrorWhileWaitingForProcess { process: String, err: io::Error },

  #[error("error while fetching resource for language {lang}: {err}")]
  FetchError { lang: String, err: String },

  #[error("error while checking out source for language {lang}: {err}")]
  CheckOutError { lang: String, err: String },

  #[error("error while compiling grammar for language {lang}: {err}")]
  CompileError { lang: String, err: io::Error },

  #[error("error while linking grammar for language {lang}: {err}")]
  LinkError { lang: String, err: io::Error },

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
