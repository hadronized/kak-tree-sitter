//! Source UI.

use colored::Colorize;
use kak_tree_sitter_config::source::Source;

use super::section::Field;

pub fn source_field(source: &Source) -> Field {
  match source {
    Source::Local { path } => Field::kv("Source (path)".blue(), path.display().to_string().green()),
    Source::Git { url, pin } => Field::kv(
      "Source (git)".blue(),
      format!(
        "{} {}{}{}",
        url.green(),
        "(".black(),
        pin.cyan(),
        ")".black()
      ),
    ),
  }
}
