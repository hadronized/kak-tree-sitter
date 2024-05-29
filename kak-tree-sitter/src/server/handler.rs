use kak_tree_sitter_config::Config;
use mio::Token;

use crate::{
  error::OhNo,
  kakoune::{buffer::BufferId, selection::Sel, text_objects::OperationMode},
  protocol::response::{Payload, Response},
  tree_sitter::{languages::Languages, nav, state::Trees},
};

use super::resources::ServerResources;

/// Type responsible for handling tree-sitter requests.
///
/// This type is stateful, as requests might have side-effect (i.e. tree-sitter
/// parsing generates trees/highlighters that can be reused, for instance).
pub struct Handler {
  trees: Trees,
  langs: Languages,
  with_highlighting: bool,
}

impl Handler {
  pub fn new(config: &Config, with_highlighting: bool) -> Result<Self, OhNo> {
    let trees = Trees::default();
    let langs = Languages::load_from_dir(config)?;

    Ok(Self {
      trees,
      langs,
      with_highlighting,
    })
  }

  /// Initiate languages on session init.
  pub fn handle_session_begin(&mut self) -> Payload {
    let enabled_langs = self
      .langs
      .langs()
      .map(|(name, lang)| (name.to_owned(), lang.remove_default_highlighter))
      .collect();
    Payload::Init { enabled_langs }
  }

  /// Update buffer metadata change.
  pub fn handle_buffer_metadata(
    &mut self,
    resources: &mut ServerResources,
    id: &BufferId,
    lang: &str,
  ) -> Result<Payload, OhNo> {
    let lang = self.langs.get(lang)?;
    let tree = self.trees.compute(resources, lang, id)?;
    let fifo = tree.fifo();
    let fifo_path = fifo.path().to_owned();
    let sentinel = fifo.sentinel().to_owned();

    Ok(Payload::BufferSetup {
      fifo_path,
      sentinel,
    })
  }

  /// Handle buffer close.
  pub fn handle_buffer_close(&mut self, id: &BufferId) {
    self.trees.delete_tree(id);
  }

  /// Update a full buffer update.
  pub fn handle_full_buffer_update(&mut self, tkn: Token) -> Result<Option<Response>, OhNo> {
    log::debug!("updating buffer (token = {tkn:?})");

    let id = self.trees.get_buf_id(&tkn)?.clone();
    log::info!("updating {id:?}");
    let tree = self.trees.get_tree_mut(&id)?;

    // update the tree
    if !tree.update_buf()? {
      // early return if no update occurred
      return Ok(None);
    }

    // run any additional post-processing on the buffer
    if !self.with_highlighting {
      return Ok(None);
    }

    // serve highlight
    let lang = self.langs.get(tree.lang())?;
    let ranges = tree.highlight(lang, |inject_lang| {
      self
        .langs
        .get(inject_lang)
        .ok()
        .map(|lang2| &lang2.hl_config)
    })?;

    let resp = Response::new(
      id.session(),
      None,
      id.buffer().to_owned(),
      Payload::Highlights { ranges },
    );

    Ok(Some(resp))
  }

  pub fn handle_text_objects(
    &mut self,
    id: &BufferId,
    pattern: &str,
    selections: &[Sel],
    mode: &OperationMode,
  ) -> Result<Payload, OhNo> {
    log::debug!("text-objects {pattern} for buffer {id:?}");

    let tree_state = self.trees.get_tree(id)?;
    let lang = self.langs.get(tree_state.lang())?;
    let sels = tree_state.text_objects(lang, pattern, selections, mode)?;

    Ok(Payload::Selections { sels })
  }

  pub fn handle_nav(
    &mut self,
    id: &BufferId,
    selections: &[Sel],
    dir: nav::Dir,
  ) -> Result<Payload, OhNo> {
    log::debug!("nav {dir:?} for buffer {id:?}");

    let tree_state = self.trees.get_tree(id)?;
    let sels = tree_state.nav_tree(selections, dir);

    Ok(Payload::Selections { sels })
  }
}
