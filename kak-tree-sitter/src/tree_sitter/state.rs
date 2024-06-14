//! Tree-sitter state (i.e. highlighting, tree walking, etc.)

use std::collections::{hash_map::Entry, HashMap};

use mio::Token;
use tree_sitter::{Node, Parser, QueryCursor};

use crate::{
  error::OhNo,
  kakoune::{
    buffer::BufferId,
    selection::{ObjectFlags, Pos, Sel, SelectMode},
    text_objects::OperationMode,
  },
  server::{fifo::Fifo, resources::ServerResources},
};

use super::{highlighting::KakHighlightRange, languages::Language, nav};

/// Lang-keyed trees.
#[derive(Default)]
pub struct Trees {
  trees: HashMap<BufferId, TreeState>,
  by_token: HashMap<Token, BufferId>,
}

impl Trees {
  /// Create or update a tree.
  pub fn compute(
    &mut self,
    resources: &mut ServerResources,
    lang: &Language,
    id: &BufferId,
  ) -> Result<&mut TreeState, OhNo> {
    match self.trees.entry(id.clone()) {
      Entry::Occupied(entry) => {
        let tree = entry.into_mut();
        tree.change_lang(lang)?;
        Ok(tree)
      }

      Entry::Vacant(entry) => {
        let tree = TreeState::new(resources, lang)?;

        self.by_token.insert(*tree.fifo.token(), id.clone());
        Ok(entry.insert(tree))
      }
    }
  }

  pub fn get_tree(&self, id: &BufferId) -> Result<&TreeState, OhNo> {
    self
      .trees
      .get(id)
      .ok_or_else(|| OhNo::UnknownBuffer { id: id.clone() })
  }

  pub fn get_tree_mut(&mut self, id: &BufferId) -> Result<&mut TreeState, OhNo> {
    self
      .trees
      .get_mut(id)
      .ok_or_else(|| OhNo::UnknownBuffer { id: id.clone() })
  }

  pub fn get_buf_id(&mut self, tkn: &Token) -> Result<&BufferId, OhNo> {
    self
      .by_token
      .get(tkn)
      .ok_or_else(|| OhNo::UnknownToken { tkn: *tkn })
  }

  pub fn delete_tree(&mut self, id: &BufferId) {
    if let Some(tree) = self.trees.remove(id) {
      let tkn = tree.fifo.token();
      self.by_token.remove(tkn);
    }
  }
}

/// State around a tree.
///
/// A tree-sitter tree represents a parsed buffer in a given state. It can be walked with queries and updated.
pub struct TreeState {
  // this will be useful for #26
  parser: Parser,
  tree: tree_sitter::Tree,
  buf: String,
  lang: String,
  fifo: Fifo,

  // TODO: for now, we don’t support custom highligthing, and hence have to use tree-sitter-highlight; see
  // #26 for further information
  highlighter: tree_sitter_highlight::Highlighter,
}

impl TreeState {
  pub fn new(resources: &mut ServerResources, lang: &Language) -> Result<Self, OhNo> {
    let mut parser = Parser::new();
    parser.set_language(lang.lang())?;

    let tree = parser
      .parse("".as_bytes(), None)
      .ok_or(OhNo::CannotParseBuffer)?;
    let highlighter = tree_sitter_highlight::Highlighter::new();

    let fifo = resources.new_fifo()?;

    Ok(Self {
      parser,
      tree,
      buf: String::default(),
      lang: lang.name.clone(),
      fifo,
      highlighter,
    })
  }

  pub fn lang(&self) -> &str {
    &self.lang
  }

  pub fn fifo(&self) -> &Fifo {
    &self.fifo
  }

  pub fn change_lang(&mut self, lang: &Language) -> Result<(), OhNo> {
    lang.lang_name().clone_into(&mut self.lang);

    self.parser = Parser::new();
    self.parser.set_language(lang.lang())?;

    self.recompute_tree()
  }

  /// Read the associated FIFO, update the buffer and recompute the tree.
  ///
  /// Return `true` if the buffer was updated.
  pub fn update_buf(&mut self) -> Result<bool, OhNo> {
    if self.fifo.read_to_buf(&mut self.buf)? {
      self.recompute_tree()?;
      return Ok(true);
    }

    Ok(false)
  }

  fn recompute_tree(&mut self) -> Result<(), OhNo> {
    self.tree = self
      .parser
      .parse(self.buf.as_bytes(), None)
      .ok_or(OhNo::CannotParseBuffer)?;
    Ok(())
  }

