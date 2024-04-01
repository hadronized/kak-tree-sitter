//! Report progress and status of commands.

use std::{
  fmt::Display,
  io::{stdout, Write},
};

use colored::Colorize;

/// A report that can updates itself on stdout.
///
/// This type can be used whenever you want to output progress of a task on the same line. Every report will replace
/// the current line, allowing an “in-place” report update, progress bar, etc.
///
/// You use it with [`Report::new`] to create the initial line, and then use the [`Report::update()`] function to update
/// the report, or one of the other methods to update the report.
///
/// Once you are done and want to finish the report and go to the next line, simply drop the report.
#[derive(Debug)]
pub struct Report;

impl Report {
  pub fn new(icon: ReportIcon, msg: impl AsRef<str>) -> Self {
    print!("\x1b[?7l");
    Self::to_stdout(icon, msg);
    Self
  }

  fn to_stdout(icon: ReportIcon, msg: impl AsRef<str>) {
    print!("{} {msg}", icon, msg = msg.as_ref());
    stdout().flush().unwrap();
  }

  pub fn update(&self, icon: ReportIcon, msg: impl AsRef<str>) {
    print!("\x1b[2K\r");
    Self::to_stdout(icon, msg);
  }

  pub fn fetch(&self, msg: impl AsRef<str>) {
    self.update(ReportIcon::Fetch, msg)
  }

  pub fn sync(&self, msg: impl AsRef<str>) {
    self.update(ReportIcon::Sync, msg)
  }

  pub fn compile(&self, msg: impl AsRef<str>) {
    self.update(ReportIcon::Compile, msg)
  }

  pub fn link(&self, msg: impl AsRef<str>) {
    self.update(ReportIcon::Link, msg)
  }

  pub fn install(&self, msg: impl AsRef<str>) {
    self.update(ReportIcon::Install, msg)
  }

  pub fn success(&self, msg: impl AsRef<str>) {
    self.update(ReportIcon::Success, msg)
  }

  pub fn error(&self, msg: impl AsRef<str>) {
    self.update(ReportIcon::Error, msg)
  }

  pub fn info(&self, msg: impl AsRef<str>) {
    self.update(ReportIcon::Info, msg)
  }
}

impl Drop for Report {
  fn drop(&mut self) {
    println!("\x1b[?7h");
    stdout().flush().unwrap();
  }
}

#[derive(Debug)]
pub enum ReportIcon {
  Fetch,
  Sync,
  Compile,
  Link,
  Install,
  Success,
  Error,
  Info,
}

impl Display for ReportIcon {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ReportIcon::Fetch => write!(f, "{}", "".magenta()),
      ReportIcon::Sync => write!(f, "{}", "".magenta()),
      ReportIcon::Compile => write!(f, "{}", "".cyan()),
      ReportIcon::Link => write!(f, "{}", "".cyan()),
      ReportIcon::Install => write!(f, "{}", "".cyan()),
      ReportIcon::Success => write!(f, "{}", "".green()),
      ReportIcon::Error => write!(f, "{}", "".red()),
      ReportIcon::Info => write!(f, "{}", "󰈅".blue()),
    }
  }
}
