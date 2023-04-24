use tree_sitter::Language;

extern "C" {
  pub fn tree_sitter_rust() -> Language;
  pub fn tree_sitter_toml() -> Language;
}

pub fn get_lang(lang: &str) -> Option<Language> {
  unsafe {
    match lang {
      "rust" => Some(tree_sitter_rust()),
      "toml" => Some(tree_sitter_toml()),
      _ => None,
    }
  }
}
