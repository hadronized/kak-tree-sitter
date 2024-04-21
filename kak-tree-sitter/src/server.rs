use std::{
  collections::HashSet,
  ffi::CString,
  fs::{self, File, OpenOptions},
  io::{self, Read, Write},
  os::{
    fd::AsRawFd,
    unix::{
      net::UnixStream,
      prelude::{OpenOptionsExt, OsStrExt},
    },
  },
  path::{Path, PathBuf},
  process::{Command, Stdio},
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Receiver, Sender},
    Arc,
  },
  thread::{spawn, JoinHandle},
};

use kak_tree_sitter_config::Config;
use mio::{net::UnixListener, unix::SourceFd, Events, Interest, Poll, Token, Waker};

use crate::{
  buffer::BufferId,
  cli::Cli,
  error::OhNo,
  handler::Handler,
  request::{Request, UnixRequest},
  response::{ConnectedResponse, Response},
  selection::Sel,
  session::{Fifo, Session, SessionState, SessionTracker},
};

/// Feedback provided after a request has finished. Mainly used to shutdown.
#[derive(Debug)]
pub enum Feedback {
  Ok,
  ShouldExit,
}

pub struct Server {
  server_state: ServerState,
}

impl Server {
  fn new(config: &Config, is_standalone: bool, with_highlighting: bool) -> Result<Self, OhNo> {
    let server_state = ServerState::new(config, is_standalone, with_highlighting)?;
    Ok(Self { server_state })
  }

  /// Bootstrap the server from the `config` and `cli`.
  pub fn bootstrap(config: &Config, cli: &Cli) -> Result<(), OhNo> {
    // find a runtime directory to write in
    let runtime_dir = ServerState::runtime_dir()?;
    log::info!("running in {}", runtime_dir.display());

    let pid_file = runtime_dir.join("pid");

    // check whether a pid file exists and can be read
    if let Ok(pid) = std::fs::read_to_string(&pid_file) {
      let pid = pid.trim();
      log::debug!("checking whether PID {pid} is still up…");

      // if the contained pid corresponds to a running process, stop right away
      // otherwise, remove the previous PID and socket files
      if Command::new("ps")
        .args(["-p", pid])
        .output()
        .is_ok_and(|o| o.status.success())
      {
        log::debug!("kak-tree-sitter already running; not starting a new server");
        return Ok(());
      } else {
        log::debug!("removing previous PID file");
        std::fs::remove_file(&pid_file).map_err(|err| OhNo::CannotStartDaemon {
          err: format!(
            "cannot remove previous PID file {path}: {err}",
            path = pid_file.display()
          ),
        })?;

        log::debug!("removing previous socket file");
        let socket_file = runtime_dir.join("socket");
        std::fs::remove_file(&socket_file).map_err(|err| OhNo::CannotStartDaemon {
          err: format!(
            "cannot remove previous socket file {path}: {err}",
            path = socket_file.display()
          ),
        })?;
      }
    }

    // ensure that the runtime directory exists, along with commands and buffers subdirectory
    let commands_dir = runtime_dir.join("commands");
    fs::create_dir_all(&commands_dir).map_err(|err| OhNo::CannotCreateDir {
      dir: commands_dir,
      err,
    })?;

    let buffers_dir = runtime_dir.join("buffers");
    fs::create_dir_all(&buffers_dir).map_err(|err| OhNo::CannotCreateDir {
      dir: buffers_dir,
      err,
    })?;

    if cli.daemonize {
      // create stdout / stderr files
      let stdout_path = runtime_dir.join("stdout.txt");
      let stderr_path = runtime_dir.join("stderr.txt");
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

    Server::new(config, !cli.kakoune, cli.with_highlighting)?.start()?;

    Ok(())
  }

  fn start(mut self) -> Result<(), OhNo> {
    // search for already existing sessions, and if so, register them ahead of time
    if let Err(err) = self.server_state.register_already_existing_sessions() {
      log::error!("error while registering already existing sessions: {err}");
    }

    self.server_state.start()
  }

  pub fn send_request(req: UnixRequest) -> Result<(), OhNo> {
    // serialize the request
    let serialized = serde_json::to_string(&req).map_err(|err| OhNo::CannotSendRequest {
      err: err.to_string(),
    })?;

    log::debug!("sending request {req:?}");

    // connect and send the request to the daemon
    UnixStream::connect(ServerState::runtime_dir()?.join("socket"))
      .map_err(|err| OhNo::CannotConnectToServer { err })?
      .write_all(serialized.as_bytes())
      .map_err(|err| OhNo::CannotSendRequest {
        err: err.to_string(),
      })
  }
}

/// Resources requiring a special drop implementation.
#[derive(Clone, Debug)]
pub struct ServerResources {
  pub runtime_dir: PathBuf,
}

impl ServerResources {
  fn new(runtime_dir: PathBuf) -> Self {
    Self { runtime_dir }
  }
}

impl Drop for ServerResources {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(self.runtime_dir.join("pid"));
  }
}

