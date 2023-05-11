use kak_tree_sitter_config::Config;

use crate::{
  grammars::Grammars,
  highlighting::{BufferId, Highlighters},
  queries::Queries,
  request::{Request, RequestPayload},
  response::Response,
  session::KakSession,
};

use std::{
  collections::{HashMap, HashSet},
  fs,
  path::Path,
};

/// Type responsible in handling requests.
///
/// This type is stateful, as requests might have side-effect (i.e. tree-sitter parsing generates trees/highlighters
/// that can be reused, for instance).
pub struct Handler {
  /// Active sessions are sessions that are currently up and requesting tree-sitter.
  ///
  /// When a session quits, it should send a special request so that we can remove it from this set.
  active_sessions: HashSet<String>,

  /// Known grammars.
  grammars: Grammars,

  /// Per-language queries.
  queries: HashMap<String, Queries>,

  /// Map a highlighter to a [`BufferId`].
  highlighters: Highlighters,
}

impl Handler {
  pub fn new(config: &Config) -> Self {
    let grammars = Grammars::load_from_dir(&config.grammars.path).unwrap(); // FIXME: unwraaaaap
    let queries = Self::load_queries(&config.queries.path);

    Self {
      active_sessions: HashSet::new(),
      grammars,
      queries,
      highlighters: Highlighters::new(config.highlight.hl_names.clone()),
    }
  }

  // FIXME: so many unwrap()
  /// Load all the queries.
  fn load_queries(dir: &Path) -> HashMap<String, Queries> {
    if !dir.is_dir() {
      eprintln!("no query directory!");
      return HashMap::new();
    }

    fs::read_dir(dir)
      .unwrap()
      .flatten()
      .map(|dir| {
        let queries = Queries::load_from_dir(dir.path());
        (dir.file_name().to_str().unwrap().to_owned(), queries)
      })
      .collect()
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
            let supported = self.grammars.get(&lang).is_some();
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
    lang_str: String,
    timestamp: u64,
    read_fifo: &Path,
  ) -> Response {
    if let Some(lang) = self.grammars.get(&lang_str) {
      if let Some(queries) = self.queries.get(&lang_str) {
        self
          .highlighters
          .highlight(lang, queries, buffer_id, timestamp, read_fifo)
      } else {
        Response::status(format!("no highlight query for language {lang_str}"))
      }
    } else {
      Response::status(format!("unsupported language: {lang_str}"))
    }
  }
}
