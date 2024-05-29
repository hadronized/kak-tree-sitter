use mio::Token;

/// Distribute tokens.
///
/// Tokens are provided by first trying to give recyled ones. If none is
/// remaining, new tokens are provided.
#[derive(Debug)]
pub struct Tokens {
  next: Token,
  free_list: Vec<Token>,
}

impl Default for Tokens {
  fn default() -> Self {
    Self {
      // 0 is for wake; 1 is for the UNIX listener; 2+ is for the rest
      next: Token(2),
      free_list: Vec::default(),
    }
  }
}

impl Tokens {
  pub fn create(&mut self) -> Token {
    self.free_list.pop().unwrap_or_else(|| {
      let t = self.next;
      self.next = Token(t.0 + 1);
      t
    })
  }

  pub fn recycle(&mut self, tkn: Token) {
    self.free_list.push(tkn);
  }
}
