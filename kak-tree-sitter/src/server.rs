mod event_loop;
pub mod fifo;
mod handler;
pub mod request;
pub mod resources;
pub mod response;
mod tmpfs;

use std::{
  collections::HashSet,
  fs::{self, File},
  io::{Read, Write},
  os::fd::{AsRawFd, RawFd},
  path::{Path, PathBuf},
  process::{Command, Stdio},
  sync::mpsc::{channel, Receiver, Sender},
  thread::{spawn, JoinHandle},
};

use kak_tree_sitter_config::Config;
use mio::{net::UnixListener, unix::SourceFd, Token};

use crate::{
  cli::Cli,
  error::OhNo,
  kakoune::{
    buffer::BufferId,
    selection::Sel,
    session::{Session, SessionState, SessionTracker},
  },
  server::{event_loop::Await, request::Request},
};

use self::{
  event_loop::{EventLoop, TokenProvider},
  fifo::{Fifo, FifoKind},
  handler::Handler,
  request::UnixRequest,
  resources::ServerResources,
  response::{ConnectedResponse, Response},
};

/// Feedback provided after a request has finished. Mainly used to shutdown.
#[derive(Debug)]
pub enum Feedback {
  Ok,
  ShouldExit,
}

pub struct Server {
  resources: ServerResources,
  event_loop: EventLoop,
  _resp_queue_handle: JoinHandle<()>,
  resp_sender: Sender<ConnectedResponse>,
  unix_handler: UnixHandler,
  fifo_handler: FifoHandler,
  session_tracker: SessionTracker,
  token_provider: TokenProvider,
}

impl Server {
  pub fn new(config: &Config, cli: &Cli, resources: ServerResources) -> Result<Self, OhNo> {
    let event_loop = EventLoop::new()?;

    let (resp_queue, resp_sender) = ResponseQueue::new();
    let unix_handler = UnixHandler::new(
      cli.is_standalone(),
      cli.with_highlighting,
      resources.clone(),
      resources.socket_path(),
      resp_sender.clone(),
    )?;
    event_loop.register(
      &mut SourceFd(&unix_handler.as_raw_fd()),
      TokenProvider::UNIX_LISTENER_TOKEN,
    )?;

    let fifo_handler = FifoHandler::new(config, resp_sender.clone())?;
    let session_tracker = SessionTracker::default();
    let token_provider = TokenProvider::default();

    let _resp_queue_handle = resp_queue.run();

    Ok(Server {
      resources,
      event_loop,
      _resp_queue_handle,
      resp_sender,
      unix_handler,
      fifo_handler,
      session_tracker,
      token_provider,
    })
  }

  /// Prepare the server, daemonizing if required, and writing the PID file.
  pub fn prepare(&self, daemonize: bool) -> Result<(), OhNo> {
    let pid_file = self.resources.pid_path();

    if daemonize {
      // create stdout / stderr files
      let stdout_path = self.resources.runtime_dir.join("stdout.txt");
      let stderr_path = self.resources.runtime_dir.join("stderr.txt");
      let stdout = File::create(&stdout_path).map_err(|err| OhNo::CannotCreateFile {
        file: stdout_path,
        err,
      })?;
      let stderr = File::create(&stderr_path).map_err(|err| OhNo::CannotCreateFile {
        file: stderr_path,
        err,
      })?;

      daemonize::Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .pid_file(pid_file)
        .start()
        .map_err(|err| OhNo::CannotStartDaemon {
          err: err.to_string(),
        })?;
    } else {
      fs::write(&pid_file, format!("{}", std::process::id())).map_err(|err| {
        OhNo::CannotWriteFile {
          file: pid_file,
          err,
        }
      })?;
    }

    Ok(())
  }

  fn register_already_existing_sessions(&mut self) -> Result<(), OhNo> {
    let sessions = Self::get_running_sessions();

    // we use command FIFOs for that
    let fifo_dir = self.resources.runtime_dir.join("commands");
    for dentry in fifo_dir
      .read_dir()
      .map_err(|err| OhNo::CannotGetSessions {
        err: err.to_string(),
      })?
      .flatten()
    {
      if let Some(name) = dentry.file_name().to_str().map(str::to_owned) {
        // check that the session exists; if not, clean it up
        if !sessions.contains(&name) {
          self.cleanup_session_data(&name);
          continue;
        }

        if let Err(err) = self.unix_handler.process_req(
          &self.event_loop,
          &mut self.token_provider,
          &mut self.session_tracker,
          &mut self.fifo_handler,
          UnixRequest::RegisterSession {
            name: name.clone(),
            client: None,
          },
        ) {
          log::error!("cannot register already existing session '{name}': {err}");
        }
      }
    }

    Ok(())
  }

