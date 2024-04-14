use std::fmt::Display;

use colored::{ColoredString, Colorize};

#[derive(Debug)]
pub enum StatusIcon {
  Fetch,
  Sync,
  Compile,
  Link,
  Install,
  Success,
  Error,
  Warn,
  Info,
}

impl From<StatusIcon> for ColoredString {
  fn from(value: StatusIcon) -> Self {
    match value {
      StatusIcon::Fetch => "".magenta(),
      StatusIcon::Sync => "".magenta(),
      StatusIcon::Compile => "".cyan(),
      StatusIcon::Link => "".cyan(),
      StatusIcon::Install => "".cyan(),
      StatusIcon::Success => "".green(),
      StatusIcon::Error => "".red(),
      StatusIcon::Warn => "".yellow(),
      StatusIcon::Info => "󰈅".blue(),
    }
  }
}

impl Display for StatusIcon {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      StatusIcon::Fetch => write!(f, "{}", "".magenta()),
      StatusIcon::Sync => write!(f, "{}", "".magenta()),
      StatusIcon::Compile => write!(f, "{}", "".cyan()),
      StatusIcon::Link => write!(f, "{}", "".cyan()),
      StatusIcon::Install => write!(f, "{}", "".cyan()),
      StatusIcon::Success => write!(f, "{}", "".green()),
      StatusIcon::Error => write!(f, "{}", "".red()),
      StatusIcon::Warn => write!(f, "{}", "".yellow()),
      StatusIcon::Info => write!(f, "{}", "󰈅".blue()),
    }
  }
}
