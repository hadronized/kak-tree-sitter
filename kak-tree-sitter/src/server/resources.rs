use std::{
  fs::{self, File},
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
    Command::new("ps")
      .args(["-p", pid])
      .output()
      .is_ok_and(|o| o.status.success())
  }

  /// Create resources, if not already there.
  fn io_create(&self) -> Result<(), OhNo> {
    log::info!("running in {}", self.paths.runtime_dir.display());

    log::debug!("ensuring that runtime directories exist (creating if not)");
    let buffers_dir = self.paths.bufs_dir();
    fs::create_dir_all(&buffers_dir).map_err(|err| OhNo::CannotCreateDir {
      dir: buffers_dir,
      err,
    })?;

    let pid_file = self.paths.pid_path();

    // check whether a pid file exists; remove it if any
    if let Ok(true) = pid_file.try_exists() {
      log::debug!("removing previous PID file");
      std::fs::remove_file(&pid_file).map_err(|err| OhNo::CannotStartDaemon {
        err: format!(
          "cannot remove previous PID file {path}: {err}",
          path = pid_file.display()
        ),
      })?;

      log::debug!("removing previous socket file");
      let socket_file = self.paths.runtime_dir.join("socket");
      if let Err(err) = std::fs::remove_file(&socket_file) {
        if err.kind() != std::io::ErrorKind::NotFound {
          return Err(OhNo::CannotStartDaemon {
            err: format!(
              "cannot remove previous socket file {path}: {err}",
              path = socket_file.display()
            ),
          });
        }
      }
    }

    Ok(())
  }

  pub fn paths(&self) -> &Paths {
    &self.paths
  }

  /// Create the PID file from the current process, or the one of the child
  /// process if daemonized.
  pub fn persist_process(&self, daemonize: bool) -> Result<(), OhNo> {
    let pid_file = self.paths.pid_path();

    if daemonize {
      // create stdout / stderr files
      let stdout_path = self.paths.stdout();
      let stderr_path = self.paths.stderr();
      let stdout = File::create(&stdout_path).map_err(|err| OhNo::CannotCreateFile {
        file: stdout_path,
        err,
      })?;
      let stderr = File::create(&stderr_path).map_err(|err| OhNo::CannotCreateFile {
        file: stderr_path,
        err,
      })?;

      daemonize::Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .pid_file(pid_file)
        .start()
        .map_err(|err| OhNo::CannotStartDaemon {
          err: err.to_string(),
        })?;
    } else {
      fs::write(&pid_file, format!("{}", std::process::id())).map_err(|err| {
        OhNo::CannotWriteFile {
          file: pid_file,
          err,
        }
      })?;
    }

    Ok(())
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