  pub fn highlight<'a>(
    &'a mut self,
    lang: &'a Language,
    injection_callback: impl FnMut(&str) -> Option<&'a tree_sitter_highlight::HighlightConfiguration>
      + 'a,
  ) -> Result<Vec<KakHighlightRange>, OhNo> {
    let events = self
      .highlighter
      .highlight(
        &lang.hl_config,
        self.buf.as_bytes(),
        None,
        injection_callback,
      )
      .map_err(|err| OhNo::HighlightError {
        err: err.to_string(),
      })?;

    Ok(KakHighlightRange::from_iter(
      &self.buf,
      &lang.hl_names,
      events.flatten(),
    ))
  }

  /// Get the text-objects for the given pattern.
  ///
  /// This function takes in a list of selections and a mode of operation, and return new selections, depending on the
  /// mode.
  pub fn text_objects(
    &self,
    lang: &Language,
    pattern: &str,
    selections: &[Sel],
    mode: &OperationMode,
  ) -> Result<Vec<Sel>, OhNo> {
    // first, check whether the language supports text-objects, and also check whether it has the text-object type in
    // its capture names
    let query = lang
      .textobject_query
      .as_ref()
      .ok_or_else(|| OhNo::UnsupportedTextObjects)?;

    // get captures’ nodes for the given pattern; this is a function because the pattern might be dynamically recomputed
    // (e.g. object mode)
    let get_captures_nodes = |pattern| {
      let capture_index =
        query
          .capture_index_for_name(pattern)
          .ok_or(OhNo::UnknownTextObjectQuery {
            pattern: pattern.to_owned(),
          })?;
      let mut cursor = QueryCursor::new();
      let captures: Vec<_> = cursor
        .captures(query, self.tree.root_node(), self.buf.as_bytes())
        .flat_map(|(cm, _)| cm.captures.iter().cloned())
        .filter(|cq| cq.index == capture_index)
        .map(|c| c.node)
        .collect();
      <Result<_, OhNo>>::Ok(captures)
    };

    let sels = match mode {
      OperationMode::SearchNext => {
        let nodes = get_captures_nodes(pattern)?;
        selections
          .iter()
          .flat_map(|sel| Self::search_next_text_object(sel, nodes.iter().cloned()))
          .collect()
      }

      OperationMode::SearchPrev => {
        let nodes = get_captures_nodes(pattern)?;
        selections
          .iter()
          .flat_map(|sel| Self::search_prev_text_object(sel, nodes.iter().cloned()))
          .collect()
      }

      OperationMode::SearchExtendNext => {
        let nodes = get_captures_nodes(pattern)?;
        selections
          .iter()
          .flat_map(|sel| Self::search_extend_next_text_object(sel, nodes.iter().cloned()))
          .collect()
      }

      OperationMode::SearchExtendPrev => {
        let nodes = get_captures_nodes(pattern)?;
        selections
          .iter()
          .flat_map(|sel| Self::search_extend_prev_text_object(sel, nodes.iter().cloned()))
          .collect()
      }

      OperationMode::FindNext => {
        let nodes = get_captures_nodes(pattern)?;
        selections
          .iter()
          .flat_map(|sel| Self::find_text_object(sel, nodes.iter().cloned(), false))
          .collect()
      }

      OperationMode::FindPrev => {
        let nodes = get_captures_nodes(pattern)?;
        selections
          .iter()
          .flat_map(|sel| Self::find_text_object(sel, nodes.iter().cloned(), true))
          .collect()
      }

      OperationMode::ExtendNext => {
        let nodes = get_captures_nodes(pattern)?;
        selections
          .iter()
          .flat_map(|sel| Self::extend_text_object(sel, nodes.iter().cloned(), false))
          .collect()
      }

      OperationMode::ExtendPrev => {
        let nodes = get_captures_nodes(pattern)?;
        selections
          .iter()
          .flat_map(|sel| Self::extend_text_object(sel, nodes.iter().cloned(), true))
          .collect()
      }

      OperationMode::Object { mode, flags } => {
        let flags = ObjectFlags::parse_kak_str(flags);

        let pattern = format!(
          "{pattern}.{}",
          if flags.inner { "inside" } else { "around" }
        );
        let nodes = get_captures_nodes(&pattern)?;

        selections
          .iter()
          .flat_map(|sel| Self::object_text_object(sel, nodes.iter().cloned(), *mode, flags))
          .collect()
      }
    };

    Ok(sels)
  }

  /// Search the next text-object for a given selection.
  fn search_next_text_object<'a>(
    sel: &Sel,
    captures: impl Iterator<Item = Node<'a>>,
  ) -> Option<Sel> {
    let p = sel.anchor.max(sel.cursor);
    let node = Self::node_after(&p, captures)?;
    let start = Pos::from(node.start_position());
    let mut end = Pos::from(node.end_position());
    end.col -= 1;

    Some(sel.replace(&start, &end))
  }

  /// Search the prev text-object for a given selection.
  fn search_prev_text_object<'a>(
    sel: &Sel,
    captures: impl Iterator<Item = Node<'a>>,
  ) -> Option<Sel> {
    let p = sel.anchor.min(sel.cursor);
    let node = Self::node_before(&p, captures)?;
    let start = Pos::from(node.start_position());
    let mut end = Pos::from(node.end_position());
    end.col -= 1;

    Some(sel.replace(&start, &end))
  }

  /// Search-extend the next text-object for a given selection.
  fn search_extend_next_text_object<'a>(
    sel: &Sel,
    captures: impl Iterator<Item = Node<'a>>,
  ) -> Option<Sel> {
    let node = Self::node_after(&sel.cursor, captures)?;
    let cursor = Pos::from(node.start_position());

    Some(Sel {
      anchor: sel.anchor,
      cursor,
    })
  }

  /// Search extend the prev text-object for a given selection.
  fn search_extend_prev_text_object<'a>(
    sel: &Sel,
    captures: impl Iterator<Item = Node<'a>>,
  ) -> Option<Sel> {
    let node = Self::node_before(&sel.cursor, captures)?;
    let cursor = Pos::from(node.start_position());

    Some(Sel {
      anchor: sel.anchor,
      cursor,
    })
  }

  /// Find the next/prev text-object for a given selection.
  fn find_text_object<'a>(
    sel: &Sel,
    nodes: impl Iterator<Item = Node<'a>>,
    is_prev: bool,
  ) -> Option<Sel> {
    let node = if is_prev {
      Self::node_before(&sel.cursor, nodes)?
    } else {
      Self::node_after(&sel.cursor, nodes)?
    };
    let cursor = node.start_position().into();
    let anchor = sel.cursor;

    Some(Sel { anchor, cursor })
  }

  /// Extend onto the next/prev text-object for a given selection.
  fn extend_text_object<'a>(
    sel: &Sel,
    nodes: impl Iterator<Item = Node<'a>>,
    is_prev: bool,
  ) -> Option<Sel> {
    let node = if is_prev {
      Self::node_before(&sel.cursor, nodes)?
    } else {
      Self::node_after(&sel.cursor, nodes)?
    };
    let cursor = node.start_position().into();
    let anchor = sel.anchor;

    Some(Sel { anchor, cursor })
  }

  /// Object-mode text-objects.
  ///
  /// Object-mode is a special in Kakoune aggregating many features, allowing to match inner / whole objects. The
  /// tree-sitter version enhances the mode with all possible tree-sitter capture groups.
  fn object_text_object<'a>(
    sel: &Sel,
    nodes: impl Iterator<Item = Node<'a>>,
    mode: SelectMode,
    flags: ObjectFlags,
  ) -> Option<Sel> {
    let node = Self::narrowest_enclosing_node(&sel.cursor, nodes)?;

    match mode {
      // extend only moves the cursor
      SelectMode::Extend => {
        let anchor = sel.anchor;
        let cursor = if flags.to_begin {
          Pos::from(node.start_position())
        } else if flags.to_end {
          let mut p = Pos::from(node.end_position());
          p.col -= 1;
          p
        } else {
          return None;
        };

        Some(Sel { anchor, cursor })
      }

      SelectMode::Replace => {
        // brute force but eh it works lol
        if flags.to_begin && !flags.to_end {
          let anchor = sel.cursor;
          let cursor = Pos::from(node.start_position());
          Some(Sel { anchor, cursor })
        } else if !flags.to_begin && flags.to_end {
          let anchor = sel.cursor;
          let mut cursor = Pos::from(node.end_position());
          cursor.col -= 1;
          Some(Sel { anchor, cursor })
        } else if flags.to_begin && flags.to_end {
          let anchor = Pos::from(node.start_position());
          let mut cursor = Pos::from(node.end_position());
          cursor.col -= 1;
          Some(Sel { anchor, cursor })
        } else {
          None
        }
      }
    }
  }

  /// Get the next node after given position.
  fn node_after<'a>(p: &Pos, nodes: impl Iterator<Item = Node<'a>>) -> Option<Node<'a>> {
    // tree-sitter API here is HORRIBLE as it mutates in-place on Iterator::next(); we can’t collect();
    //
    // Related discussions:
    // - <https://github.com/tree-sitter/tree-sitter/issues/2265>
    // - <https://github.com/tree-sitter/tree-sitter/issues/608>
    let mut candidates = nodes
      .filter(|node| &Pos::from(node.start_position()) > p)
      .collect::<Vec<_>>();

    candidates.sort_by_key(|node| node.start_byte());
    candidates.first().cloned()
  }

  /// Get the previous node before a given position.
  fn node_before<'a>(p: &Pos, nodes: impl Iterator<Item = Node<'a>>) -> Option<Node<'a>> {
    // tree-sitter API here is HORRIBLE as it mutates in-place on Iterator::next(); we can’t collect();
    //
    // Related discussions:
    // - <https://github.com/tree-sitter/tree-sitter/issues/2265>
    // - <https://github.com/tree-sitter/tree-sitter/issues/608>
    let mut candidates = nodes
      .filter(|node| &Pos::from(node.start_position()) < p)
      .collect::<Vec<_>>();

    candidates.sort_by_key(|node| node.start_byte());
    candidates.last().cloned()
  }

  /// Get the narrowest enclosing node of a given position.
  fn narrowest_enclosing_node<'a>(
    p: &Pos,
    nodes: impl Iterator<Item = Node<'a>>,
  ) -> Option<Node<'a>> {
    // tree-sitter API here is HORRIBLE as it mutates in-place on Iterator::next(); we can’t collect();
    //
    // Related discussions:
    // - <https://github.com/tree-sitter/tree-sitter/issues/2265>
    // - <https://github.com/tree-sitter/tree-sitter/issues/608>
    let mut candidates = nodes
      .filter(|node| &Pos::from(node.start_position()) < p && &Pos::from(node.end_position()) > p)
      .collect::<Vec<_>>();

    candidates.sort_by_key(|node| node.start_byte());
    candidates.last().cloned()
  }

  /// Navigate the tree.
  ///
  /// This function will apply the direction on all selections, expanding or collapsing them. If a selection is not
  /// spanning on a node, the closet node is selected first, so that if you have the cursor and anchor at the same
  /// location and you want to select the next child, your cursor will expand to the whole nearest enclosing node first.
  pub fn nav_tree(&self, selections: &[Sel], dir: nav::Dir) -> Vec<Sel> {
    selections
      .iter()
      .map(|sel| {
        self
          .find_sel_node(sel)
          .and_then(|node| {
            // if our selection is not the same as the node, we pick the node
            if !sel.fully_selects(&node) {
              log::debug!("selection {sel:?} doesn’t fully select node {node:?}");
              return Some(node);
            }

            log::debug!("walking node {node:?} for dir {dir:?}");
            log::debug!("  parent: {:?}", node.parent());
            log::debug!("  1st child: {:?}", node.child(0));
            log::debug!("  next sibling: {:?}", node.next_sibling());

            let res = match dir {
              nav::Dir::Parent => node.parent(),
              nav::Dir::FirstChild => node.child(0),
              nav::Dir::LastChild => node
                .child_count()
                .checked_sub(1)
                .and_then(|i| node.child(i)),
              nav::Dir::FirstSibling => node.parent().and_then(|parent| parent.child(0)),
              nav::Dir::LastSibling => node.parent().and_then(|parent| {
                parent
                  .child_count()
                  .checked_sub(1)
                  .and_then(|i| parent.child(i))
              }),
              nav::Dir::PrevSibling { cousin } if cousin => {
                Self::find_prev_sibling_or_cousin(&node)
              }
              nav::Dir::NextSibling { cousin } if cousin => {
                Self::find_next_sibling_or_cousin(&node)
              }
              nav::Dir::PrevSibling { .. } => node.prev_sibling(),
              nav::Dir::NextSibling { .. } => node.next_sibling(),
            };

            log::debug!("navigated to node: {res:?}");
            res
          })
          .map(|node| sel.replace_with_node(&node))
          .unwrap_or_else(|| sel.clone())
      })
      .collect()
  }

  /// Find the node for a selection.
  fn find_sel_node(&self, sel: &Sel) -> Option<Node> {
    log::trace!("finding node for selection {sel:?}");

    let start = sel.anchor.min(sel.cursor);
    let mut end= sel.cursor.max(sel.anchor);
    end.col += 1; // Kakoune ranges are inclusive
    let node = self
      .tree
      .root_node()
      .descendant_for_point_range(start.into(), end.into());

    log::trace!("found node: {node:?}");

    node
  }

  /// Get the next sibiling or cousin.
  fn find_next_sibling_or_cousin<'a>(node: &Node<'a>) -> Option<Node<'a>> {
    node.next_sibling().or_else(|| {
      let parent = node.parent()?;
      let parent_sibling = parent.next_sibling()?;

      if parent_sibling.child_count() > 0 {
        parent_sibling.child(0)
      } else {
        None
      }
    })
  }

  /// Get the previous sibiling or cousin.
  fn find_prev_sibling_or_cousin<'a>(node: &Node<'a>) -> Option<Node<'a>> {
    node.prev_sibling().or_else(|| {
      let parent = node.parent()?;
      let parent_sibling = parent.prev_sibling()?;

      if parent_sibling.child_count() > 0 {
        parent_sibling.child(parent_sibling.child_count() - 1)
      } else {
        None
      }
    })
  }
}
