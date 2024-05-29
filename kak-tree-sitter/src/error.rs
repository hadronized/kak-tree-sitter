use std::{io, path::PathBuf};

use kak_tree_sitter_config::error::ConfigError;
use log::SetLoggerError;
use mio::Token;
use thiserror::Error;
use tree_sitter::{LanguageError, QueryError};

use crate::kakoune::buffer::BufferId;

#[derive(Debug, Error)]
pub enum OhNo {
  #[error("nothing to do; please either use --server or --request")]
  NothingToDo,

  #[error("no runtime directory")]
  NoRuntimeDir,

  #[error("cannot initialize logging: {err}")]
  LoggerInit {
    #[from]
    err: SetLoggerError,
  },

  #[error("configuration error: {err}")]
  ConfigError {
    #[from]
    err: ConfigError,
  },

  #[error("cannot create directory {dir}: {err}")]
  CannotCreateDir { dir: PathBuf, err: io::Error },

  #[error("cannot create file {file}: {err}")]
  CannotCreateFile { file: PathBuf, err: io::Error },

  #[error("cannot write to file {file}: {err}")]
  CannotWriteFile { file: PathBuf, err: io::Error },

  #[error("cannot start daemon: {err}")]
  CannotStartDaemon { err: String },

  #[error("cannot set SIGINT handler: {err}")]
  SigIntHandlerError {
    #[from]
    err: ctrlc::Error,
  },

  #[error("cannot create FIFO: {err}")]
  CannotCreateFifo { err: String },

  #[error("cannot open non-blocking FIFO: {err}")]
  CannotOpenFifo { err: io::Error },

  #[error("cannot read FIFO: {err}")]
  CannotReadFifo { err: io::Error },

  #[error("poll error: {err}")]
  PollError { err: io::Error },

  #[error("cannot start server: {err}")]
  CannotStartServer { err: io::Error },

  #[error("cannot load grammar for language {lang}: {err}")]
  CannotLoadGrammar { lang: String, err: String },

  #[error("UNIX socket error: {err}")]
  UnixSocketError { err: io::Error },

  #[error("invalid request {req}: {err}")]
  InvalidRequest { req: String, err: String },

  #[error("cannot connect to server; is it running?: {err}")]
  CannotConnectToServer { err: io::Error },

  #[error("cannot send request: {err}")]
  CannotSendRequest { err: String },

  #[error("cannot parse buffer")]
  CannotParseBuffer,

  #[error("highlight error: {err}")]
  HighlightError { err: String },

  #[error("unknown language: {lang}")]
  UnknownLang { lang: String },

  #[error("unknown buffer: {id:?}")]
  UnknownBuffer { id: BufferId },

  #[error("unknown buffer token: {tkn:?}")]
  UnknownToken { tkn: Token },

  #[error("language error: {err}")]
  LangError {
    #[from]
    err: LanguageError,
  },

  #[error("query error: {err}")]
  QueryError {
    #[from]
    err: QueryError,
  },

  #[error("text-objects not supported")]
  UnsupportedTextObjects,

  #[error("no such {pattern} text-object query")]
  UnknownTextObjectQuery { pattern: String },
}
