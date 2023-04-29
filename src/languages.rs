use tree_sitter::Language;

pub fn get_lang(lang: &str) -> Option<Language> {
  match lang {
    "rust" => Some(tree_sitter_rust::language()),
    "toml" => Some(tree_sitter_toml::language()),
    _ => None,
  }
}
