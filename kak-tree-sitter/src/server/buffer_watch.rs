//! Watch over buffer content as they changed on the tmpfs.

use std::{
  path::Path,
  sync::{
    mpsc::{channel, Receiver},
    Arc,
  },
  time::Duration,
};

use mio::Waker;
use notify_debouncer_full::{
  notify::{event::CreateKind, EventKind, RecommendedWatcher, RecursiveMode, Watcher},
  DebouncedEvent, Debouncer, FileIdMap,
};

use crate::error::OhNo;

use super::tmpfs::TmpFile;

/// Current view of a buffer we have.
#[derive(Debug)]
pub struct BufferView {
  timestamp: u64,
  name: String,
  content: String,
}

impl BufferView {
  /// Create a [`BufView`] from a temporary file.
  pub fn new(file: TmpFile) -> Result<Self, OhNo> {
    // SAFETY: should be fiiiiiiiiiine
    let filename = file.path().file_name().unwrap().to_str().unwrap();

    let dash_idx = filename
      .find('-')
      .ok_or_else(|| OhNo::BufferViewDecodeError {
        path: file.path().to_owned(),
        err: format!("cannot parse timestamp"),
      })?;
    let timestamp =
      filename[..dash_idx]
        .parse::<u64>()
        .map_err(|err| OhNo::BufferViewDecodeError {
          path: file.path().to_owned(),
          err: err.to_string(),
        })?;

    let name = filename[dash_idx + 1..].to_owned();
    let content = file.into_string()?;

    Ok(Self {
      name,
      timestamp,
      content,
    })
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn content(&self) -> &str {
    &self.content
  }
}

/// Watch over buffers, updating them when new buffers are written.
#[derive(Debug)]
pub struct BufferWatch {
  watcher: Debouncer<RecommendedWatcher, FileIdMap>,
  update_rx: Receiver<BufferView>,
}

impl BufferWatch {
  pub fn new(buffer_dir: impl AsRef<Path>, waker: Arc<Waker>) -> Result<Self, OhNo> {
    let (update_tx, update_rx) = channel();
    let mut watcher = notify_debouncer_full::new_debouncer(
      Duration::from_millis(80),
      None,
      move |events: Result<Vec<DebouncedEvent>, _>| {
        if let Ok(events) = events {
          for event in events {
            if let EventKind::Create(CreateKind::File) = event.kind {
              for path in &event.paths {
                log::debug!("detected new file creation: {}", path.display());

                match BufferView::new(TmpFile::new(path)) {
                  Ok(bv) => {
                    if let Err(err) = update_tx.send(bv) {
                      log::error!("cannot send buffer view update; channel error: {err}");
                    } else if let Err(err) = waker.wake() {
                      log::error!("cannot wake poll thread: {err}");
                    }
                  }
                  Err(err) => {
                    log::error!("error while creating buffer view: {err}");
                  }
                }
              }
            }
          }
        }
      },
    )
    .map_err(|err| OhNo::BufferWatchError {
      err: format!("{err}"),
    })?;

    watcher
      .watcher()
      .watch(buffer_dir.as_ref(), RecursiveMode::NonRecursive)
      .map_err(|err| OhNo::BufferWatchError {
        err: format!("{err}"),
      })?;

    Ok(Self { watcher, update_rx })
  }

  pub fn updates(&self) -> impl '_ + Iterator<Item = BufferView> {
    self.update_rx.try_iter()
  }
}
