use std::{fs, path::PathBuf, process::Command};

use crate::error::OhNo;

/// Resources requiring a special drop implementation.
#[derive(Clone, Debug)]
pub struct ServerResources {
  pub runtime_dir: PathBuf,
}

impl ServerResources {
  pub fn new() -> Result<Self, OhNo> {
    let dir = dirs::runtime_dir()
      .or_else(||
        // macOS doesn’t implement XDG, yay…
        std::env::var("TMPDIR").map(PathBuf::from).ok())
      .ok_or_else(|| OhNo::NoRuntimeDir)?;
    let runtime_dir = dir.join("kak-tree-sitter");

    // create the resources
    let sr = Self { runtime_dir };
    sr.io_create()?;

    Ok(sr)
  }

  /// Create resources, if not already there.
  fn io_create(&self) -> Result<(), OhNo> {
    log::info!("running in {}", self.runtime_dir.display());

    log::debug!("ensuring that runtime directories exist (creating if not)");
    let commands_dir = self.runtime_dir.join("commands");
    fs::create_dir_all(&commands_dir).map_err(|err| OhNo::CannotCreateDir {
      dir: commands_dir,
      err,
    })?;

    let buffers_dir = self.runtime_dir.join("buffers");
    fs::create_dir_all(&buffers_dir).map_err(|err| OhNo::CannotCreateDir {
      dir: buffers_dir,
      err,
    })?;

    let pid_file = self.pid_path();

    // check whether a pid file exists and can be read
    if let Ok(pid) = std::fs::read_to_string(&pid_file) {
      let pid = pid.trim();
      log::debug!("checking whether PID {pid} is still up…");

      // if the contained pid corresponds to a running process, stop right away
      // otherwise, remove the previous PID and socket files
      if Self::is_running(pid) {
        log::debug!("kak-tree-sitter already running; not starting a new server");
        return Ok(());
      }

      log::debug!("removing previous PID file");
      std::fs::remove_file(&pid_file).map_err(|err| OhNo::CannotStartDaemon {
        err: format!(
          "cannot remove previous PID file {path}: {err}",
          path = pid_file.display()
        ),
      })?;

      log::debug!("removing previous socket file");
      let socket_file = self.runtime_dir.join("socket");
      std::fs::remove_file(&socket_file).map_err(|err| OhNo::CannotStartDaemon {
        err: format!(
          "cannot remove previous socket file {path}: {err}",
          path = socket_file.display()
        ),
      })?;
    }

    Ok(())
  }

  fn is_running(pid: &str) -> bool {
    Command::new("ps")
      .args(["-p", pid])
      .output()
      .is_ok_and(|o| o.status.success())
  }

  /// Socket used by the server to receive initial commands.
  pub fn socket_path(&self) -> PathBuf {
    self.runtime_dir.join("socket")
  }

  pub fn pid_path(&self) -> PathBuf {
    self.runtime_dir.join("pid")
  }
}

impl Drop for ServerResources {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(self.runtime_dir.join("pid"));
  }
}
