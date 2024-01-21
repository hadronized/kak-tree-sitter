use std::collections::{hash_map::Entry, HashMap};

use kak_tree_sitter_config::Config;
use tree_sitter::Language;

use crate::{
  error::OhNo,
  highlighting::{BufferId, Highlighters},
  languages::Languages,
  response::Response,
  tree_sitter_state::TreeState,
};

/// Type responsible for handling requests.
///
/// This type is stateful, as requests might have side-effect (i.e. tree-sitter parsing generates trees/highlighters
/// that can be reused, for instance).
pub struct Handler {
  // TODO: should be removed and moved into `trees`
  /// Map a highlighter to a [`BufferId`].
  highlighters: Highlighters,

  /// Tree-sitter trees associated with a [`BufferId`].
  trees: HashMap<BufferId, TreeState>,

  /// Known languages.
  langs: Languages,
}

impl Handler {
  pub fn new(config: &Config) -> Result<Self, OhNo> {
    let highlighters = Highlighters::new();
    let trees = HashMap::default();
    let langs = Languages::load_from_dir(config)?;

    Ok(Self {
      highlighters,
      trees,
      langs,
    })
  }

  /// Ensure we have a parsed tree for this buffer id and buffer content.
  fn compute_tree<'a>(
    tree_states: &'a mut HashMap<BufferId, TreeState>,
    lang: Language,
    buffer_id: BufferId,
    buf: &str,
  ) -> Result<&'a mut TreeState, OhNo> {
    match tree_states.entry(buffer_id) {
      Entry::Vacant(entry) => {
        // first time we see this buffer; full parse
        let tree_state = TreeState::new(lang, buf)?;
        Ok(entry.insert(tree_state))
      }

      Entry::Occupied(mut entry) => {
        // TODO(#26): we already have a parsed buffer; we want an incremental update instead of fully reparsing everything
        let tree_state = TreeState::new(lang, buf)?;
        entry.insert(tree_state);
        Ok(entry.into_mut())
      }
    }
  }

  pub fn handle_try_enable_highlight(
    &mut self,
    session_name: impl AsRef<str>,
    lang_name: &str,
  ) -> Result<Response, OhNo> {
    let session_name = session_name.as_ref();

    log::info!("try enable highlight for language {lang_name}, session {session_name}");

    let lang = self.langs.get(lang_name);
    let supported = lang.is_some();
    let remove_default_highlighter = lang
      .map(|lang| lang.remove_default_highlighter)
      .unwrap_or_default();

    if !supported {
      log::warn!("language {lang_name} is not supported");
    }

    Ok(Response::FiletypeSupported {
      supported,
      remove_default_highlighter,
    })
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
      // HACK: current test with the new tree-sitter implementation; actually, I think we shouldn’t do that here; we
      // should require the buffer to be parsed in the first place
      let tree_state = Self::compute_tree(&mut self.trees, lang.lang(), buffer_id, buf)?;

      tree_state.query(&lang.hl_config.query, buf);
      log::info!("trying injections now…");
      Ok(Response::status("unimplemented"))

      //self
      //  .highlighters
      //  .highlight(lang, &self.langs, buffer_id, timestamp, buf)
    } else {
      Ok(Response::status(format!(
        "unsupported language: {lang_name}"
      )))
    }
  }
}
