use kak_tree_sitter_config::Config;

use crate::{
  error::OhNo,
  highlighting::{BufferId, Highlighters},
  languages::Languages,
  request::{KakTreeSitterOrigin, Request, RequestPayload},
  response::Response,
  session::KakSession,
};

use std::collections::HashSet;

/// Type responsible in handling requests.
///
/// This type is stateful, as requests might have side-effect (i.e. tree-sitter parsing generates trees/highlighters
/// that can be reused, for instance).
pub struct Handler {
  /// Active sessions are sessions that are currently up and requesting tree-sitter.
  ///
  /// When a session quits, it should send a special request so that we can remove it from this set.
  active_sessions: HashSet<String>,

  /// Map a highlighter to a [`BufferId`].
  highlighters: Highlighters,

  /// Known languages.
  langs: Languages,
}

impl Handler {
  pub fn new(config: &Config) -> Result<Self, OhNo> {
    let active_sessions = HashSet::new();
    let highlighters = Highlighters::new();
    let langs = Languages::load_from_dir(config)?;

    Ok(Self {
      active_sessions,
      highlighters,
      langs,
    })
  }

  /// Handle the request and return whether the handler should shutdown.
  pub fn handle_request(
    &mut self,
    req: Request<KakTreeSitterOrigin>,
  ) -> Result<Option<(KakSession, Response)>, OhNo> {
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
          return Ok(Some((req.session, Response::Shutdown)));
        }
      }

      RequestPayload::Shutdown => {
        return Ok(Some((req.session, Response::Shutdown)));
      }

      RequestPayload::TryEnableHighlight { lang } => {
        let supported = self.langs.get(&lang).is_some();
        return Ok(Some((
          req.session,
          Response::FiletypeSupported { supported },
        )));
      }

      RequestPayload::Highlight {
        buffer,
        lang,
        timestamp,
        payload,
      } => {
        let buffer_id = BufferId::new(&req.session.session_name, buffer);
        return Ok(Some((
          req.session,
          self.handle_highlight_req(buffer_id, lang, timestamp, &payload)?,
        )));
      }
    }

    Ok(None)
  }

  fn handle_highlight_req(
    &mut self,
    buffer_id: BufferId,
    lang_name: String,
    timestamp: u64,
    source: &str,
  ) -> Result<Response, OhNo> {
    if let Some(lang) = self.langs.get(&lang_name) {
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
