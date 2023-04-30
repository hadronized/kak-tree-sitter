use std::{
  env,
  fs::{self, File},
  io::{Read, Write},
  os::unix::net::{UnixListener, UnixStream},
  path::PathBuf,
};

use crate::{config::Config, handler::Handler, request::Request};

#[derive(Debug)]
pub struct Daemon {
  config: Config,
  daemon_dir: PathBuf,
  unix_listener: UnixListener,
}

impl Daemon {
  fn new(config: Config, daemon_dir: PathBuf) -> Self {
    let unix_listener = UnixListener::bind(daemon_dir.join("socket")).unwrap(); // FIXME: unwrap()

    Self {
      config,
      daemon_dir,
      unix_listener,
    }
  }

  fn daemon_dir() -> PathBuf {
    let tmpdir = PathBuf::from(env::var("TMPDIR").expect("temporary directory"));
    let user = env::var("USER").expect("user");
    tmpdir.join(format!("kak-tree-sitter-{}", user))
  }

  pub fn start(config: Config) {
    // ensure we have a directory to write in
    let daemon_dir = Self::daemon_dir();
    fs::create_dir_all(&daemon_dir).unwrap(); // FIXME: error

    // PID file
    let pid_file = daemon_dir.join("pid");

    // check whether the PID file is already there; if so, it means the daemon is already running, so we wil just
    // stop right away
    if let Ok(true) = pid_file.try_exists() {
      eprintln!("kak-tree-sitter daemon already running; exiting");
      return;
    }

    // create stdout / stderr files
    let stdout_path = daemon_dir.join("stdout.txt");
    let stderr_path = daemon_dir.join("stderr.txt");
    let stdout = File::create(&stdout_path).unwrap();
    let stderr = File::create(&stderr_path).unwrap();

    daemonize::Daemonize::new()
      .stdout(stdout)
      .stderr(stderr)
      .pid_file(pid_file)
      .start()
      .expect("daemon");

    let daemon = Daemon::new(config, daemon_dir);

    daemon.run();
  }

  // Wait for incoming client and handle their requests.
  fn run(self) {
    let mut req_handler = Handler::new(&self.config);

    for client in self.unix_listener.incoming() {
      // FIXME: error handling
      if let Ok(mut client) = client {
        println!("client connected: {client:?}");
        let mut request = String::new();
        client.read_to_string(&mut request).unwrap(); // FIXME: unwrap()
        println!("request = {request:#?}");

        if request.is_empty() {
          break;
        }

        let resp = req_handler.handle_request(request);

        if let Some((mut session, resp)) = resp {
          let should_shutdown = resp.should_shutdown();
          session.send_response(resp);

          if should_shutdown {
            break;
          }
        }
      }
    }

    println!("bye!");
  }

  pub fn send_request(req: Request) {
    // serialize the request
    let serialized = serde_json::to_string(&req).unwrap(); // FIXME: unwrap()

    // connect and send the request to the daemon
    UnixStream::connect(Self::daemon_dir().join("socket"))
    .unwrap() // FIXME: unwrap()
    .write_all(serialized.as_bytes())
    .unwrap(); // FIXME: unwrap()
  }
}

impl Drop for Daemon {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.daemon_dir);
  }
}
