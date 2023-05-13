use kak_tree_sitter_config::Config;

use crate::{
  highlighting::{BufferId, Highlighters},
  languages::Languages,
  request::{Request, RequestPayload},
  response::Response,
  session::KakSession,
};

use std::{collections::HashSet, path::Path};

/// Type responsible in handling requests.
///
/// This type is stateful, as requests might have side-effect (i.e. tree-sitter parsing generates trees/highlighters
/// that can be reused, for instance).
pub struct Handler {
  /// Active sessions are sessions that are currently up and requesting tree-sitter.
  ///
  /// When a session quits, it should send a special request so that we can remove it from this set.
  active_sessions: HashSet<String>,

  /// Known languages.
  langs: Languages,

  /// Map a highlighter to a [`BufferId`].
  highlighters: Highlighters,
}

impl Drop for Handler {
  fn drop(&mut self) {
    // drop queries and highlighters first before dropping grammars; otherwise we might trying to call tree-sitter code
    // that was unloaded
    drop(&mut self.highlighters);
    drop(&mut self.langs);

    // we don’t care about the order for the rest
  }
}

impl Handler {
  pub fn new(config: &Config) -> Self {
    let active_sessions = HashSet::new();
    let langs = Languages::load_from_dir(config);
    let highlighters = Highlighters::new(config.highlight.hl_names.clone());

    Self {
      active_sessions,
      langs,
      highlighters,
    }
  }

  /// Handle the request and return whether the handler should shutdown.
  pub fn handle_request(&mut self, request: String) -> Option<(KakSession, Response)> {
    // parse the request and dispatch
    match serde_json::from_str::<Request>(&request) {
      Ok(req) => {
        // mark the session as active
        if !self.active_sessions.contains(&req.session.session_name) {
          println!("new active session {}", req.session.session_name);

          self
            .active_sessions
            .insert(req.session.session_name.clone());
        }

        match req.payload {
          RequestPayload::SessionEnd => {
            println!("ending session {}", req.session.session_name);

            self.active_sessions.remove(&req.session.session_name);

            if self.active_sessions.is_empty() {
              return Some((req.session, Response::Shutdown));
            }
          }

          RequestPayload::Shutdown => {
            return Some((req.session, Response::Shutdown));
          }

          RequestPayload::TryEnableHighlight { lang } => {
            let supported = self.langs.get(&lang).is_some();
            return Some((req.session, Response::FiletypeSupported { supported }));
          }

          RequestPayload::Highlight {
            buffer,
            lang,
            timestamp,
            read_fifo,
          } => {
            let buffer_id = BufferId::new(&req.session.session_name, &buffer);
            let resp = self.handle_highlight_req(buffer_id, lang, timestamp, &read_fifo);
            return Some((req.session, resp));
          }
        }
      }

      Err(err) => {
        eprintln!("cannot parse request {request}: {err}");
      }
    }

    None
  }

  fn handle_highlight_req(
    &mut self,
    buffer_id: BufferId,
    lang_name: String,
    timestamp: u64,
    read_fifo: &Path,
  ) -> Response {
    if let Some(lang) = self.langs.get(&lang_name) {
      self
        .highlighters
        .highlight(lang, &self.langs, buffer_id, timestamp, read_fifo)
    } else {
      Response::status(format!("unsupported language: {lang_name}"))
    }
  }
}
