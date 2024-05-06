use std::{
  io,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use mio::{unix::SourceFd, Events, Interest, Poll, Token, Waker};

use crate::error::OhNo;

/// Token distribution.
#[derive(Debug)]
pub struct TokenProvider {
  // next available token
  next_token: Token,

  // available tokens (i.e. dead sessions’ tokens)
  free_tokens: Vec<Token>,
}

impl Default for TokenProvider {
  fn default() -> Self {
    Self {
      next_token: Self::CMD_FIFO_FIRST_TOKEN,
      free_tokens: Vec::default(),
    }
  }
}

impl TokenProvider {
  const WAKER_TOKEN: Token = Token(0);
  pub const UNIX_LISTENER_TOKEN: Token = Token(1);
  pub const CMD_FIFO_FIRST_TOKEN: Token = Token(2);

  /// Get a new token for a new session.
  pub fn create(&mut self) -> Token {
    self.free_tokens.pop().unwrap_or_else(|| {
      let token = self.next_token;
      self.next_token = Token(token.0 + 1);
      token
    })
  }

  pub fn recycle(&mut self, token: Token) {
    self.free_tokens.push(token);
  }
}

/// Poll-based event loop.
#[derive(Debug)]
pub struct EventLoop {
  poll: Poll,
  waker: Arc<Waker>,
  shutdown: Arc<AtomicBool>,
  events: Events,
}

impl EventLoop {
  pub fn new() -> Result<Self, OhNo> {
    let poll = Poll::new().map_err(|err| OhNo::CannotStartPoll { err })?;
    let waker = Arc::new(
      Waker::new(poll.registry(), TokenProvider::WAKER_TOKEN)
        .map_err(|err| OhNo::CannotStartServer { err })?,
    );

    // SIGINT handler; we just ask to shutdown the server
    let shutdown = Arc::new(AtomicBool::new(false));

    {
      let waker = waker.clone();
      let shutdown = shutdown.clone();
      ctrlc::set_handler(move || {
        log::warn!("received SIGINT");
        shutdown.store(true, Ordering::Relaxed);
        waker.wake().unwrap();
      })
      .map_err(|err| OhNo::SigIntHandlerError { err })?;
    }

    let events = Events::with_capacity(1024);

    Ok(Self {
      poll,
      waker,
      shutdown,
      events,
    })
  }

  pub fn stop(&self) {
    self.shutdown.store(true, Ordering::Relaxed);
  }

  pub fn register(&self, source: &mut SourceFd, token: Token) -> Result<(), OhNo> {
    self
      .poll
      .registry()
      .register(source, token, Interest::READABLE)
      .map_err(|err| OhNo::PollError { err })?;

    Ok(())
  }

  pub fn unregister(&self, source: &mut SourceFd) -> Result<(), OhNo> {
    self
      .poll
      .registry()
      .deregister(source)
      .map_err(|err| OhNo::PollError { err })?;

    Ok(())
  }

  pub fn waker(&self) -> Arc<Waker> {
    self.waker.clone()
  }

  pub fn run(&mut self) -> Result<&Self, OhNo> {
    log::debug!("waiting on poll…");
    if let Err(err) = self.poll.poll(&mut self.events, None) {
      if err.kind() == io::ErrorKind::Interrupted {
        log::warn!("mio interrupted");
      } else {
        return Err(OhNo::PollError { err });
      }
    }

    Ok(self)
  }

  pub fn events(&self) -> Await {
    if self.shutdown.load(Ordering::Relaxed) {
      return Await::Shutdown;
    }

    Await::Tokens(Tokens {
      events: &self.events,
    })
  }
}

/// Result of IO waiting.
#[derive(Debug)]
pub enum Await<'a> {
  /// Something needs to be read.
  Tokens(Tokens<'a>),

  /// The event loop has been shutdown.
  Shutdown,
}

#[derive(Debug)]
pub struct Tokens<'a> {
  events: &'a Events,
}

impl<'a> Tokens<'a> {
  pub fn into_iter(self) -> impl 'a + Iterator<Item = Token> {
    self
      .events
      .iter()
      .filter(|event| event.is_readable())
			// we do not expose the waker token
      .filter(|event| event.token() != TokenProvider::WAKER_TOKEN)
      .map(|event| event.token())
  }
}
