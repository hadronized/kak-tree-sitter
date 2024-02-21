use std::collections::{hash_map::Entry, HashMap};

use kak_tree_sitter_config::Config;

use crate::{
  error::OhNo,
  highlighting::BufferId,
  kak,
  languages::{Language, Languages},
  response::Response,
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
    tree_states: &'a mut HashMap<BufferId, TreeState>,
    lang: &Language,
    buffer_id: BufferId,
    buf: &str,
  ) -> Result<&'a mut TreeState, OhNo> {
    let ts = match tree_states.entry(buffer_id) {
      Entry::Vacant(entry) => entry.insert(TreeState::new(lang)?),

      Entry::Occupied(mut entry) => {
        if entry.get().lang() != lang.lang() {
          entry.insert(TreeState::new(lang)?);
        }
        entry.into_mut()
      }
    };
    // Ignore errors here...
    let _ = ts.update(buf);
    Ok(ts)
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

    if let Some(lang) = self.langs.get(lang_name) {
      let tree_state = Self::compute_tree(&mut self.trees, &lang, buffer_id, buf)?;

      // tree_state.query(&lang.hl_config.query, buf);
      // log::info!("trying injections now…");
      // Ok(Response::status("unimplemented"));

      Ok(Response::Highlights {
        timestamp,
        ranges: tree_state.highlight(lang, &self.langs, buf)?,
      })
    } else {
      Ok(Response::status(format!(
        "unsupported language: {lang_name}"
      )))
    }
  }

  pub fn query_text_objects(
    &mut self,
    session_name: &str,
    buffer: &str,
    lang_name: &str,
    timestamp: u64,
    buf: &str,
    textobject_type: &str,
    inside: bool,
  ) -> Result<Vec<(kak::LocRange, usize)>, OhNo> {
    log::debug!(
      "text_objects for session {session_name}, buffer {buffer}, lang {lang_name}, timestamp {timestamp}"
    );
    let buffer_id = BufferId::new(session_name, buffer);
    let Some(lang) = self.langs.get(lang_name) else {
      return Err(OhNo::TextobjectError {
        err: format!("unsupported language: {lang_name}"),
      });
    };
    let Some(query) = lang.textobjects_query.as_ref() else {
      return Err(OhNo::TextobjectError {
        err: "language does not support textobjects".into(),
      });
    };

    let tree_state = Self::compute_tree(&mut self.trees, lang, buffer_id, buf)?;
    let mut cursor = tree_sitter::QueryCursor::new();
    let names = query.capture_names();
    let captures = cursor.captures(query, tree_state.tree().root_node(), buf.as_bytes());

    let capture_name = format!(
      "{textobject_type}.{}",
      if inside { "inside" } else { "around" }
    );

    let Some(name_idx) = query.capture_index_for_name(&capture_name) else {
      return Err(OhNo::TextobjectError {
        err: "language does not support this textobject".into(),
      });
    };

    // Iterator over all code ranges that match the textobject type
    let ranges = captures
      .filter_map(|(query_match, _size)| {
        let mut iter = query_match
          .captures
          .iter()
          .filter_map(|x| (x.index == name_idx).then_some(x.node));
        if let Some(first) = iter.next() {
          let last = iter.last().unwrap_or(first);
          let range = first.start_position()..last.end_position();
          let byte_len = first.start_byte().abs_diff(last.end_byte());
          Some((kak::LocRange::from(range), byte_len))
        } else {
          None
        }
      })
      .collect();
    Ok(ranges)
  }

  pub fn handle_text_objects(
    &mut self,
    session_name: &str,
    buffer: &str,
    lang_name: &str,
    timestamp: u64,
    buf: &str,
    textobject_type: &str,
    selections: impl Iterator<Item = kak::LocRange>,
    object_flags: &kak::ObjectFlags,
    select_mode: kak::SelectMode,
  ) -> Result<Response, OhNo> {
    let ranges = self.query_text_objects(
      session_name,
      buffer,
      lang_name,
      timestamp,
      buf,
      textobject_type,
      object_flags.inner,
    )?;

    // Objects that contain the given selection and are not equal to it. This lets us call the function repeatedly to expand the s
    let surrounding_textobject = move |selection| {
      let ranges = ranges
        .iter()
        .filter(|(range, _byte_len)| range.contains_range(&selection) && *range != selection);
      ranges
        .min_by_key(|(_range, byte_len)| *byte_len)
        .map(|x| x.0)
    };
    
    // We want the innermost object, i.e. the shortest one that contains the selection
    let ranges = selections
      .filter_map(|selection| {
        let range = surrounding_textobject(selection)?;
        let mut out_sel = if object_flags.to_begin && object_flags.to_end {
          range
        } else if object_flags.to_end {
          kak::LocRange::new(selection.start(), range.end())
        } else if object_flags.to_begin {
          kak::LocRange::new(selection.end(), range.start())
        } else {
          selection
        };
        if select_mode == kak::SelectMode::Extend {
          out_sel = out_sel.extend(selection)
        }
        Some(out_sel)
      })
      .collect();

    Ok(Response::Selections { timestamp, ranges })
  }

  pub fn handle_select(
    &mut self,
    session_name: &str,
    buffer: &str,
    lang_name: &str,
    timestamp: u64,
    buf: &str,
    textobject_type: &str,
    selections: impl Iterator<Item = kak::LocRange>,
  ) -> Result<Response, OhNo> {
    let ranges = self.query_text_objects(
      session_name,
      buffer,
      lang_name,
      timestamp,
      buf,
      textobject_type,
      false,
    )?;
    let mut res: Vec<kak::LocRange> = Vec::new();
    for selection in selections {
      for (r, _len) in &ranges {
        if selection.contains_range(r) && res.last().map(|x| !x.contains_range(r)).unwrap_or(true) {
          res.push(*r)
        }
      }
    }

    Ok(Response::Selections {
      timestamp,
      ranges: res,
    })
  }
}