/// Token distribution.
#[derive(Debug)]
struct TokenProvider {
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
  const UNIX_LISTENER_TOKEN: Token = Token(1);
  const CMD_FIFO_FIRST_TOKEN: Token = Token(2);

  /// Get a new token for a new session.
  fn create(&mut self) -> Token {
    self.free_tokens.pop().unwrap_or_else(|| {
      let token = self.next_token;
      self.next_token = Token(token.0 + 1);
      token
    })
  }

  fn recycle(&mut self, token: Token) {
    self.free_tokens.push(token);
  }
}

pub struct ServerState {
  resources: ServerResources,
  poll: Poll,
  _resp_queue_handle: JoinHandle<()>,
  resp_sender: Sender<ConnectedResponse>,
  unix_handler: UnixHandler,
  fifo_handler: FifoHandler,
  shutdown: Arc<AtomicBool>,
  session_tracker: SessionTracker,
  token_provider: TokenProvider,
}

impl ServerState {
  pub fn new(config: &Config, is_standalone: bool, with_highlighting: bool) -> Result<Self, OhNo> {
    let resources = ServerResources::new(Self::runtime_dir()?);
    let mut poll = Poll::new().map_err(|err| OhNo::CannotStartPoll { err })?;
    let waker = Arc::new(
      Waker::new(poll.registry(), TokenProvider::WAKER_TOKEN)
        .map_err(|err| OhNo::CannotStartServer { err })?,
    );
    let (resp_queue, resp_sender) = ResponseQueue::new();
    let mut unix_handler = UnixHandler::new(
      is_standalone,
      with_highlighting,
      resources.clone(),
      ServerState::socket_path()?,
      resp_sender.clone(),
    )?;
    let fifo_handler = FifoHandler::new(config, resp_sender.clone())?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let session_tracker = SessionTracker::default();
    let token_provider = TokenProvider::default();

    // SIGINT handler; we just ask to shutdown the server
    {
      let shutdown = shutdown.clone();
      ctrlc::set_handler(move || {
        log::warn!("received SIGINT");
        shutdown.store(true, Ordering::Relaxed);
        waker.wake().unwrap();
      })
      .map_err(|err| OhNo::SigIntHandlerError { err })?;
    }

    unix_handler.register_poll(&mut poll)?;

    let _resp_queue_handle = resp_queue.run();

    Ok(ServerState {
      resources,
      poll,
      _resp_queue_handle,
      resp_sender,
      unix_handler,
      fifo_handler,
      shutdown,
      session_tracker,
      token_provider,
    })
  }

  pub fn runtime_dir() -> Result<PathBuf, OhNo> {
    let dir = dirs::runtime_dir()
      .or_else(||
        // macOS doesn’t implement XDG, yay…
        std::env::var("TMPDIR").map(PathBuf::from).ok())
      .ok_or_else(|| OhNo::NoRuntimeDir)?;
    Ok(dir.join("kak-tree-sitter"))
  }