  fn get_running_sessions() -> HashSet<String> {
    log::debug!("getting running sessions via kak -l");

    Command::new("kak")
      .arg("-l")
      .output()
      .map(|output| {
        let output_str = String::from_utf8(output.stdout).unwrap_or_default();
        output_str.lines().map(|s| s.to_owned()).collect()
      })
      .unwrap_or_default()
  }

  /// Remove the runtime files of a session.
  fn cleanup_session_data(&mut self, session: &str) {
    log::warn!("removing session '{session}' data");

    let command_fifo = self
      .resources
      .runtime_dir
      .join(format!("commands/{session}"));
    if let Err(err) = std::fs::remove_file(&command_fifo) {
      log::warn!(
        "cannot remove command FIFO at {path}: {err}",
        path = command_fifo.display()
      );
    }

    let buf_fifo = self
      .resources
      .runtime_dir
      .join(format!("buffers/{session}"));
    if let Err(err) = std::fs::remove_file(&buf_fifo) {
      log::warn!(
        "cannot remove buffer FIFO at {path}: {err}",
        path = buf_fifo.display()
      );
    }
  }

  /// Start the server state and wait for events to be dispatched.
  pub fn start(mut self) -> Result<(), OhNo> {
    log::info!("starting server");

    // search for already existing sessions, and if so, register them ahead of time
    if let Err(err) = self.register_already_existing_sessions() {
      log::error!("error while registering already existing sessions: {err}");
    }

    loop {
      self.event_loop.run()?;
      match self.event_loop.events() {
        Await::Shutdown => break,
        Await::Tokens(tokens) => {
          for token in tokens.into_iter() {
            match token {
              TokenProvider::UNIX_LISTENER_TOKEN => {
                match self.unix_handler.accept(
                  &self.event_loop,
                  &mut self.token_provider,
                  &mut self.session_tracker,
                  &mut self.fifo_handler,
                ) {
                  Ok(Feedback::ShouldExit) => self.event_loop.stop(),

                  Err(err) => {
                    log::error!("{err}");
                  }

                  _ => (),
                }
              }

              _ => self.fifo_handler.accept(&mut self.session_tracker, token)?,
            }
          }
        }
      }
    }

    log::info!("shutting down");
    self.disconnect_sessions();

    Ok(())
  }

  /// Disconnect all sessions by sending them all a [`Response::Deinit`].
  fn disconnect_sessions(&self) {
    for session_name in self.session_tracker.sessions() {
      let conn_resp = ConnectedResponse::new(session_name, None, Response::Deinit);
      if let Err(err) = self.resp_sender.send(conn_resp) {
        log::error!("cannot send response: {err}");
      } else {
        log::info!("disconnected session {session_name}");
      }
    }
  }
}

#[derive(Debug)]
struct UnixHandler {
  is_standalone: bool,
  with_highlighting: bool,
  resources: ServerResources,
  unix_listener: UnixListener,
  resp_sender: Sender<ConnectedResponse>,
}

impl UnixHandler {
  fn new(
    is_standalone: bool,
    with_highlighting: bool,
    resources: ServerResources,
    socket_path: impl AsRef<Path>,
    resp_sender: Sender<ConnectedResponse>,
  ) -> Result<Self, OhNo> {
    let unix_listener =
      UnixListener::bind(socket_path).map_err(|err| OhNo::CannotStartServer { err })?;

    Ok(Self {
      is_standalone,
      with_highlighting,
      resources,
      unix_listener,
      resp_sender,
    })
  }

  pub fn as_raw_fd(&self) -> RawFd {
    self.unix_listener.as_raw_fd()
  }

