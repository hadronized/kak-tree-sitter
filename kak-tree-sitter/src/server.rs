use std::{
  collections::HashMap,
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
  path::PathBuf,
  process::Command,
};

use colored::Colorize;
use kak_tree_sitter_config::Config;
use mio::{net::UnixListener, unix::SourceFd, Events, Interest, Poll, Token};

use crate::{
  error::OhNo,
  handler::Handler,
  request::{KakTreeSitterOrigin, Request, UnidentifiedRequest},
  response::Response,
  session::KakSession,
};

pub struct Server {
  server_state: ServerState,
}

impl Server {
  fn new(config: &Config) -> Result<Self, OhNo> {
    let server_state = ServerState::new(config)?;
    Ok(Self { server_state })
  }

  pub fn bootstrap(config: &Config, daemonize: bool) -> Result<(), OhNo> {
    // find a runtime directory to write in
    let runtime_dir = ServerState::runtime_dir()?;
    eprintln!("running in {}", runtime_dir.display());

    // PID file
    let pid_file = runtime_dir.join("pid");

    // check whether a pid file exists and can be read
    if let Ok(pid) = std::fs::read_to_string(&pid_file) {
      // if the contained pid corresponds to a running process, stop right away
      // otherwise, remove the files left by the previous instance and continue
      if Command::new("ps")
        .args(["-p", &pid])
        .output()
        .is_ok_and(|o| o.status.success())
      {
        eprintln!("kak-tree-sitter already running; exiting");
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

    if daemonize {
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

    Server::new(config)?.start()
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
    let _ = std::fs::remove_dir_all(&self.runtime_dir);
  }
}

pub struct ServerState {
  resources: ServerResources,

  // readable poll
  poll: Poll,

  // UNIX socket listener
  unix_listener: UnixListener,

  // whether we should shutdown
  shutdown: bool,

  // active command FIFOs; one per session
  cmd_fifos: HashMap<Token, SessionFifo>,

  // next available token
  next_token: Token,

  // available tokens (i.e. dead sessions’ tokens)
  free_tokens: Vec<Token>,

  // request handler
  req_handler: Handler,
}

impl ServerState {
  const UNIX_LISTENER_TOKEN: Token = Token(0);
  const CMD_FIFO_FIRST_TOKEN: Token = Token(1);

  pub fn new(config: &Config) -> Result<Self, OhNo> {
    let resources = ServerResources::new(Self::runtime_dir()?);
    let poll = Poll::new().map_err(|err| OhNo::CannotStartPoll { err })?;
    let mut unix_listener = UnixListener::bind(ServerState::socket_dir()?)
      .map_err(|err| OhNo::CannotStartServer { err })?;
    let shutdown = false;
    let cmd_fifos = HashMap::default();
    let next_token = Self::CMD_FIFO_FIRST_TOKEN;
    let free_tokens = Vec::default();
    let req_handler = Handler::new(config)?;

    poll
      .registry()
      .register(
        &mut unix_listener,
        Self::UNIX_LISTENER_TOKEN,
        Interest::READABLE,
      )
      .map_err(|err| OhNo::CannotStartPoll { err })?;

    Ok(ServerState {
      resources,
      poll,
      unix_listener,
      shutdown,
      cmd_fifos,
      next_token,
      free_tokens,
      req_handler,
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

  pub fn socket_dir() -> Result<PathBuf, OhNo> {
    Ok(Self::runtime_dir()?.join("socket"))
  }

  /// Start the server state and wait for events to be dispatched.
  pub fn start(&mut self) -> Result<(), OhNo> {
    eprintln!("starting server");

    let mut events = Events::with_capacity(1024);
    loop {
      if self.shutdown {
        break;
      }

      if let Err(err) = self.poll.poll(&mut events, None) {
        if err.kind() == io::ErrorKind::WouldBlock {
          // spurious events
          continue;
        } else {
          return Err(OhNo::PollEventsError { err });
        }
      }

      for event in &events {
        match event.token() {
          Self::UNIX_LISTENER_TOKEN if event.is_readable() => self.accept_unix_request()?,

          tkn if event.is_readable() => self.accept_cmd_fifo_req(tkn)?,

          _ => (),
        }
      }
    }

    Ok(())
  }

  /// Accept a new request on the UNIX socket.
  fn accept_unix_request(&mut self) -> Result<(), OhNo> {
    let (mut client, _) = self
      .unix_listener
      .accept()
      .map_err(|err| OhNo::UnixConnectionError { err })?;

    println!("client connected: {client:?}");

    // read the request and parse it
    let mut req_str = String::new();
    client
      .read_to_string(&mut req_str)
      .map_err(|err| OhNo::InvalidRequest {
        err: err.to_string(),
      })?;
    println!("UNIX socket request: {req_str}");

    let req = serde_json::from_str::<UnidentifiedRequest>(&req_str).map_err(|err| {
      OhNo::InvalidRequest {
        err: err.to_string(),
      }
    })?;

    self.treat_unidentified_request(req)
  }

  fn treat_unidentified_request(&mut self, req: UnidentifiedRequest) -> Result<(), OhNo> {
    match req {
      UnidentifiedRequest::NewSession { name } => {
        let cmd_fifo_path = self.add_session_fifo(name.clone())?;
        let resp = Response::InitialResponse {
          fifo_cmd_path: cmd_fifo_path,
        };
        KakSession::new(name, None).send_response(&resp)?;
      }

      UnidentifiedRequest::SessionExit { name } => self.recycle_session_fifo(name)?,

      UnidentifiedRequest::Shutdown => {
        self.shutdown = true;
      }
    }

    Ok(())
  }

  /// Take into account a new session by creating a command FIFO for it and associating it with a token.
  fn add_session_fifo(&mut self, session_name: impl Into<String>) -> Result<PathBuf, OhNo> {
    let session_name = session_name.into();

    let (cmd_fifo_path, cmd_fifo) = self.create_session_cmd_fifo(&session_name)?;
    let token = self.create_session_fifo_token();

    self
      .poll
      .registry()
      .register(
        &mut SourceFd(&cmd_fifo.as_raw_fd()),
        token,
        Interest::READABLE,
      )
      .map_err(|err| OhNo::PollEventsError { err })?;

    self
      .cmd_fifos
      .insert(token, SessionFifo::new(session_name, cmd_fifo));

    Ok(cmd_fifo_path)
  }

  /// Get a new token for a new session.
  fn create_session_fifo_token(&mut self) -> Token {
    self.free_tokens.pop().unwrap_or_else(|| {
      let token = self.next_token;
      self.next_token = Token(self.next_token.0 + 1);
      token
    })
  }

  /// Recycle a session.
  fn recycle_session_fifo(&mut self, session_name: impl AsRef<str>) -> Result<(), OhNo> {
    if let Some((token, session_fifo)) = self
      .cmd_fifos
      .iter()
      .find(|(_, session_fifo)| session_fifo.session_name == session_name.as_ref())
    {
      let token = *token;
      self
        .poll
        .registry()
        .deregister(&mut SourceFd(&session_fifo.fifo.as_raw_fd()))?;
      // TODO: remove the FIFO file? do we care?
      self.free_tokens.push(token);
      self.cmd_fifos.remove(&token);
    }

    if self.cmd_fifos.is_empty() {
      self.shutdown = true;
    }

    Ok(())
  }

  /// Create a command FIFO for a given session, associating it with a token to be registered in a poll.
  fn create_session_cmd_fifo(&self, session_name: &str) -> Result<(PathBuf, File), OhNo> {
    let cmds_dir = self.resources.runtime_dir.join("commands");
    let cmd_fifo_path = cmds_dir.join(session_name);

    // if the file doesn’t already exist, create it
    if let Ok(false) = cmd_fifo_path.try_exists() {
      // ensure the commands directory exists
      fs::create_dir_all(cmds_dir)?;

      let path = cmd_fifo_path.as_os_str().as_bytes();
      let c_path = CString::new(path).map_err(|err| OhNo::CannotCreateCommandFifo {
        err: err.to_string(),
      })?;

      let c_err = unsafe { libc::mkfifo(c_path.as_ptr(), 0o777) };
      if c_err != 0 {
        return Err(OhNo::CannotCreateCommandFifo {
          err: format!("cannot create FIFO file for session {session_name}"),
        });
      }
    }

    let cmd_fifo = OpenOptions::new()
      .read(true)
      .custom_flags(libc::O_NONBLOCK)
      .open(&cmd_fifo_path)?;

    Ok((cmd_fifo_path, cmd_fifo))
  }

  /// Accept a command request on a FIFO identified by a token.
  fn accept_cmd_fifo_req(&mut self, token: Token) -> Result<(), OhNo> {
    if let Some(session_fifo) = self.cmd_fifos.get_mut(&token) {
      let mut commands = String::new();
      session_fifo.fifo.read_to_string(&mut commands)?;

      let split_cmds = commands.split(';').filter(|s| !s.is_empty());

      for cmd in split_cmds {
        println!("FIFO request: {cmd}");
        let req = serde_json::from_str::<Request<KakTreeSitterOrigin>>(cmd).map_err(|err| {
          OhNo::InvalidRequest {
            err: err.to_string(),
          }
        });

        match req {
          Ok(req) => match self.req_handler.handle_request(req) {
            Ok(resp) => {
              if let Some((mut session, resp)) = resp {
                if let Err(err) = session.send_response(&resp) {
                  eprintln!("failure while sending response: {}", format!("{err}").red());
                }
              }
            }
            Err(err) => {
              eprintln!("handling request failed: {}", format!("{err}").red());
            }
          },

          Err(err) => {
            eprintln!("malformed request: {}", format!("{err}").red());
          }
        }
      }
    }

    Ok(())
  }
}

/// FIFO associated with a session.
#[derive(Debug)]
struct SessionFifo {
  session_name: String,
  fifo: File,
}

impl SessionFifo {
  fn new(session_name: String, fifo: File) -> Self {
    Self { session_name, fifo }
  }
}
