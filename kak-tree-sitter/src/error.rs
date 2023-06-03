use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OhNo {
  #[error("no runtime directory")]
  NoRuntimeDir,

  #[error("cannot create directory {dir}: {err}")]
  CannotCreateDir { dir: PathBuf, err: io::Error },

  #[error("cannot create file {file}: {err}")]
  CannotCreateFile { file: PathBuf, err: io::Error },

  #[error("cannot write to file {file}: {err}")]
  CannotWriteFile { file: PathBuf, err: io::Error },

  #[error("cannot start daemon: {err}")]
  CannotStartDaemon { err: String },

  #[error("cannot start async runtime: {err}")]
  CannotStartAsyncRuntime { err: io::Error },

  #[error("cannot start server: {err}")]
  CannotStartServer { err: io::Error },

  #[error("invalid request: {err}")]
  InvalidRequest { err: String },

  #[error("cannot read buffer from filesystem: {err}")]
  CannotReadBuffer { err: io::Error },

  #[error("cannot connect to server; is it running?: {err}")]
  CannotConnectToServer { err: io::Error },

  #[error("cannot send request: {err}")]
  CannotSendRequest { err: String },

  #[error("error while shutting down: {err}")]
  ShutdownFailure { err: String },

  #[error("highlight error: {err}")]
  HighlightError { err: String },
}
