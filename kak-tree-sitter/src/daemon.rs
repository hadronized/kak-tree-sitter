use std::{
  ffi::CString,
  fs::{self, File, OpenOptions},
  io::{Read, Write},
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
    mpsc::{channel, Receiver, Sender},
    Arc,
  },
  thread,
};

use colored::Colorize;
use kak_tree_sitter_config::Config;
use libc::O_NONBLOCK;
use mio::{net::UnixListener, unix::SourceFd, Events, Interest, Poll, Token, Waker};

use crate::{
  error::OhNo,
  handler::Handler,
  request::{KakTreeSitterOrigin, KakouneOrigin, Request},
  response::Response,
};

// TODO: we need to split that into separate types
//
// - one type that will hold the data required to read from UNIX sockets and/or FIFOs and enqueue requests
// - one type that will be responsible in dequeuing requests
pub struct Daemon {
  _resources: DaemonResources,
  intake: Intake,
  req_queue: RequestQueue,
}

impl Daemon {
  fn new(config: Config, daemon_dir: PathBuf) -> Result<Self, OhNo> {
    let resources = DaemonResources::new(daemon_dir);
    let (req_sx, req_rx) = channel();
    let shutdown = Arc::new(AtomicBool::new(false));
    let intake = Intake::new(
      &resources.daemon_dir.join("socket"),
      req_sx,
      shutdown.clone(),
    )?;
    let req_queue = RequestQueue::new(&config, req_rx, shutdown, intake.waker()?)?;

    Ok(Self {
      _resources: resources,
      intake,
      req_queue,
    })
  }

  fn daemon_dir() -> Result<PathBuf, OhNo> {
    let dir = dirs::runtime_dir()
      .or_else(||
        // macOS doesn’t implement XDG, yay…
        std::env::var("TMPDIR").map(PathBuf::from).ok())
      .ok_or_else(|| OhNo::NoRuntimeDir)?;
    Ok(dir.join("kak-tree-sitter"))
  }

  pub fn command_fifo_path() -> Result<PathBuf, OhNo> {
    Ok(Self::daemon_dir()?.join("commands"))
  }

  pub fn bootstrap(config: Config, daemonize: bool) -> Result<(), OhNo> {
    // find a runtime directory to write in
    let daemon_dir = Self::daemon_dir()?;
    eprintln!("running in {}", daemon_dir.display());

    // PID file
    let pid_file = daemon_dir.join("pid");

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
        let _ = std::fs::remove_dir_all(&daemon_dir);
      }
    }

    // ensure that the runtime directory exists
    fs::create_dir_all(&daemon_dir).map_err(|err| OhNo::CannotCreateDir {
      dir: daemon_dir.clone(),
      err,
    })?;

    if daemonize {
      // create stdout / stderr files
      let stdout_path = daemon_dir.join("stdout.txt");
      let stderr_path = daemon_dir.join("stderr.txt");
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

    let daemon = Daemon::new(config, daemon_dir)?;
    daemon.start()
  }

  fn start(mut self) -> Result<(), OhNo> {
    // start the intake; this will wait for incoming requests and will write them to the request queue
    let mut intake = self.intake;
    thread::spawn(move || {
      if let Err(err) = intake.start() {
        eprintln!("{}", format!("{err}").red());
      }
    });

    // dequeue requests
    self.req_queue.start()?;

    Ok(())
  }

  pub fn send_request(req: Request<KakouneOrigin>) -> Result<(), OhNo> {
    // reinterpret the request to mark it as from kak-tree-sitter
    let kts_req = req.reinterpret()?;

    // serialize the request
    let serialized = serde_json::to_string(&kts_req).map_err(|err| OhNo::CannotSendRequest {
      err: err.to_string(),
    })?;

    // connect and send the request to the daemon
    UnixStream::connect(Self::daemon_dir()?.join("socket"))
      .map_err(|err| OhNo::CannotConnectToServer { err })?
      .write_all(serialized.as_bytes())
      .map_err(|err| OhNo::CannotSendRequest {
        err: err.to_string(),
      })
  }
}

/// Daemon resources.
#[derive(Debug)]
struct DaemonResources {
  daemon_dir: PathBuf,
}

impl DaemonResources {
  fn new(daemon_dir: PathBuf) -> Self {
    Self { daemon_dir }
  }
}

impl Drop for DaemonResources {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.daemon_dir);
  }
}

/// Request intake.
///
/// Can read from UNIX sockets and FIFOs.
#[derive(Debug)]
struct Intake {
  poll: Poll,
  unix_listener: UnixListener,
  cmd_fifo: File,
  req_sx: Sender<Request<KakTreeSitterOrigin>>,
  shutdown: Arc<AtomicBool>,
}

impl Intake {
  const WAKE_TOKEN: Token = Token(0);
  const UNIX_LISTENER_TOKEN: Token = Token(1);
  const CMD_FIFO_TOKEN: Token = Token(2);

