//! rc file used by Kakoune to inject kak-tree-sitter commands.

pub fn static_kak() -> &'static str {
  include_str!("../rc/static.kak")
}
