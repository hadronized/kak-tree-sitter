use kak_tree_sitter_config::Config;
use mio::Token;

use crate::{
  error::OhNo,
  kakoune::{buffer::BufferId, selection::Sel, text_objects::OperationMode},
  protocol::response::{EnqueueResponse, Payload, Response},
  tree_sitter::{
    languages::{Language, Languages},
    nav,
    state::{TreeState, Trees},
  },
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
  with_indent_guidelines: bool,
}

impl Handler {
  pub fn new(
    config: &Config,
    with_highlighting: bool,
    with_indent_guidelines: bool,
  ) -> Result<Self, OhNo> {
    let trees = Trees::default();
    let langs = Languages::load_from_dir(config)?;

    Ok(Self {
      trees,
      langs,
      with_highlighting,
      with_indent_guidelines,
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
  pub fn handle_full_buffer_update(
    &mut self,
    enqueue_response: &EnqueueResponse,
    tkn: Token,
  ) -> Result<(), OhNo> {
    let id = self.trees.get_buf_id(&tkn)?.clone();
    log::debug!("updating {id:?}, token {tkn:?}");
    let tree = self.trees.get_tree_mut(&id)?;
    let lang = self.langs.get(tree.lang())?;

    // update the tree and early return if no update occurred
    if !tree.update_buf()? {
      return Ok(());
    }

    // run any additional post-processing on the buffer
    if self.with_highlighting {
      let resp = Self::handle_highlighting(&self.langs, lang, &id, tree)?;
      enqueue_response.enqueue(resp);
    }

    if self.with_indent_guidelines {
      let resp = Self::handle_indent_guidelines(&self.langs, lang, &id, tree)?;
      enqueue_response.enqueue(resp);
    }

    Ok(())
  }

  fn handle_highlighting(
    langs: &Languages,
    lang: &Language,
    id: &BufferId,
    tree: &mut TreeState,
  ) -> Result<Response, OhNo> {
    let ranges = tree.highlight(lang, |inject_lang| {
      langs.get(inject_lang).ok().map(|lang2| &lang2.hl_config)
    })?;

    Ok(Response::new(
      id.session(),
      None,
      id.buffer().to_owned(),
      Payload::Highlights { ranges },
    ))
  }

  fn handle_indent_guidelines(
    langs: &Languages,
    lang: &Language,
    id: &BufferId,
    tree: &mut TreeState,
  ) -> Result<Response, OhNo> {
    todo!()
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
