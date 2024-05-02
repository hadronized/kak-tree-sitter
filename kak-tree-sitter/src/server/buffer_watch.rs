//! Watch over buffer content as they changed on the tmpfs.

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::{Arc, Mutex, RwLock},
  time::Duration,
};

use notify::{recommended_watcher, RecommendedWatcher};
use notify_debouncer_full::{
  notify::{event::CreateKind, EventKind},
  Debouncer, FileIdMap,
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
}

/// Watch over buffers, updating them when new buffers are written.
#[derive(Debug)]
pub struct BufferWatch {
  watcher: Debouncer<RecommendedWatcher, FileIdMap>,
  views: Arc<Mutex<HashMap<String, BufferView>>>,
}

impl BufferWatch {
  pub fn new(buffer_dir: impl AsRef<Path>) -> Result<Self, OhNo> {
    let views = Arc::new(Mutex::new(HashMap::default()));

    let views_ = views.clone();
    let watcher = notify_debouncer_full::new_debouncer(Duration::from_millis(80), None, |events| {
      if let Ok(events) = events {
        for event in events {
          match event.kind {
            EventKind::Create(CreateKind::File) => {
              let views = views_.lock().expect("poisoned");
            }

            _ => (),
          }
        }
      }
    })
    .map_err(|err| OhNo::BufferWatchError {
      err: format!("{err}"),
    })?;
  }
}
