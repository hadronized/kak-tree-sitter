pub mod fifo;
mod handler;
pub mod resources;
mod tokens;

use std::{
  io::Read,
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Receiver},
    Arc,
  },
  thread::{spawn, JoinHandle},
};

use kak_tree_sitter_config::Config;
use mio::{net::UnixListener, Events, Interest, Poll, Token, Waker};

use crate::{
  cli::Cli,
  error::OhNo,
  kakoune::{
    buffer::BufferId,
    selection::Sel,
    session::{Session, SessionTracker},
  },
  protocol::{
    request::{self, Request},
    response::{self, EnqueueResponse, Response},
  },
};

use self::{
  handler::Handler,
  resources::{Paths, ServerResources},
};

/// Feedback provided after a request has finished. Mainly used to shutdown.
#[derive(Debug)]
pub enum Feedback {
  Ok,
  ShouldExit,
}

pub struct Server {
  _resp_queue_handle: JoinHandle<()>,
  enqueue_response: EnqueueResponse,
  io_handler: IOHandler,
  session_tracker: SessionTracker,
}

impl Server {
  pub fn new(
    config: &Config,
    cli: &Cli,
    resources: ServerResources,
    poll: Poll,
  ) -> Result<Self, OhNo> {
    let (resp_queue, enqueue_response) = ResponseQueue::new();

    let session_tracker = SessionTracker::default();

    let unix_handler = IOHandler::new(
      config,
      cli.is_standalone(),
      cli.with_highlighting || config.features.highlighting,
      resources,
      poll,
      enqueue_response.clone(),
    )?;

    let _resp_queue_handle = resp_queue.run();

    Ok(Server {
      _resp_queue_handle,
      enqueue_response,
      io_handler: unix_handler,
      session_tracker,
    })
  }

  pub fn is_server_running(paths: &Paths) -> bool {
    match std::fs::read_to_string(paths.pid_path()) {
      Err(_) => false,
      Ok(pid) => ServerResources::is_running(pid.trim()),
    }
  }

  /// Initiate the first session, if any.
  ///
  /// It’s possible to start the server from within Kakoune. In that case, we
  /// need to simulate an init request from that session.
  pub fn init_first_session(&mut self, session: impl Into<String>) -> Result<(), OhNo> {
    let session = session.into();
    log::info!("initiating first session {session}");

    self
      .io_handler
      .process_req(&mut self.session_tracker, Request::init_session(session))?;

    Ok(())
  }

  /// Start the server state and wait for events to be dispatched.
  pub fn start(mut self) -> Result<(), OhNo> {
    log::info!("starting server");

    let quit = Arc::new(AtomicBool::new(false));
    let waker = self.io_handler.waker()?;

    {
      let quit = quit.clone();
      ctrlc::set_handler(move || {
        log::debug!("SIGINT received");
        quit.store(true, Ordering::Relaxed);

        if let Err(err) = waker.wake() {
          log::error!("cannot wake poll: {err}");
        }
      })?;
    }

    self
      .io_handler
      .start(&mut self.session_tracker, quit.clone());

    log::info!("shutting down");
    self.disconnect_sessions();

    Ok(())
  }

  /// Disconnect all sessions by sending them all a [`Response::Deinit`].
  fn disconnect_sessions(&self) {
    for session_name in self.session_tracker.sessions() {
      let resp = Response::new(session_name, None, None, response::Payload::Deinit);
      self.enqueue_response.enqueue(resp);
    }
  }
}

/// UNIX socket request handler.
///
/// This type is responsible for accepting UNIX socket connection, forwarding
/// the request to the appropriate handler and then sending back responses to
/// Kakoune via the response queue.
struct IOHandler {
  is_standalone: bool,
  with_highlighting: bool,
  resources: ServerResources,
  poll: Poll,
  unix_listener: UnixListener,
  enqueue_response: EnqueueResponse,
  handler: Handler,
}

impl IOHandler {
  const WAKE_TKN: Token = Token(0);
  const UNIX_LISTENER_TKN: Token = Token(1);

  fn new(
    config: &Config,
    is_standalone: bool,
    with_highlighting: bool,
    resources: ServerResources,
    poll: Poll,
    enqueue_response: EnqueueResponse,
  ) -> Result<Self, OhNo> {
    let mut unix_listener = UnixListener::bind(resources.paths().socket_path())
      .map_err(|err| OhNo::CannotStartServer { err })?;
    poll
      .registry()
      .register(
        &mut unix_listener,
        Self::UNIX_LISTENER_TKN,
        Interest::READABLE,
      )
      .map_err(|err| OhNo::PollError { err })?;

    let handler = Handler::new(config, with_highlighting)?;

    Ok(Self {
      is_standalone,
      with_highlighting,
      resources,
      poll,
      unix_listener,
      enqueue_response,
      handler,
    })
  }

  fn start(&mut self, session_tracker: &mut SessionTracker, quit: Arc<AtomicBool>) {
    let mut events = Events::with_capacity(64);

    'event_loop: while self.poll.poll(&mut events, None).is_ok() {
      if quit.load(Ordering::Relaxed) {
        break 'event_loop;
      }

      for ev in &events {
        match ev.token() {
          Self::UNIX_LISTENER_TKN if ev.is_readable() => {
            match self.unix_listener_accept(session_tracker) {
              Ok(Feedback::ShouldExit) => break 'event_loop,

              Err(err) => {
                log::error!("error during UNIX request: {err}");
              }

              _ => (),
            }
          }

          tkn => {
            log::debug!("FIFO readable (token = {tkn:?})");
            if let Err(err) = self.read_buffer(tkn) {
              log::error!("error while reading buffer (token = {tkn:?}): {err}");
            }
          }
        }
      }
    }