  /// Create a new [`Intake`] that will send request to the provided sender.
  fn new(
    socket_path: &Path,
    req_sx: Sender<Request<KakTreeSitterOrigin>>,
    shutdown: Arc<AtomicBool>,
  ) -> Result<Self, OhNo> {
    let mut unix_listener =
      UnixListener::bind(socket_path).map_err(|err| OhNo::CannotStartServer { err })?;
    let cmd_fifo_path = Daemon::command_fifo_path()?;

    // create the FIFO file, if it doesn’t already exists
    if let Ok(false) = cmd_fifo_path.try_exists() {
      Self::create_cmd_fifo(&cmd_fifo_path)?;
    }

    // let cmd_fifo = File::open(cmd_fifo_path)?;
    let cmd_fifo = OpenOptions::new()
      .read(true)
      .custom_flags(O_NONBLOCK)
      .open(cmd_fifo_path)?;

    let poll = Poll::new().map_err(|err| OhNo::CannotStartPoll { err })?;
    let registry = poll.registry();
    registry.register(
      &mut unix_listener,
      Self::UNIX_LISTENER_TOKEN,
      Interest::READABLE,
    )?;
    registry.register(
      &mut SourceFd(&cmd_fifo.as_raw_fd()),
      Self::CMD_FIFO_TOKEN,
      Interest::READABLE,
    )?;

    Ok(Self {
      poll,
      unix_listener,
      cmd_fifo,
      req_sx,
      shutdown,
    })
  }

  /// Return a waker to manually wake the poll.
  ///
  /// Used mainly to shutdown after setting an atomic bool.
  pub fn waker(&self) -> Result<Arc<Waker>, OhNo> {
    Ok(Arc::new(Waker::new(
      self.poll.registry(),
      Self::WAKE_TOKEN,
    )?))
  }

  /// Create the command FIFO, which is used by Kakoune sessions to send requests.
  ///
  /// This FIFO is shared with all sessions.
  fn create_cmd_fifo(cmd_fifo_path: &Path) -> Result<(), OhNo> {
    let path = cmd_fifo_path.as_os_str().as_bytes();
    let c_path = CString::new(path).map_err(|err| OhNo::CannotCreateCommandFifo {
      err: err.to_string(),
    })?;

    let c_err = unsafe { libc::mkfifo(c_path.as_ptr(), 0o777) };
    if c_err != 0 {
      return Err(OhNo::CannotCreateCommandFifo {
        err: "cannot create FIFO file".to_owned(),
      });
    }

    Ok(())
  }

  /// Wait for events.
  fn start(&mut self) -> Result<(), OhNo> {
    eprintln!("starting intake");

    let mut events = Events::with_capacity(1024);

    'outer: loop {
      self
        .poll
        .poll(&mut events, None)
        .map_err(|err| OhNo::PollEventsError { err })?;

      for event in &events {
        eprintln!("poll event: {event:?}",);

        let tkn = event.token();

        match tkn {
          Self::WAKE_TOKEN if event.is_readable() => {
            if self.shutdown.load(Ordering::Relaxed) {
              break 'outer;
            }
          }

          Self::UNIX_LISTENER_TOKEN if event.is_readable() => self.accept_unix_request()?,

          Self::CMD_FIFO_TOKEN if event.is_readable() => self.accept_cmd_fifo_req()?,

          _ => (),
        }
      }
    }

    Ok(())
  }

  fn accept_unix_request(&self) -> Result<(), OhNo> {
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

    let req = serde_json::from_str::<Request<KakTreeSitterOrigin>>(&req_str).map_err(|err| {
      OhNo::InvalidRequest {
        err: err.to_string(),
      }
    })?;

    self
      .req_sx
      .send(req)
      .map_err(|err| OhNo::CannotSendRequest {
        err: err.to_string(),
      })?;

    Ok(())
  }

  fn accept_cmd_fifo_req(&mut self) -> Result<(), OhNo> {
    let mut commands = String::new();
    self.cmd_fifo.read_to_string(&mut commands)?;

    let split_cmds = commands.split(';').filter(|s| !s.is_empty());

    for cmd in split_cmds {
      println!("FIFO request: {cmd}");
      let req = serde_json::from_str::<Request<KakTreeSitterOrigin>>(cmd).map_err(|err| {
        OhNo::InvalidRequest {
          err: err.to_string(),
        }
      });

      match req {
        Ok(req) => {
          self
            .req_sx
            .send(req)
            .map_err(|err| OhNo::CannotSendRequest {
              err: err.to_string(),
            })?;
        }

        Err(err) => {
          eprintln!("{}", format!("{err}").red());
        }
      }
    }

    Ok(())
  }
}

/// Request queue.
///
/// This type is responsible for holding inbound requests and handling them.
struct RequestQueue {
  req_handler: Handler,
  req_rx: Receiver<Request<KakTreeSitterOrigin>>,
  shutdown: Arc<AtomicBool>,
  intake_waker: Arc<Waker>,
}

impl RequestQueue {
  fn new(
    config: &Config,
    req_rx: Receiver<Request<KakTreeSitterOrigin>>,
    shutdown: Arc<AtomicBool>,
    intake_waker: Arc<Waker>,
  ) -> Result<Self, OhNo> {
    let req_handler = Handler::new(config)?;

    Ok(RequestQueue {
      req_handler,
      req_rx,
      shutdown,
      intake_waker,
    })
  }

  fn start(&mut self) -> Result<(), OhNo> {
    eprintln!("waiting for events to dequeue…");

    for req in &self.req_rx {
      eprintln!("dequeued request: {req:?}");

      match self.req_handler.handle_request(req) {
        Ok(resp) => {
          if let Some((mut session, resp)) = resp {
            if let Response::Shutdown = resp {
              self.shutdown.store(true, Ordering::Relaxed);
              self.intake_waker.wake()?;
              break;
            }

            if let Err(err) = session.send_response(&resp) {
              eprintln!("{}", err);
            }
          }
        }

        Err(err) => {
          eprintln!("{}", format!("error while handling request: {err}").red());
        }
      }
    }

    eprintln!("request queue stopped");

    Ok(())
  }
}