  fn accept(
    &mut self,
    event_loop: &EventLoop,
    token_provider: &mut TokenProvider,
    session_tracker: &mut SessionTracker,
    fifo_handler: &mut FifoHandler,
  ) -> Result<Feedback, OhNo> {
    let (mut client, _) = self
      .unix_listener
      .accept()
      .map_err(|err| OhNo::UnixConnectionError { err })?;

    log::info!("client connected: {client:?}");

    // read the request and parse it
    let mut req_str = String::new();
    client
      .read_to_string(&mut req_str)
      .map_err(|err| OhNo::InvalidRequest {
        req: req_str.clone(),
        err: err.to_string(),
      })?;
    log::info!("UNIX socket request: {req_str}");

    let req =
      serde_json::from_str::<UnixRequest>(&req_str).map_err(|err| OhNo::InvalidRequest {
        req: req_str,
        err: err.to_string(),
      })?;

    self.process_req(
      event_loop,
      token_provider,
      session_tracker,
      fifo_handler,
      req,
    )
  }

  fn process_req(
    &mut self,
    event_loop: &EventLoop,
    token_provider: &mut TokenProvider,
    session_tracker: &mut SessionTracker,
    fifo_handler: &mut FifoHandler,
    req: UnixRequest,
  ) -> Result<Feedback, OhNo> {
    match req {
      UnixRequest::RegisterSession { name, client } => {
        log::info!("registering session {name}");

        let (cmd_fifo_path, buf_fifo_path) =
          self.track_session(event_loop, token_provider, session_tracker, &name)?;

        let resp = Response::Init {
          cmd_fifo_path,
          buf_fifo_path,
          with_highlighting: self.with_highlighting,
        };

        let conn_resp = ConnectedResponse::new(name, client, resp);
        if let Err(err) = self.resp_sender.send(conn_resp) {
          log::error!("cannot send response: {err}");
        }
      }

      UnixRequest::Reload => {
        log::info!("reloading configuration, grammars and queries");
        self.reload(fifo_handler);
      }

      UnixRequest::SessionExit { name } => {
        self.recycle_session(event_loop, session_tracker, token_provider, name)?;

        // only shutdown if were started with an initial session (non standalone)
        let feedback = if !self.is_standalone && session_tracker.is_empty() {
          log::info!("last session exited; stopping the server…");
          Feedback::ShouldExit
        } else {
          Feedback::Ok
        };

        return Ok(feedback);
      }

      UnixRequest::Shutdown => return Ok(Feedback::ShouldExit),
    }

    Ok(Feedback::Ok)
  }

  fn track_session(
    &mut self,
    event_loop: &EventLoop,
    token_provider: &mut TokenProvider,
    session_tracker: &mut SessionTracker,
    session_name: &str,
  ) -> Result<(PathBuf, PathBuf), OhNo> {
    let cmd_fifo =
      Fifo::open_nonblocking(&self.resources.runtime_dir, FifoKind::Cmd, session_name)?;
    let cmd_path = cmd_fifo.path().to_owned();
    let cmd_token = token_provider.create();

    let buf_fifo =
      Fifo::open_nonblocking(&self.resources.runtime_dir, FifoKind::Buf, session_name)?;
    let buf_path = buf_fifo.path().to_owned();
    let buf_token = token_provider.create();

    event_loop.register(&mut SourceFd(&cmd_fifo.as_raw_fd()), cmd_token)?;
    event_loop.register(&mut SourceFd(&buf_fifo.as_raw_fd()), buf_token)?;

    session_tracker.track(
      session_name,
      Session::new(session_name, cmd_token, buf_token),
      cmd_fifo,
      buf_fifo,
    );

    Ok((cmd_path, buf_path))
  }

  /// Recycle a session by removing the session from the session tracker and recycling the token in the token provider.
  fn recycle_session(
    &mut self,
    event_loop: &EventLoop,
    session_tracker: &mut SessionTracker,
    token_provider: &mut TokenProvider,
    session_name: impl AsRef<str>,
  ) -> Result<(), OhNo> {
    let session_name = session_name.as_ref();

    log::info!("recycling session {session_name}");
    if let Some((session, cmd_fifo, buf_fifo)) = session_tracker.untrack(session_name) {
      if let Some(cmd_fifo) = cmd_fifo {
        event_loop.unregister(&mut SourceFd(&cmd_fifo.as_raw_fd()))?;
      }

      if let Some(buf_fifo) = buf_fifo {
        event_loop.unregister(&mut SourceFd(&buf_fifo.as_raw_fd()))?;
      }

      token_provider.recycle(session.cmd_token());
      token_provider.recycle(session.buf_token());
    }

    Ok(())
  }

