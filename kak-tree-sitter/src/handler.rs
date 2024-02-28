use std::collections::{hash_map::Entry, HashMap};

use kak_tree_sitter_config::Config;

use crate::{
  buffer::BufferId,
  error::OhNo,
  languages::{Language, Languages},
  response::Response,
  selection::Sel,
  text_objects,
  tree_sitter_state::TreeState,
};

/// Type responsible for handling requests.
///
/// This type is stateful, as requests might have side-effect (i.e. tree-sitter parsing generates trees/highlighters
/// that can be reused, for instance).
pub struct Handler {
  /// Tree-sitter trees associated with a [`BufferId`].
  trees: HashMap<BufferId, TreeState>,

  /// Known languages.
  langs: Languages,
}

impl Handler {
  pub fn new(config: &Config) -> Result<Self, OhNo> {
    let trees = HashMap::default();
    let langs = Languages::load_from_dir(config)?;

    Ok(Self { trees, langs })
  }

  /// Ensure we have a parsed tree for this buffer id and buffer content.
  fn compute_tree<'a>(
    trees: &'a mut HashMap<BufferId, TreeState>,
    lang: &Language,
    buffer_id: BufferId,
    buf: &str,
  ) -> Result<&'a mut TreeState, OhNo> {
    match trees.entry(buffer_id) {
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
    buffer_id: BufferId,
    lang_name: &str,
    timestamp: u64,
    buf: &str,
  ) -> Result<Response, OhNo> {
    log::debug!("highlight for buffer {buffer_id:?}, lang {lang_name}, timestamp {timestamp}");

    let Some(lang) = self.langs.get(lang_name) else {
      return Ok(Response::status(format!(
        "unsupported language: {lang_name}"
      )));
    };

    let tree_state = Self::compute_tree(&mut self.trees, lang, buffer_id, buf)?;

    let ranges = tree_state.highlight(lang, buf, |lang2| {
      self.langs.get(lang2).map(|lang2| &lang2.hl_config)
    })?;

    Ok(Response::Highlights { timestamp, ranges })
  }

  pub fn handle_text_objects(
    &mut self,
    buffer_id: BufferId,
    lang_name: &str,
    buf: &str,
    pattern: &str,
    selections: &[Sel],
    mode: &text_objects::OperationMode,
  ) -> Result<Response, OhNo> {
    log::debug!("text-objects {pattern} for buffer {buffer_id:?}, lang {lang_name}");

    let Some(lang) = self.langs.get(lang_name) else {
      return Ok(Response::status(format!(
        "unsupported language: {lang_name}"
      )));
    };

    let tree_state = Self::compute_tree(&mut self.trees, lang, buffer_id, buf)?;
    let sels = tree_state.text_objects(lang, buf, pattern, selections, mode)?;

    Ok(Response::Selections { sels })
  }
}
