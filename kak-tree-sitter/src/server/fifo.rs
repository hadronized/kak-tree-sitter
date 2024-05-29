//! A simple FIFO wrapper.

use std::{
  ffi::CString,
  fs::{self, File, OpenOptions},
  io::{ErrorKind, Read},
  os::{
    fd::AsRawFd,
    unix::{ffi::OsStrExt, fs::OpenOptionsExt},
  },
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use mio::{unix::SourceFd, Interest, Registry, Token};

use crate::error::OhNo;

use super::tokens::Tokens;

#[derive(Debug)]
pub struct Fifo {
  registry: Arc<Registry>,
  tokens: Arc<Mutex<Tokens>>,
  path: PathBuf,
  file: File,
  tkn: Token,
  sentinel: String,
  buf: String,
}

impl Fifo {
  /// Create a FIFO.
  pub fn create(
    registry: &Arc<Registry>,
    tokens: &Arc<Mutex<Tokens>>,
    path: impl Into<PathBuf>,
  ) -> Result<Self, OhNo> {
    let path = path.into();

    Self::create_fifo(&path)?;
    let file = Self::open_nonblocking(&path)?;
    let tkn = Self::register(registry, tokens, &file)?;
    let sentinel = uuid::Uuid::new_v4().to_string();

    Ok(Self {
      registry: registry.clone(),
      tokens: tokens.clone(),
      path,
      file,
      tkn,
      sentinel,
      buf: String::default(),
    })
  }

  /// Wrap libc::mkfifo() and create the FIFO on the filesystem within the [`ServerResources`].
  fn create_fifo(path: &Path) -> Result<(), OhNo> {
    // if the file already exists, abort
    if let Ok(true) = path.try_exists() {
      log::debug!("FIFO already exists for path {}", path.display());
      return Ok(());
    }

    let path_bytes = path.as_os_str().as_bytes();
    let c_path = CString::new(path_bytes).map_err(|err| OhNo::CannotCreateFifo {
      err: err.to_string(),
    })?;

    let c_err = unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };
    if c_err != 0 {
      return Err(OhNo::CannotCreateFifo {
        err: format!("cannot create FIFO at path {path}", path = path.display()),
      });
    }

    Ok(())
  }

  fn open_nonblocking(path: &Path) -> Result<File, OhNo> {
    OpenOptions::new()
      .read(true)
      .custom_flags(libc::O_NONBLOCK)
      .open(path)
      .map_err(|err| OhNo::CannotOpenFifo { err })
  }

  fn register(
    registry: &Arc<Registry>,
    tokens: &Arc<Mutex<Tokens>>,
    file: &File,
  ) -> Result<Token, OhNo> {
    let tkn = tokens.lock().expect("tokens").create();
    registry
      .register(&mut SourceFd(&file.as_raw_fd()), tkn, Interest::READABLE)
      .map_err(|err| OhNo::PollError { err })?;

    Ok(tkn)
  }

  fn unregister(&self) {
    if let Err(err) = self
      .registry
      .deregister(&mut SourceFd(&self.file.as_raw_fd()))
    {
      log::error!(
        "cannot unregister FIFO {path} from poll registry: {err}",
        path = self.path.display()
      );
    }

    self.tokens.lock().expect("tokens").recycle(self.tkn);
  }

  pub fn token(&self) -> &Token {
    &self.tkn
  }

  pub fn path(&self) -> &Path {
    &self.path
  }

  pub fn sentinel(&self) -> &str {
    &self.sentinel
  }

  pub fn read_to_buf(&mut self, target: &mut String) -> Result<bool, OhNo> {
    loop {
      match self.file.read_to_string(&mut self.buf) {
        Ok(0) => break,
        Ok(_) => continue,

        Err(err) => match err.kind() {
          ErrorKind::WouldBlock => break,

          _ => {
            // reset the buffer in case of errors
            self.buf.clear();
            return Err(OhNo::CannotReadFifo { err });
          }
        },
      }
    }

    // search for the sentinel; if we find it, it means we have a complete
    // buffer; cut it from the data and reset to be ready to read the next
    // buffer
    if let Some(index) = self.buf.find(&self.sentinel) {
      log::trace!(
        "found sentinel {sentinel} in buffer {path}",
        sentinel = self.sentinel,
        path = self.path.display()
      );
      target.clear();
      target.push_str(&self.buf[..index]);

      log::trace!("new buffer content:\n{target}");

      self.buf.drain(..index + self.sentinel.len());
      return Ok(true);
    }

    Ok(false)
  }
}

// We implement Drop here because we want to clean the FIFO when the session
// exits automatically. Failing doing so is not a hard error.
impl Drop for Fifo {
  fn drop(&mut self) {
    self.unregister();

    if let Err(err) = fs::remove_file(&self.path) {
      log::warn!("cannot remove FIFO at path {}: {err}", self.path.display());
    }
  }
}
