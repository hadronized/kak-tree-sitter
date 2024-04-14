//! Process handling.

use std::{
  io::Read,
  path::Path,
  process::{Command, Stdio},
};

use crate::error::HellNo;

/// Small wrapper above [`std::process::Command`].
///
/// A process is run by muting its stdout and piping its stderr. The exit code of the process is checked and if it’s
/// not `0`, the content of its stderr file is returned in [`AppError`].
#[derive(Debug)]
pub struct Process<'a> {
  name: &'a str,
}

impl<'a> Process<'a> {
  pub fn new(name: &'a str) -> Self {
    Self { name }
  }

  pub fn run<'b>(&self, cwd: impl Into<Option<&'b Path>>, args: &[&str]) -> Result<(), HellNo> {
    let process = format!("{} {}", self.name, args.join(" "));
    let mut cmd = Command::new(self.name);
    cmd.args(args);

    if let Some(cwd) = cwd.into() {
      cmd.current_dir(cwd);
    }

    let mut child = cmd
      .stdout(Stdio::null())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|err| HellNo::ProcessRunError {
        process: process.clone(),
        err,
      })?;
    let stderr = child.stderr.take();

    let exit_status = child.wait().map_err(|err| HellNo::ProcessRunError {
      process: process.clone(),
      err,
    })?;

    if !exit_status.success() {
      if let Some(mut stderr) = stderr {
        let mut err = String::new();
        stderr
          .read_to_string(&mut err)
          .map_err(|err| HellNo::ProcessRunError {
            process: process.clone(),
            err,
          })?;

        return Err(HellNo::ProcessExitedWithError {
          process: process.clone(),
          err: err.to_string(),
        });
      }
    }

    Ok(())
  }
}