    log::debug!("poll loop exited");
  }

  pub fn waker(&self) -> Result<Arc<Waker>, OhNo> {
    let waker =
      Waker::new(self.poll.registry(), Self::WAKE_TKN).map_err(|err| OhNo::PollError { err })?;
    Ok(Arc::new(waker))
  }

  fn unix_listener_accept(
    &mut self,
    session_tracker: &mut SessionTracker,
  ) -> Result<Feedback, OhNo> {
    log::debug!("client connecting");
    let (mut client, _) = self
      .unix_listener
      .accept()
      .map_err(|err| OhNo::UnixSocketError { err })?;
    log::debug!("client connected: {client:?}");

    // read the request and parse it
    let mut req_str = String::new();
    client
      .read_to_string(&mut req_str)
      .map_err(|err| OhNo::UnixSocketError { err })?;
    log::debug!("UNIX socket request: {req_str}");

    let req = serde_json::from_str(&req_str).map_err(|err| OhNo::InvalidRequest {
      req: req_str,
      err: err.to_string(),
    })?;

    self.process_req(session_tracker, req)
  }

  fn process_req(
    &mut self,
    session_tracker: &mut SessionTracker,
    req: Request,
  ) -> Result<Feedback, OhNo> {
    match req.payload() {
      request::Payload::SessionBegin => {
        let session = req.session();
        if session_tracker.tracks(session) {
          log::warn!("session {session} already tracked");
          return Ok(Feedback::Ok);
        }

        log::info!("registering session {}", req.session());

        let session = Session::new(req.session())?;
        session_tracker.track(session);

        let resp_payload = self.handler.handle_session_begin();
        let resp = req.reply(resp_payload);

        self.enqueue_response.enqueue(resp);
      }

      request::Payload::SessionEnd => {
        log::info!("session {} exit", req.session());
        session_tracker.untrack(req.session());

        // only shutdown if were started with an initial session (non standalone)
        let feedback = if !self.is_standalone && session_tracker.is_empty() {
          log::info!("last session exited; stopping the server…");
          Feedback::ShouldExit
        } else {
          Feedback::Ok
        };

        return Ok(feedback);
      }

      request::Payload::Reload => {
        log::info!("reloading configuration, grammars and queries");
        self.reload();
      }

      request::Payload::Shutdown => {
        log::info!("shutting down");
        return Ok(Feedback::ShouldExit);
      }

      request::Payload::BufferMetadata { lang } => {
        let buffer = req.buffer().ok_or_else(|| OhNo::UnknownBuffer {
          id: BufferId::new(req.session(), String::new()),
        })?;

        log::info!("buffer metadata {buffer} ({lang})");
        let id = BufferId::new(req.session(), buffer);

        let resp_payload = self
          .handler
          .handle_buffer_metadata(&mut self.resources, &id, lang)?;
        self.enqueue_response.enqueue(req.reply(resp_payload));
      }

      request::Payload::BufferClose => {
        if let Some(buffer) = req.buffer() {
          log::info!("buffer close {buffer}");
          let id = BufferId::new(req.session(), buffer);
          self.handler.handle_buffer_close(&id);
        }
      }

      request::Payload::TextObjects {
        buffer,
        pattern,
        selections,
        mode,
      } => {
        log::info!("text objects for buffer {buffer}, pattern {pattern}, mode {mode:?}");

        let id = BufferId::new(req.session(), buffer);
        let sels = Sel::parse_many(selections);

        let resp_payload = self
          .handler
          .handle_text_objects(&id, pattern, &sels, mode)?;
        self.enqueue_response.enqueue(req.reply(resp_payload));
      }

      request::Payload::Nav {
        buffer,
        selections,
        dir,
      } => {
        log::info!("nav for buffer {buffer}, dir {dir:?}");

        let id = BufferId::new(req.session(), buffer);
        let sels = Sel::parse_many(selections);

        let resp_payload = self.handler.handle_nav(&id, &sels, *dir)?;
        self.enqueue_response.enqueue(req.reply(resp_payload));
      }
    }

    Ok(Feedback::Ok)
  }

  /// Read the buffer associated with the argument token.
  fn read_buffer(&mut self, tkn: Token) -> Result<(), OhNo> {
    if let Some(resp) = self.handler.handle_full_buffer_update(tkn)? {
      self.enqueue_response.enqueue(resp);
    }

    Ok(())
  }

  fn reload(&mut self) {
    let config = match Config::load_default_user() {
      Ok(config) => config,
      Err(err) => {
        log::error!("reloading config failed: {err}");
        return;
      }
    };

    match Handler::new(&config, self.with_highlighting) {
      Ok(new_handler) => self.handler = new_handler,
      Err(err) => log::error!("reloading failed: {err}"),
    }
  }
}

/// Response queue, responsible in sending responses to Kakoune session.
struct ResponseQueue {
  receiver: Receiver<Response>,
}

impl ResponseQueue {
  fn new() -> (Self, EnqueueResponse) {
    let (sender, receiver) = channel();
    (Self { receiver }, EnqueueResponse::new(sender))
  }

  /// Run the response queue by dequeuing connected responses as they arrive in a dedicated thread.
  fn run(self) -> JoinHandle<()> {
    spawn(move || {
      for resp in self.receiver {
        log::trace!("sending response: {resp:?}");

        if let Err(err) = Session::send_response(resp) {
          log::error!("error while sending connected response: {err}");
        }
      }
    })
  }
}
