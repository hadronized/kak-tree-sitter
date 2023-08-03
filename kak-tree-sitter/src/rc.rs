//! rc file used by Kakoune to inject kak-tree-sitter commands.

pub fn rc_commands() -> &'static str {
  include_str!("../rc/kak-tree-sitter.kak")
}
