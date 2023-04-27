use tree_sitter_highlight::{HighlightConfiguration, Highlighter};

use crate::{
  highlighting::KakHighlight,
  languages,
  request::{BufferId, Request},
  response::Response,
};
use std::{collections::HashMap, fs, path::PathBuf};

/// Default set of names to highlight.
const DEFAULT_HL_NAMES: &[&str] = &[
  "attribute",
  "constant",
  "function.builtin",
  "function",
  "keyword",
  "operator",
  "property",
  "punctuation",
  "punctuation.bracket",
  "punctuation.delimiter",
  "string",
  "string.special",
  "tag",
  "type",
  "type.builtin",
  "variable",
  "variable.builtin",
  "variable.parameter",
];

/// Type responsible in handling requests.
///
/// This type is stateful, as requests might have side-effect (i.e. tree-sitter parsing generates trees/highlighters
/// that can be reused, for instance).
pub struct Handler {
  highlighters: HashMap<BufferId, Highlighter>,
}

impl Handler {
  pub fn new() -> Self {
    Self {
      highlighters: HashMap::new(),
    }
  }

  /// Handle the request and return whether the handler should shutdown.
  pub fn handle_request(&mut self, request: String) -> Response {
    // parse the request and dispatch
    match serde_json::from_str::<Request>(&request) {
      Ok(req) => match req {
        Request::Shutdown => {
          return Response::status_changed("kak-tree-sitter: quit", true);
        }

        Request::Highlight {
          buffer_id,
          lang,
          path,
        } => self.handle_highlight_req(buffer_id, lang, path),
      },

      Err(err) => eprintln!("cannot parse request {request}: {err}"),
    }

    Response::status_changed("kak-tree-sitter: unknown request", false)
  }

  fn handle_highlight_req(&mut self, buffer_id: BufferId, lang_str: String, path: PathBuf) {
    // TODO: move that to the config
    if let Some((lang, hl_query)) = languages::get_lang_hl_query(&lang_str) {
      println!("parsing {buffer_id:?}");

      let source = fs::read_to_string(path).unwrap(); // FIXME: unwrap()

      let highlighter = self
        .highlighters
        .entry(buffer_id)
        .or_insert(Highlighter::new());
      // re-parse
      let mut hl_config = HighlightConfiguration::new(lang, hl_query, "", "").unwrap();
      hl_config.configure(DEFAULT_HL_NAMES); // FIXME: config

      let events = highlighter
        .highlight(&hl_config, source.as_bytes(), None, |_| None)
        .unwrap();

      let kak_hls = KakHighlight::from_iter(&source, DEFAULT_HL_NAMES, events.flatten());

      for hl in kak_hls {
        let s = hl.as_ranges_str();
        println!("{}", s);
      }
    }
  }
}
