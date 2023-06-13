use std::{
  fs::{self, File},
  io::Write,
  os::unix::net::UnixStream,
  path::PathBuf,
  process::Command,
  sync::mpsc,
};

use colored::Colorize;
use kak_tree_sitter_config::Config;
use tokio::{io::AsyncReadExt, net::UnixListener, select};

use crate::{
  error::OhNo,
  handler::Handler,
  request::{KakTreeSitterOrigin, KakouneOrigin, Request},
  response::Response,
};

#[derive(Debug)]
pub struct Daemon {
  config: Config,
  daemon_dir: PathBuf,
  unix_listener: UnixListener,
}

impl Daemon {
  fn new(config: Config, daemon_dir: PathBuf) -> Result<Self, OhNo> {
    let unix_listener = UnixListener::bind(daemon_dir.join("socket"))
      .map_err(|err| OhNo::CannotStartServer { err })?;

    Ok(Self {
      config,
      daemon_dir,
      unix_listener,
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

    let async_rt =
      tokio::runtime::Runtime::new().map_err(|err| OhNo::CannotStartAsyncRuntime { err })?;
    async_rt.block_on(async {
      let daemon = Daemon::new(config, daemon_dir)?;
      daemon.run().await
    })
  }

  /// Wait for incoming client and enqueue their requests.
  async fn run(self) -> Result<(), OhNo> {
    let mut req_handler = Handler::new(&self.config)?;
    let (req_sx, req_rx) = mpsc::channel();
    let (shutdown_sx, mut shutdown_rx) = tokio::sync::mpsc::unbounded_channel();

    let handler_handle = tokio::task::spawn_blocking(move || {
      for req in req_rx {
        match req_handler.handle_request(req) {
          Ok(resp) => {
            if let Some((mut session, resp)) = resp {
              if let Response::Shutdown = resp {
                if let Err(err) = shutdown_sx.send(()) {
                  eprintln!("{}", format!("cannot properly shutdown: {err}").red());
                }

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
    });

    loop {
      select! {
        _ = shutdown_rx.recv() => break,
        Ok((mut client, _)) = self.unix_listener.accept() => {
          println!("client connected: {client:?}");

          // read the request and parse it
          let mut req_str = String::new();
          client.read_to_string(&mut req_str).await.map_err(|err| OhNo::InvalidRequest { err: err.to_string() })?;

          let req = serde_json::from_str::<Request<KakTreeSitterOrigin>>(&req_str).map_err(|err| OhNo::InvalidRequest { err: err.to_string() })?;

          req_sx.send(req).map_err(|err| OhNo::CannotSendRequest { err: err.to_string() })?;
        }
      }
    }

    handler_handle.await.map_err(|err| OhNo::ShutdownFailure {
      err: err.to_string(),
    })?;

    println!("bye!");

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

impl Drop for Daemon {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.daemon_dir);
  }
}