  pub fn socket_path() -> Result<PathBuf, OhNo> {
    Ok(Self::runtime_dir()?.join("socket"))
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
          &mut self.poll,
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
  pub fn start(&mut self) -> Result<(), OhNo> {
    log::info!("starting server");

    let mut events = Events::with_capacity(1024);
    loop {
      if self.shutdown.load(Ordering::Relaxed) {
        break;
      }

      log::debug!("waiting on poll…");
      if let Err(err) = self.poll.poll(&mut events, None) {
        if err.kind() == io::ErrorKind::Interrupted {
          log::warn!("mio interrupted");
        } else {
          return Err(OhNo::PollError { err });
        }
      }

      for event in &events {
        log::trace!("mio event: {event:#?}");

        match event.token() {
          TokenProvider::WAKER_TOKEN => {
            log::debug!("waking up mio poll before shutting down");
            break;
          }

          TokenProvider::UNIX_LISTENER_TOKEN if event.is_readable() => {
            match self.unix_handler.accept(
              &mut self.poll,
              &mut self.token_provider,
              &mut self.session_tracker,
              &mut self.fifo_handler,
            ) {
              Ok(Feedback::ShouldExit) => self.shutdown.store(true, Ordering::Relaxed),

              Err(err) => {
                log::error!("{err}");
              }

              _ => (),
            }
          }

          tkn if event.is_readable() => self.fifo_handler.accept(&mut self.session_tracker, tkn)?,

          _ => (),
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

  /// Register the Unix handler to a Poll.
  fn register_poll(&mut self, poll: &mut Poll) -> Result<(), OhNo> {
    poll
      .registry()
      .register(
        &mut self.unix_listener,
        TokenProvider::UNIX_LISTENER_TOKEN,
        Interest::READABLE,
      )
      .map_err(|err| OhNo::CannotStartPoll { err })
  }

  fn accept(
    &mut self,
    poll: &mut Poll,
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

    self.process_req(poll, token_provider, session_tracker, fifo_handler, req)
  }

  fn process_req(
    &mut self,
    poll: &mut Poll,
    token_provider: &mut TokenProvider,
    session_tracker: &mut SessionTracker,
    fifo_handler: &mut FifoHandler,
    req: UnixRequest,
  ) -> Result<Feedback, OhNo> {
    match req {
      UnixRequest::RegisterSession { name, client } => {
        log::info!("registering session {name}");

        let (cmd_fifo_path, buf_fifo_path) =
          self.track_session(poll, token_provider, session_tracker, name.clone())?;

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
        self.recycle_session(poll, session_tracker, token_provider, name)?;

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
    poll: &mut Poll,
    token_provider: &mut TokenProvider,
    session_tracker: &mut SessionTracker,
    session_name: impl Into<String>,
  ) -> Result<(PathBuf, PathBuf), OhNo> {
    let session_name = session_name.into();

    let cmd_fifo_path = Self::create_fifo(&self.resources, Path::new("commands"), &session_name)?;
    let cmd_fifo_file = Self::open_nonblocking_fifo(&cmd_fifo_path)?;
    let cmd_token = token_provider.create();

    let buf_fifo_path = Self::create_fifo(&self.resources, Path::new("buffers"), &session_name)?;
    let buf_fifo_file = Self::open_nonblocking_fifo(&buf_fifo_path)?;
    let buf_token = token_provider.create();

    let registry = poll.registry();
    registry
      .register(
        &mut SourceFd(&cmd_fifo_file.as_raw_fd()),
        cmd_token,
        Interest::READABLE,
      )
      .map_err(|err| OhNo::PollError { err })?;
    registry
      .register(
        &mut SourceFd(&buf_fifo_file.as_raw_fd()),
        buf_token,
        Interest::READABLE,
      )
      .map_err(|err| OhNo::PollError { err })?;

    session_tracker.track(
      session_name.clone(),
      Session::new(session_name.clone(), cmd_token, buf_token),
      Fifo::Cmd {
        session_name: session_name.clone(),
        file: cmd_fifo_file,
        buffer: String::new(),
      },
      Fifo::Buf {
        session_name,
        file: buf_fifo_file,
        buffer: String::new(),
      },
    );

    Ok((cmd_fifo_path, buf_fifo_path))
  }

  fn create_fifo(
    resources: &ServerResources,
    prefix: &Path,
    session_name: &str,
  ) -> Result<PathBuf, OhNo> {
    let dir = resources.runtime_dir.join(prefix);
    let path = dir.join(session_name);

    // if the file doesn’t already exist, create it
    if let Ok(false) = path.try_exists() {
      // ensure the directory exists
      fs::create_dir_all(&dir).map_err(|err| OhNo::CannotCreateFifo {
        err: format!("cannot create directory {dir}: {err}", dir = dir.display()),
      })?;

      let path_bytes = path.as_os_str().as_bytes();
      let c_path = CString::new(path_bytes).map_err(|err| OhNo::CannotCreateFifo {
        err: err.to_string(),
      })?;

      let c_err = unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };
      if c_err != 0 {
        return Err(OhNo::CannotCreateFifo {
          err: format!(
            "cannot create FIFO file for session {session_name} at path {path}",
            path = path.display()
          ),
        });
      }
    }

    Ok(path)
  }

  /// Open a FIFO and obtain a non-blocking [`File`].
  fn open_nonblocking_fifo(path: &Path) -> Result<File, OhNo> {
    let fifo = OpenOptions::new()
      .read(true)
      .custom_flags(libc::O_NONBLOCK)
      .open(path)
      .map_err(|err| OhNo::CannotCreateFifo {
        err: format!(
          "cannot open non-blocking FIFO {path}: {err}",
          path = path.display()
        ),
      })?;
    Ok(fifo)
  }

  /// Recycle a session by removing the session from the session tracker and recycling the token in the token provider.
  fn recycle_session(
    &mut self,
    poll: &mut Poll,
    session_tracker: &mut SessionTracker,
    token_provider: &mut TokenProvider,
    session_name: impl AsRef<str>,
  ) -> Result<(), OhNo> {
    let session_name = session_name.as_ref();

    log::info!("recycling session {session_name}");
    if let Some((session, cmd_fifo, buf_fifo)) = session_tracker.untrack(session_name) {
      if let Some(cmd_fifo) = cmd_fifo {
        poll
          .registry()
          .deregister(&mut SourceFd(&cmd_fifo.file().as_raw_fd()))
          .map_err(|err| OhNo::PollError { err })?;
      }

      if let Some(buf_fifo) = buf_fifo {
        poll
          .registry()
          .deregister(&mut SourceFd(&buf_fifo.file().as_raw_fd()))
          .map_err(|err| OhNo::PollError { err })?;
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
      match fifo {
        Fifo::Cmd { file, buffer, .. } => self.accept_cmd(session, file, buffer)?,
        Fifo::Buf { file, buffer, .. } => self.accept_buf(session, file, buffer)?,
      }
    }

    Ok(())
  }

  fn accept_cmd(
    &mut self,
    session: &mut Session,
    file: &mut File,
    buffer: &mut String,
  ) -> Result<(), OhNo> {
    log::debug!(
      "reading command FIFO for session {session_name}…",
      session_name = session.name()
    );

    if let Err(err) = file.read_to_string(buffer) {
      if err.kind() == io::ErrorKind::WouldBlock {
        log::debug!("command FIFO is not ready");
        return Ok(());
      } else {
        buffer.clear();
        return Err(OhNo::InvalidRequest {
          req: "<cmd>".to_owned(),
          err: err.to_string(),
        });
      }
    };

    log::info!("FIFO request: {buffer}");

    let req = serde_json::from_str::<Request>(buffer).map_err(|err| OhNo::InvalidRequest {
      req: buffer.clone(),
      err: err.to_string(),
    });

    buffer.clear();

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
          dir: dir.clone(),
        };

        Ok(None)
      }
    }
  }

  fn accept_buf(
    &mut self,
    session: &mut Session,
    file: &mut File,
    buffer: &mut String,
  ) -> Result<(), OhNo> {
    log::debug!(
      "reading buffer FIFO for session {session_name}…",
      session_name = session.name()
    );

    if let Err(err) = file.read_to_string(buffer) {
      if err.kind() == io::ErrorKind::WouldBlock {
        log::debug!("buffer FIFO is not ready");
        return Ok(());
      } else {
        return Err(OhNo::InvalidRequest {
          req: "<buf>".to_owned(),
          err: err.to_string(),
        });
      }
    };

    let res = self.process_buf(session, buffer);
    buffer.clear();

    res
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
