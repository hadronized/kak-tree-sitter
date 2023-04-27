use tree_sitter::Language;

pub fn get_lang_hl_query(lang: &str) -> Option<(Language, &'static str)> {
  match lang {
    "rust" => Some((
      tree_sitter_rust::language(),
      tree_sitter_rust::HIGHLIGHT_QUERY,
    )),
    "toml" => Some((
      tree_sitter_toml::language(),
      tree_sitter_toml::HIGHLIGHT_QUERY,
    )),
    _ => None,
  }
}
