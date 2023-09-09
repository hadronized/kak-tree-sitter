use std::{
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
  process::Command,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use colored::Colorize;
use kak_tree_sitter_config::Config;
use mio::{net::UnixListener, unix::SourceFd, Events, Interest, Poll, Token, Waker};

use crate::{
  cli::Cli,
  error::OhNo,
  handler::Handler,
  request::{Request, UnidentifiedRequest},
  response::Response,
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
  fn new(config: &Config, is_standalone: bool) -> Result<Self, OhNo> {
    let server_state = ServerState::new(config, is_standalone)?;
    Ok(Self { server_state })
  }

  /// Return whether we run as a server.
  pub fn bootstrap(config: &Config, cli: &Cli) -> Result<(), OhNo> {
    // find a runtime directory to write in
    let runtime_dir = ServerState::runtime_dir()?;
    eprintln!("running in {}", runtime_dir.display());

    // PID file
    let pid_file = runtime_dir.join("pid");

    // check whether a pid file exists and can be read
    if let Ok(pid) = std::fs::read_to_string(&pid_file) {
      let pid = pid.trim();
      eprintln!("checking whether PID {pid} is still up…");

      // if the contained pid corresponds to a running process, stop right away
      // otherwise, remove the files left by the previous instance and continue
      if Command::new("ps")
        .args(["-p", pid])
        .output()
        .is_ok_and(|o| o.status.success())
      {
        eprintln!("kak-tree-sitter already running; not starting a new server");
        return Ok(());
      } else {
        eprintln!("cleaning up previous instance");
        let _ = std::fs::remove_dir_all(&runtime_dir);
      }
    }

    // ensure that the runtime directory exists
    fs::create_dir_all(&runtime_dir).map_err(|err| OhNo::CannotCreateDir {
      dir: runtime_dir.clone(),
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

    Server::new(config, !cli.kakoune)?.start()?;

    Ok(())
  }

  fn start(mut self) -> Result<(), OhNo> {
    self.server_state.start()
  }

  pub fn send_request(req: UnidentifiedRequest) -> Result<(), OhNo> {
    // serialize the request
    let serialized = serde_json::to_string(&req).map_err(|err| OhNo::CannotSendRequest {
      err: err.to_string(),
    })?;

    eprintln!("sending unidentified request {req:?}");

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
#[derive(Debug)]
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
    // NOTE (#84): I’m not entirely sure what we should delete, because if KTS crashes for whatever reason, we will want
    // to keep access to the logs…

    // for now, we just remove the pid file so that we don’t cleanup next time we start
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

  // whether we were started as a standalone server
  is_standalone: bool,

  // readable poll
  poll: Poll,

  // UNIX handler.
  unix_handler: UnixHandler,

  // FIFO handler.
  fifo_handler: FifoHandler,

  // whether we should shutdown
  shutdown: Arc<AtomicBool>,

  // active sessions
  session_tracker: SessionTracker,

  // provider for FIFO tokens
  token_provider: TokenProvider,
}

impl ServerState {
  pub fn new(config: &Config, is_standalone: bool) -> Result<Self, OhNo> {
    let resources = ServerResources::new(Self::runtime_dir()?);
    let mut poll = Poll::new().map_err(|err| OhNo::CannotStartPoll { err })?;
    let waker = Arc::new(Waker::new(poll.registry(), TokenProvider::WAKER_TOKEN)?);
    let mut unix_handler = UnixHandler::new(ServerState::socket_path()?)?;
    let fifo_handler = FifoHandler::new(config)?;
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
      })?;
    }

    unix_handler.register_poll(&mut poll)?;

    Ok(ServerState {
      is_standalone,
      resources,
      poll,
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
          return Err(OhNo::from(err));
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
              self.is_standalone,
              &self.resources,
              &mut self.poll,
              &mut self.token_provider,
              &mut self.session_tracker,
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

    Ok(())
  }
}

#[derive(Debug)]
struct UnixHandler {
  unix_listener: UnixListener,
}

impl UnixHandler {
  fn new(socket_path: impl AsRef<Path>) -> Result<Self, OhNo> {
    let unix_listener =
      UnixListener::bind(socket_path).map_err(|err| OhNo::CannotStartServer { err })?;

    Ok(Self { unix_listener })
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
    is_standalone: bool,
    resources: &ServerResources,
    poll: &mut Poll,
    token_provider: &mut TokenProvider,
    session_tracker: &mut SessionTracker,
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

    let req = serde_json::from_str::<UnidentifiedRequest>(&req_str).map_err(|err| {
      OhNo::InvalidRequest {
        req: req_str,
        err: err.to_string(),
      }
    })?;

    self.process_req(
      is_standalone,
      resources,
      poll,
      token_provider,
      session_tracker,
      req,
    )
  }

  fn process_req(
    &mut self,
    is_standalone: bool,
    resources: &ServerResources,
    poll: &mut Poll,
    token_provider: &mut TokenProvider,
    session_tracker: &mut SessionTracker,
    req: UnidentifiedRequest,
  ) -> Result<Feedback, OhNo> {
    match req {
      UnidentifiedRequest::NewSession { name, client } => {
        let (cmd_fifo_path, buf_fifo_path) = self.track_session(
          resources,
          poll,
          token_provider,
          session_tracker,
          name.clone(),
        )?;

        let resp = Response::Init {
          cmd_fifo_path,
          buf_fifo_path,
        };
        Session::send_non_connected_response(&name, Some(&client), &resp)?;
      }

      UnidentifiedRequest::SessionExit { name } => {
        self.recycle_session(poll, session_tracker, token_provider, name)?;

        // only shutdown if were started with an initial session (non standalone)
        let feedback = if !is_standalone && session_tracker.is_empty() {
          log::info!("last session exited; stopping the server…");
          Feedback::ShouldExit
        } else {
          Feedback::Ok
        };
        return Ok(feedback);
      }

      UnidentifiedRequest::Shutdown => return Ok(Feedback::ShouldExit),
    }

    Ok(Feedback::Ok)
  }

  fn track_session(
    &mut self,
    resources: &ServerResources,
    poll: &mut Poll,
    token_provider: &mut TokenProvider,
    session_tracker: &mut SessionTracker,
    session_name: impl Into<String>,
  ) -> Result<(PathBuf, PathBuf), OhNo> {
    let session_name = session_name.into();

    let cmd_fifo_path = Self::create_fifo(resources, Path::new("commands"), &session_name)?;
    let cmd_fifo_file = Self::open_nonblocking_fifo(&cmd_fifo_path)?;
    let cmd_token = token_provider.create();

    let buf_fifo_path = Self::create_fifo(resources, Path::new("buffers"), &session_name)?;
    let buf_fifo_file = Self::open_nonblocking_fifo(&buf_fifo_path)?;
    let buf_token = token_provider.create();

    let registry = poll.registry();
    registry
      .register(
        &mut SourceFd(&cmd_fifo_file.as_raw_fd()),
        cmd_token,
        Interest::READABLE,
      )
      .map_err(|err| OhNo::PollEventsError { err })?;
    registry
      .register(
        &mut SourceFd(&buf_fifo_file.as_raw_fd()),
        buf_token,
        Interest::READABLE,
      )
      .map_err(|err| OhNo::PollEventsError { err })?;

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
      fs::create_dir_all(dir)?;

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
      .open(path)?;
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
          .deregister(&mut SourceFd(&cmd_fifo.file().as_raw_fd()))?;
      }

      if let Some(buf_fifo) = buf_fifo {
        poll
          .registry()
          .deregister(&mut SourceFd(&buf_fifo.file().as_raw_fd()))?;
      }

      token_provider.recycle(session.cmd_token());
      token_provider.recycle(session.buf_token());
    }

    Ok(())
  }
}

struct FifoHandler {
  handler: Handler,
}

impl FifoHandler {
  fn new(config: &Config) -> Result<Self, OhNo> {
    let handler = Handler::new(config)?;
    Ok(Self { handler })
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

    match file.read_to_string(buffer) {
      Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
        log::warn!("command FIFO is not ready");
        return Ok(());
      }

      x => x?,
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

          if let Err(err) = session.send_response(client, &resp) {
            log::error!("failure while sending response: {}", format!("{err}").red());
          }
        }

        Err(err) => {
          log::error!("handling request failed: {}", format!("{err}").red());
        }

        _ => (),
      },

      Err(err) => {
        log::error!("malformed request: {}", format!("{err}").red());
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

    match file.read_to_string(buffer) {
      Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
        log::warn!("buffer FIFO is not ready");
        return Ok(());
      }

      x => x?,
    };

    let res = self.process_buf(session, buffer);
    buffer.clear();

    res
  }

  fn process_buf(&mut self, session: &mut Session, buf: &str) -> Result<(), OhNo> {
    if let SessionState::HighlightingWaiting {
      client,
      buffer,
      lang,
      timestamp,
    } = session.state()
    {
      let client = client.clone();
      let handled = self
        .handler
        .handle_highlight(session.name(), buffer, lang, *timestamp, buf);

      // switch back to idle, as we have read the FIFO
      session.state_mut().idle();

      match handled {
        Ok(resp) => {
          if let Err(err) = session.send_response(Some(&client), &resp) {
            log::error!("failure while sending response: {}", format!("{err}").red());
          }
        }

        Err(err) => {
          log::error!(
            "handling highlight failed for session {session_name}, buffer {buf}: {err}",
            session_name = session.name()
          );
        }
      }
    }

    Ok(())
  }
}
