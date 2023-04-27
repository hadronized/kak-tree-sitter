//! Response sent from the daemon to Kakoune, typically via the socket interface (kak -p, etc.).

/// Response sent by the daemon to Kakoune.
#[derive(Debug, Eq, PartialEq)]
pub enum Response {
  /// Status change.
  ///
  /// This response is emitted when the daemon connects or disconnects.
  StatusChanged(String),
}
