//! Report progress and status of commands.

use std::io::{stdout, Write};

use super::status_icon::StatusIcon;

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
  pub fn new(icon: StatusIcon, msg: impl AsRef<str>) -> Self {
    print!("\x1b[?7l");
    Self::to_stdout(icon, msg);
    Self
  }

  fn to_stdout(icon: StatusIcon, msg: impl AsRef<str>) {
    print!("{} {msg}", icon, msg = msg.as_ref());
    stdout().flush().unwrap();
  }

  pub fn update(&self, icon: StatusIcon, msg: impl AsRef<str>) {
    print!("\x1b[2K\r");
    Self::to_stdout(icon, msg);
  }

  pub fn fetch(&self, msg: impl AsRef<str>) {
    self.update(StatusIcon::Fetch, msg)
  }

  pub fn sync(&self, msg: impl AsRef<str>) {
    self.update(StatusIcon::Sync, msg)
  }

  pub fn success(&self, msg: impl AsRef<str>) {
    self.update(StatusIcon::Success, msg)
  }

  pub fn info(&self, msg: impl AsRef<str>) {
    self.update(StatusIcon::Info, msg)
  }
}

impl Drop for Report {
  fn drop(&mut self) {
    println!("\x1b[?7h");
    stdout().flush().unwrap();
  }
}
