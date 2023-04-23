//! Requests that can be sent to the server from Kakoune.

#[derive(Debug)]
pub enum Request {
  /// Ask to highlight the given buffer.
  Highlight {
    buffer_name: String,
    lang: String,
    content: String,
  },
}
