//! FIFO used for commands, buffers, etc.

use std::{
  ffi::CString,
  fs::{self, File, OpenOptions},
  io::Read,
  os::{
    fd::{AsRawFd, RawFd},
    unix::{ffi::OsStrExt, fs::OpenOptionsExt},
  },
  path::{Path, PathBuf},
};

use crate::error::OhNo;

/// Kind of fifo; that is, the purpose of usage.
///
/// That is used to, mainly, dispatch FIFO IO operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FifoKind {
  Cmd,
  Buf,
}

impl FifoKind {
  /// Prefix used to locate the FIFO inside a filesystem.
  fn as_prefix(self) -> &'static str {
    match self {
      FifoKind::Cmd => "commands",
      FifoKind::Buf => "buffers",
    }
  }
}

#[derive(Debug)]
pub struct Fifo {
  kind: FifoKind,

  /// Session name the FIFO is attached to.
  session: String,

  /// FIFO file.
  file: File,

  /// Path of the FIFO.
  path: PathBuf,

  /// Buffer used to read from the FIFO.
  buffer: String,
}

impl Fifo {
  /// Create and open a non-blocking FIFO.
  ///
  /// If the FIFO doesn’t exist, it’s created before opening.
  pub fn open_nonblocking(
    runtime_dir: impl AsRef<Path>,
    kind: FifoKind,
    session: impl Into<String>,
  ) -> Result<Self, OhNo> {
    let session = session.into();
    let path = Self::create_fifo(runtime_dir.as_ref(), kind.as_prefix(), &session)?;
    let file = Self::open_nonblocking_fifo(path.as_ref())?;
    let buffer = String::new();

    Ok(Self {
      kind,
      session,
      file,
      path,
      buffer,
    })
  }

  /// Wrap libc::mkfifo() and create the FIFO on the filesystem within the [`ServerResources`].
  fn create_fifo(runtime_dir: &Path, prefix: &str, session_name: &str) -> Result<PathBuf, OhNo> {
    let dir = runtime_dir.join(prefix);
    let path = dir.join(session_name);

    // if the file doesn’t already exist, create it
    if let Ok(false) = path.try_exists() {
      // ensure the directory exists
      fs::create_dir_all(&dir).map_err(|err| OhNo::CannotCreateFifo {
        err: format!("cannot create directory {dir}: {err}", dir = dir.display()),
      })?;

      let path_bytes = path.as_os_str().as_bytes();
      let c_path = CString::new(path_bytes).map_err(|err| OhNo::CannotCreateFifo {
        err: err.to_string(),
      })?;

      let c_err = unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };
      if c_err != 0 {
        return Err(OhNo::CannotCreateFifo {
          err: format!(
            "cannot create FIFO file for session {session_name} at path {path}",
            path = path.display()
          ),
        });
      }
    }

    Ok(path)
  }

  /// Open a FIFO and obtain a non-blocking [`File`].
  fn open_nonblocking_fifo(path: &Path) -> Result<File, OhNo> {
    let fifo = OpenOptions::new()
      .read(true)
      .custom_flags(libc::O_NONBLOCK)
      .open(path)
      .map_err(|err| OhNo::CannotCreateFifo {
        err: format!(
          "cannot open non-blocking FIFO {path}: {err}",
          path = path.display()
        ),
      })?;
    Ok(fifo)
  }

  pub fn kind(&self) -> FifoKind {
    self.kind
  }

  /// Session the FIFO is attached to.
  pub fn session(&self) -> &str {
    &self.session
  }

  /// Read on the FIFO until no data is available.
  pub fn read_all(&mut self) -> Result<&str, OhNo> {
    self.buffer.clear();

    loop {
      match self.file.read_to_string(&mut self.buffer) {
        Ok(bytes) => {
          if bytes == 0 {
            break;
          }
        }
        Err(err) => {
          if err.kind() == std::io::ErrorKind::WouldBlock {
            log::trace!("FIFO not ready");
          } else {
            return Err(OhNo::CannotReadFifo { err });
          }
        }
      }
    }

    Ok(self.buffer.as_str())
  }

  pub fn as_raw_fd(&self) -> RawFd {
    self.file.as_raw_fd()
  }

  pub fn path(&self) -> &Path {
    &self.path
  }
}
