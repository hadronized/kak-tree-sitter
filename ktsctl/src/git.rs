//! Git utilities.

use std::{fs, path::Path};

use crate::{error::HellNo, process::Process, ui::report::Report};

/// Result of a successful git clone.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Clone {
  /// The repository was cloned remotely.
  Cloned,

  /// The repository was already cloned and thus a cached version is used.
  Cached,
}

/// Clone a git repository.
///
/// Return `Ok(true)` if something was cloned; `Ok(false)` if it was already there.
pub fn clone(report: &Report, fetch_path: &Path, url: &str) -> Result<Clone, HellNo> {
  // check if the fetch path already exists; if not, we clone the repository
  let fetched;
  if let Ok(false) = fetch_path.try_exists() {
    report.fetch(format!("cloning {url}"));

    fs::create_dir_all(fetch_path).map_err(|err| HellNo::CannotCreateDir {
      dir: fetch_path.to_owned(),
      err,
    })?;

    // shallow clone of the repository
    let git_clone_args = [
      "clone",
      "--depth",
      "1",
      "-n",
      url,
      fetch_path
        .as_os_str()
        .to_str()
        .ok_or_else(|| HellNo::BadPath)?,
    ];

    Process::new("git").run(None, &git_clone_args)?;
    fetched = Clone::Cloned;
  } else {
    fetched = Clone::Cached;
  }

  Ok(fetched)
}

/// Checkout a source at a given pin.
pub fn checkout(report: &Report, url: &str, fetch_path: &Path, pin: &str) -> Result<(), HellNo> {
  report.info(format!("checking out {url} at {pin}"));
  Process::new("git").run(fetch_path, &["checkout", pin])
}

/// Fetch remote git objects.
///
/// This function expects a `pin` to prevent fetching the whole remote repository.
pub fn fetch(
  report: &Report,
  lang: &str,
  fetch_path: &Path,
  url: &str,
  pin: &str,
) -> Result<(), HellNo> {
  report.sync(format!("fetching {lang} git remote objects {url}"));
  Process::new("git").run(fetch_path, &["fetch", "origin", "--prune", pin])?;
  checkout(report, url, fetch_path, pin)
}
