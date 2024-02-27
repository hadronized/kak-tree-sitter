//! rc file used by Kakoune to inject kak-tree-sitter commands.

/// Main RC file.
pub fn static_kak() -> &'static str {
  include_str!("../rc/static.kak")
}

/// Text-objects related file.
pub fn text_objects_kak() -> &'static str {
  include_str!("../rc/text-objects.kak")
}
