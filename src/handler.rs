use crate::{languages, request::Request};
use std::path::PathBuf;

pub type SessionName = String;
pub type BufferName = String;

/// Type responsible in handling requests.
///
/// This type is stateful, as requests might have side-effect (i.e. tree-sitter parsing generates trees that can be
/// reused, for instance).
#[derive(Debug)]
pub struct Handler {}

impl Handler {
  pub fn new() -> Self {
    Self {}
  }

  pub fn handle_request(&mut self, request: String) {
    // parse the request and dispatch
    match serde_json::from_str::<Request>(&request) {
      Ok(req) => match req {
        Request::Highlight {
          session_name,
          buffer_name,
          lang,
          path,
        } => self.handle_highlight_req(session_name, buffer_name, lang, path),
      },

      Err(err) => eprintln!("cannot parse request {request}: {err}"),
    }
  }

  fn handle_highlight_req(
    &mut self,
    session: String,
    buffer: String,
    lang_str: String,
    path: PathBuf,
  ) {
    if let Some(lang) = languages::get_lang(&lang_str) {
      // TODO: highlighting
    }
  }
}
