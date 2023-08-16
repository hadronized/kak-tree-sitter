use std::fs;

use colored::Colorize;
use kak_tree_sitter_config::Config;

use crate::{
  error::OhNo,
  highlighting::{BufferId, Highlighters},
  languages::Languages,
  request::Request,
  response::Response,
  server::SessionFifo,
  session::KakSession,
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

  /// Handle the request and return an optional response to send back to Kakoune.
  pub fn handle_request(
    &mut self,
    session: &KakSession,
    session_fifo: &mut SessionFifo,
    req: &Request,
  ) -> Result<Response, OhNo> {
    match req {
      Request::TryEnableHighlight { lang, .. } => {
        eprintln!("try enable highlight for session {session:?}");

        let supported = self.langs.get(lang).is_some();

        if !supported {
          eprintln!("{}", format!("language {lang} is not supported").red());
        }

        Ok(Response::FiletypeSupported { supported })
      }

      Request::Highlight {
        buffer,
        lang,
        timestamp,
        ..
      } => {
        eprintln!(
          "highlight for session {session:?}, buffer {buffer}, lang {lang}, timestamp {timestamp}"
        );

        let buffer_id = BufferId::new(&session.session_name, buffer);

        // read the buffer content from the command FIFO; this is the law
        let payload = fs::read_to_string(session_fifo.buffer_fifo_path())?;

        Ok(self.handle_highlight_req(buffer_id, lang, *timestamp, &payload)?)
      }
    }
  }

  fn handle_highlight_req(
    &mut self,
    buffer_id: BufferId,
    lang_name: &str,
    timestamp: u64,
    source: &str,
  ) -> Result<Response, OhNo> {
    if let Some(lang) = self.langs.get(lang_name) {
      self
        .highlighters
        .highlight(lang, &self.langs, buffer_id, timestamp, source)
    } else {
      Ok(Response::status(format!(
        "unsupported language: {lang_name}"
      )))
    }
  }
}
