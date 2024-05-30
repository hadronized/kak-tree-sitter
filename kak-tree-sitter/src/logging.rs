//! Logging related module.

use log::{Level, Log};

use crate::error::OhNo;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Verbosity {
  #[default]
  Error,
  Warn,
  Info,
  Debug,
  Trace,
  Off,
}

impl Verbosity {
  pub fn from_count(count: u8) -> Self {
    match count {
      0 => Self::Off,
      1 => Self::Error,
      2 => Self::Warn,
      3 => Self::Info,
      4 => Self::Debug,
      _ => Self::Trace,
    }
  }

  pub fn to_level(self) -> Option<log::Level> {
    match self {
      Verbosity::Error => Some(log::Level::Error),
      Verbosity::Warn => Some(log::Level::Warn),
      Verbosity::Info => Some(log::Level::Info),
      Verbosity::Debug => Some(log::Level::Debug),
      Verbosity::Trace => Some(log::Level::Trace),
      Verbosity::Off => None,
    }
  }
}

/// A logger that simply writes to a kakoune session.
///
/// This logger is important when the binary is started from within Kakoune, as Kakoune interprets stdout.
#[derive(Debug)]
pub struct KakouneLogger {
  level: Level,
}

impl KakouneLogger {
  pub fn new(level: Level) -> Self {
    Self { level }
  }

  pub fn register(self) -> Result<(), OhNo> {
    log::set_max_level(self.level.to_level_filter());
    log::set_boxed_logger(Box::new(self))?;

    Ok(())
  }
}

impl Log for KakouneLogger {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
    metadata.level() <= self.level
  }

  fn log(&self, record: &log::Record) {
    if !self.enabled(record.metadata()) {
      return;
    }

    let now = chrono::Utc::now();
    println!(
      "echo -debug -- tree-sitter {time} ({target}) [{level}]: {args};",
      time = now.to_rfc3339(),
      target = record.target(),
      level = record.level(),
      args = record.args()
    );
  }

  fn flush(&self) {}
}
