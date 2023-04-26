//! Configuration configuration.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
  highlight: HighlightConfig,
}

/// Configuration for highlighting.
///
/// Highlighting configuration consists of mapping tree-sitter node kind to faces.
#[derive(Debug, Deserialize, Serialize)]
pub struct HighlightConfig {
  kind_to_face_map: HashMap<String, String>,
}