  fn reload(&mut self, fifo_handler: &mut FifoHandler) {
    let config = match Config::load_default_user() {
      Ok(config) => config,
      Err(err) => {
        log::error!("reloading config failed: {err}");
        return;
      }
    };

    match FifoHandler::new(&config, self.resp_sender.clone()) {
      Ok(new_fifo_handler) => *fifo_handler = new_fifo_handler,
      Err(err) => log::error!("refreshing grammars/queries failed: {err}"),
    }
  }
}

struct FifoHandler {
  handler: Handler,
  resp_sender: Sender<ConnectedResponse>,
}

impl FifoHandler {
  fn new(config: &Config, resp_sender: Sender<ConnectedResponse>) -> Result<Self, OhNo> {
    let handler = Handler::new(config)?;

    Ok(Self {
      handler,
      resp_sender,
    })
  }

  /// Dispatch FIFO reads.
  pub fn accept(&mut self, session_tracker: &mut SessionTracker, token: Token) -> Result<(), OhNo> {
    if let Some((session, fifo)) = session_tracker.by_token(token) {
      match fifo.kind() {
        FifoKind::Cmd => self.accept_cmd(session, fifo)?,
        FifoKind::Buf => self.accept_buf(session, fifo)?,
      }
    }

    Ok(())
  }

  fn accept_cmd(&mut self, session: &mut Session, fifo: &mut Fifo) -> Result<(), OhNo> {
    log::debug!(
      "reading command FIFO for session {session_name}…",
      session_name = session.name()
    );

    let buffer = fifo.read_all()?;

    log::info!("FIFO request: {buffer}");

    let req = serde_json::from_str::<Request>(buffer).map_err(|err| OhNo::InvalidRequest {
      req: buffer.to_owned(),
      err: err.to_string(),
    });

    match req {
      Ok(req) => match self.process_cmd(session, &req) {
        Ok(Some(resp)) => {
          let client = req.client_name();
          let conn_resp =
            ConnectedResponse::new(session.name(), client.map(|c| c.to_owned()), resp);

          if let Err(err) = self.resp_sender.send(conn_resp) {
            log::error!("failure while sending response: {err}");
          }
        }

        Err(err) => {
          log::error!("handling request failed: {err}");
        }

        _ => (),
      },

      Err(err) => {
        log::error!("malformed request: {err}");
      }
    }

    Ok(())
  }

  fn process_cmd(
    &mut self,
    session: &mut Session,
    req: &Request,
  ) -> Result<Option<Response>, OhNo> {
    match req {
      Request::TryEnableHighlight { lang, .. } => self
        .handler
        .handle_try_enable_highlight(session.name(), lang)
        .map(Option::Some),

      Request::Highlight {
        client,
        buffer,
        lang,
        timestamp,
      } => {
        // we do not send the highlight immediately; instead, we change the state machine
        *session.state_mut() = SessionState::HighlightingWaiting {
          client: client.clone(),
          buffer: buffer.clone(),
          lang: lang.clone(),
          timestamp: *timestamp,
        };

        Ok(None)
      }

      Request::TextObjects {
        client,
        buffer,
        lang,
        pattern,
        selections,
        mode,
      } => {
        let selections = Sel::parse_many(selections);
        *session.state_mut() = SessionState::TextObjectsWaiting {
          client: client.clone(),
          buffer: buffer.clone(),
          lang: lang.clone(),
          pattern: pattern.clone(),
          selections,
          mode: mode.clone(),
        };

        Ok(None)
      }

      Request::Nav {
        client,
        buffer,
        lang,
        selections,
        dir,
      } => {
        let selections = Sel::parse_many(selections);
        *session.state_mut() = SessionState::NavWaiting {
          client: client.clone(),
          buffer: buffer.clone(),
          lang: lang.clone(),
          selections,
          dir: *dir,
        };

        Ok(None)
      }
    }
  }

