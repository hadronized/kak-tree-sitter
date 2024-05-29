use serde::{Deserialize, Serialize};

/// A unique way to identify a buffer.
///
/// Currently tagged by the session name and the buffer name.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct BufferId {
  session: String,
  buffer: String,
}

impl BufferId {
  pub fn new(session: impl Into<String>, buffer: impl Into<String>) -> Self {
    Self {
      session: session.into(),
      buffer: buffer.into(),
    }
  }

  pub fn session(&self) -> &str {
    &self.session
  }

  pub fn buffer(&self) -> &str {
    &self.buffer
  }
}
