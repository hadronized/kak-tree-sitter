//! Logging related module.

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
