use colored::Colorize;
use kak_tree_sitter_config::Config;

use crate::{
  error::OhNo,
  highlighting::{BufferId, Highlighters},
  languages::Languages,
  response::Response,
};

/// Type responsible for handling requests.
///
/// This type is stateful, as requests might have side-effect (i.e. tree-sitter parsing generates trees/highlighters
/// that can be reused, for instance).
pub struct Handler {
  /// Map a highlighter to a [`BufferId`].
  highlighters: Highlighters,

  /// Known languages.
  langs: Languages,
}

impl Handler {
  pub fn new(config: &Config) -> Result<Self, OhNo> {
    let highlighters = Highlighters::new();
    let langs = Languages::load_from_dir(config)?;

    Ok(Self {
      highlighters,
      langs,
    })
  }

  pub fn handle_try_enable_highlight(
    &mut self,
    session_name: impl AsRef<str>,
    lang: &str,
  ) -> Result<Response, OhNo> {
    let session_name = session_name.as_ref();

    log::info!("try enable highlight for language {lang}, session {session_name}");

    let supported = self.langs.get(lang).is_some();

    if !supported {
      log::warn!("{}", format!("language {lang} is not supported").red());
    }

    Ok(Response::FiletypeSupported { supported })
  }

  pub fn handle_highlight(
    &mut self,
    session_name: &str,
    buffer: &str,
    lang_name: &str,
    timestamp: u64,
    buf: &str,
  ) -> Result<Response, OhNo> {
    log::debug!(
      "highlight for session {session_name}, buffer {buffer}, lang {lang_name}, timestamp {timestamp}"
    );

    let buffer_id = BufferId::new(session_name, buffer);
    self.handle_highlight_req(buffer_id, lang_name, timestamp, buf)
  }

  fn handle_highlight_req(
    &mut self,
    buffer_id: BufferId,
    lang_name: &str,
    timestamp: u64,
    buf: &str,
  ) -> Result<Response, OhNo> {
    if let Some(lang) = self.langs.get(lang_name) {
      self
        .highlighters
        .highlight(lang, &self.langs, buffer_id, timestamp, buf)
    } else {
      Ok(Response::status(format!(
        "unsupported language: {lang_name}"
      )))
    }
  }
}
