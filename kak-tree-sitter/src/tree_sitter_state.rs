//! Tree-sitter state (i.e. highlighting, tree walking, etc.)

use tree_sitter::{Node, Parser, QueryCursor};

use crate::{
  error::OhNo,
  highlighting::KakHighlightRange,
  languages::Language,
  nav::Dir,
  selection::{ObjectFlags, Pos, Sel, SelectMode},
  text_objects::OperationMode,
};

/// State around a tree.
///
/// A tree-sitter tree represents a parsed buffer in a given state. It can be walked with queries and updated.
pub struct TreeState {
  // this will be useful for #26
  _parser: Parser,
  tree: tree_sitter::Tree,

  // TODO: for now, we don’t support custom highligthing, and hence have to use tree-sitter-highlight; see
  // #26 for further information
  highlighter: tree_sitter_highlight::Highlighter,
}

impl TreeState {
  pub fn new(lang: &Language, buf: &str) -> Result<Self, OhNo> {
    let mut parser = Parser::new();
    parser.set_language(lang.lang())?;

    let tree = parser
      .parse(buf.as_bytes(), None)
      .ok_or(OhNo::CannotParseBuffer)?;

    let highlighter = tree_sitter_highlight::Highlighter::new();

    Ok(Self {
      _parser: parser,
      tree,
      highlighter,
    })
  }

  pub fn highlight<'a>(
    &'a mut self,
    lang: &'a Language,
    buf: &'a str,
    injection_callback: impl FnMut(&str) -> Option<&'a tree_sitter_highlight::HighlightConfiguration>
      + 'a,
  ) -> Result<Vec<KakHighlightRange>, OhNo> {
    let events = self
      .highlighter
      .highlight(&lang.hl_config, buf.as_bytes(), None, injection_callback)
      .map_err(|err| OhNo::HighlightError {
        err: err.to_string(),
      })?;

    Ok(KakHighlightRange::from_iter(
      buf,
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
    buf: &str,
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
        .captures(query, self.tree.root_node(), buf.as_bytes())
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
  pub fn nav_tree(&self, selections: &[Sel], dir: Dir) -> Vec<Sel> {
    selections
      .iter()
      .map(|sel| {
        self
          .find_sel_node(sel)
          .and_then(|node| {
            // if we are fused, we select the nearest node
            if sel.is_fused() {
              return Some(node);
            }

            log::debug!("walking node {node:?} for dir {dir:?}");
            log::debug!("  parent: {:?}", node.parent());
            log::debug!("  1st child: {:?}", node.child(0));
            log::debug!("  next sibling: {:?}", node.next_sibling());

            let res = match dir {
              Dir::Parent => node.parent(),
              Dir::FirstChild => node.child(0),
              Dir::FirstSibling => node.parent().and_then(|node| node.child(0)),
              Dir::PrevSibling => node.prev_sibling(),
              Dir::NextSibling => node.next_sibling(),
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

    // TODO: this is wrong too; the anchor can be after the cursor
    let node = self
      .tree
      .root_node()
      .descendant_for_point_range(sel.anchor.into(), sel.cursor.into());

    log::trace!("found node: {node:?}");

    node
  }
}