  fn accept_buf(&mut self, session: &mut Session, fifo: &mut Fifo) -> Result<(), OhNo> {
    log::debug!(
      "reading buffer FIFO for session {session_name}…",
      session_name = session.name()
    );

    let buffer = fifo.read_all()?;
    self.process_buf(session, buffer)
  }

  fn process_buf(&mut self, session: &mut Session, buf: &str) -> Result<(), OhNo> {
    match session.state() {
      SessionState::HighlightingWaiting {
        client,
        buffer,
        lang,
        timestamp,
      } => {
        let client = client.clone();
        let buffer_id = BufferId::new(session.name(), buffer);
        let resp = self
          .handler
          .handle_highlight(buffer_id, lang, *timestamp, buf);

        self.finish_cmd(session, Some(&client), resp);
      }

      SessionState::TextObjectsWaiting {
        client,
        buffer,
        lang,
        pattern,
        selections,
        mode,
      } => {
        let client = client.clone();
        let buffer_id = BufferId::new(session.name(), buffer);
        let resp = self
          .handler
          .handle_text_objects(buffer_id, lang, buf, pattern, selections, mode);

        self.finish_cmd(session, Some(&client), resp);
      }

      SessionState::NavWaiting {
        client,
        buffer,
        lang,
        selections,
        dir,
      } => {
        let client = client.clone();
        let buffer_id = BufferId::new(session.name(), buffer);
        let resp = self
          .handler
          .handle_nav(buffer_id, lang, buf, selections, *dir);

        self.finish_cmd(session, Some(&client), resp);
      }

      // keep this branch so that we have exhaustiveness
      SessionState::Idle => (),
    }

    Ok(())
  }

  /// Inspect the result of a command and eventually send a response back to the Kakoune session.
  fn finish_cmd(
    &mut self,
    session: &mut Session,
    client: Option<&str>,
    resp: Result<Response, OhNo>,
  ) {
    // switch back to idle, as we have read the FIFO
    session.state_mut().idle();

    match resp {
      Ok(resp) => {
        let conn_resp =
          ConnectedResponse::new(session.name().to_owned(), client.map(str::to_owned), resp);

        if let Err(err) = self.resp_sender.send(conn_resp) {
          log::error!("failure while sending response: {err}");
        }
      }

      Err(err) => {
        log::error!(
          "command failed for session {session_name}: {err}",
          session_name = session.name()
        );
      }
    }
  }
}

/// Response queue, responsible in sending responses to Kakoune session.
struct ResponseQueue {
  receiver: Receiver<ConnectedResponse>,
}

impl ResponseQueue {
  fn new() -> (Self, Sender<ConnectedResponse>) {
    let (sender, receiver) = channel();
    (Self { receiver }, sender)
  }

  /// Run the response queue by dequeuing connected responses as they arrive in a dedicated thread.
  fn run(self) -> JoinHandle<()> {
    spawn(move || {
      for conn_resp in self.receiver {
        Self::send(conn_resp);
      }
    })
  }

  fn send(conn_resp: ConnectedResponse) {
    let resp = conn_resp.resp.to_kak_cmd(conn_resp.client.as_deref());

    if let Some(data) = resp {
      if let Err(err) = Self::send_via_kak_p(&conn_resp.session, &data) {
        log::error!("error while sending connected response: {err}");
      }
    }
  }

  fn send_via_kak_p(session: &str, data: &str) -> Result<(), OhNo> {
    let mut child = std::process::Command::new("kak")
      .args(["-p", session])
      .stdin(Stdio::piped())
      .spawn()
      .map_err(|err| OhNo::CannotSendRequest {
        err: err.to_string(),
      })?;
    let child_stdin = child
      .stdin
      .as_mut()
      .ok_or_else(|| OhNo::CannotSendRequest {
        err: "cannot pipe data to kak -p".to_owned(),
      })?;

    child_stdin
      .write_all(data.as_bytes())
      .map_err(|err| OhNo::CannotSendRequest {
        err: err.to_string(),
      })?;

    child_stdin.flush().map_err(|err| OhNo::CannotSendRequest {
      err: err.to_string(),
    })?;

    child.wait().map_err(|err| OhNo::CannotSendRequest {
      err: format!("error while waiting on kak -p: {err}"),
    })?;
    Ok(())
  }
}
