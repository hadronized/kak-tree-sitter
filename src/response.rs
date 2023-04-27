//! Response sent from the daemon to Kakoune, typically via the socket interface (kak -p, etc.).

/// Response sent by the daemon to Kakoune.
#[derive(Debug, Eq, PartialEq)]
pub enum Response {
  /// Status change.
  ///
  /// This response is emitted when the daemon connects or disconnects.
  StatusChanged { status: String, shutdown: bool },
}

impl Response {
  pub fn should_shutdown(&self) -> bool {
    match self {
      Response::StatusChanged { shutdown, .. } => *shutdown,
    }
  }

  pub fn status_changed(status: impl Into<String>, shutdown: bool) -> Self {
    Response::StatusChanged {
      status: status.into(),
      shutdown,
    }
  }
}
