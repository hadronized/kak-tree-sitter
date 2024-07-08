use std::{
  fs,
  path::PathBuf,
  process::Command,
  sync::{Arc, Mutex},
};

use mio::Registry;

use crate::error::OhNo;

use super::{fifo::Fifo, tokens::Tokens};

/// Paths used by all KTS components.
#[derive(Debug)]
pub struct Paths {
  runtime_dir: PathBuf,
}

impl Paths {
  pub fn new() -> Result<Self, OhNo> {
    let dir = dirs::runtime_dir()
      .or_else(||
        // macOS doesn’t implement XDG, yay…
        std::env::var("TMPDIR").map(PathBuf::from).ok())
      .ok_or_else(|| OhNo::NoRuntimeDir)?;
    let runtime_dir = dir.join("kak-tree-sitter");

    // create the runtime dir if it doesn’t exist
    if let Ok(false) = runtime_dir.try_exists() {
      fs::create_dir(&runtime_dir).map_err(|err| OhNo::CannotCreateDir {
        dir: runtime_dir.clone(),
        err,
      })?;
    }

    Ok(Paths { runtime_dir })
  }

  /// Socket used by the server to receive initial commands.
  pub fn socket_path(&self) -> PathBuf {
    self.runtime_dir.join("socket")
  }

  pub fn pid_path(&self) -> PathBuf {
    self.runtime_dir.join("pid")
  }

  pub fn stdout(&self) -> PathBuf {
    self.runtime_dir.join("stdout.txt")
  }

  pub fn stderr(&self) -> PathBuf {
    self.runtime_dir.join("stderr.txt")
  }

  pub fn bufs_dir(&self) -> PathBuf {
    self.runtime_dir.join("bufs")
  }
}

/// Resources requiring a special drop implementation.
#[derive(Debug)]
pub struct ServerResources {
  paths: Paths,
  tokens: Arc<Mutex<Tokens>>,
  registry: Arc<Registry>,
}

impl ServerResources {
  pub fn new(paths: Paths, registry: Arc<Registry>) -> Result<Self, OhNo> {
    log::debug!("initializing server resources");
    let tokens = Arc::new(Mutex::new(Tokens::default()));

    // create the resources
    let sr = Self {
      paths,
      tokens,
      registry,
    };
    sr.io_create()?;

    Ok(sr)
  }

  pub fn is_running(pid: &str) -> bool {
    log::debug!("checking whether kak-tree-sitter ({pid}) is still running");
    Command::new("ps")
      .args(["-p", pid])
      .output()
      .is_ok_and(|o| o.status.success())
  }

  /// Create resources, if not already there.
  fn io_create(&self) -> Result<(), OhNo> {
    log::debug!("ensuring that runtime directories exist (creating if not)");
    let buffers_dir = self.paths.bufs_dir();
    fs::create_dir_all(&buffers_dir).map_err(|err| OhNo::CannotCreateDir {
      dir: buffers_dir,
      err,
    })?;

    Ok(())
  }

  pub fn paths(&self) -> &Paths {
    &self.paths
  }

  pub fn tokens(&self) -> &Arc<Mutex<Tokens>> {
    &self.tokens
  }

  /// Generate a new, unique FIFO path.
  fn new_fifo_path(&self) -> Result<PathBuf, OhNo> {
    let name = uuid::Uuid::new_v4();
    Ok(self.paths.bufs_dir().join(name.to_string()))
  }

  /// Create a new FIFO and associate it with a token on the given poll.
  pub fn new_fifo(&mut self) -> Result<Fifo, OhNo> {
    let path = self.new_fifo_path()?;
    Fifo::create(&self.registry, &self.tokens, path)
  }
}

impl Drop for ServerResources {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(self.paths.pid_path());
    let _ = std::fs::remove_dir_all(self.paths.socket_path());
  }
}
