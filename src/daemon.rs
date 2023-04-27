use std::{
  env,
  fs::{self, File},
  io::{Read, Write},
  os::unix::net::{UnixListener, UnixStream},
  path::PathBuf,
};

use crate::handler::Handler;

#[derive(Debug)]
pub struct Daemon {
  daemon_dir: PathBuf,
  unix_listener: UnixListener,
}

impl Daemon {
  fn new(daemon_dir: PathBuf) -> Self {
    let unix_listener = UnixListener::bind(daemon_dir.join("socket")).unwrap(); // FIXME: unwrap()

    Self {
      daemon_dir,
      unix_listener,
    }
  }

  fn daemon_dir() -> PathBuf {
    let tmpdir = PathBuf::from(env::var("TMPDIR").expect("temporary directory"));
    let user = env::var("USER").expect("user");
    tmpdir.join(format!("kak-tree-sitter-{}", user))
  }

  pub fn start() {
    // ensure we have a directory to write in
    let daemon_dir = Self::daemon_dir();
    fs::create_dir_all(&daemon_dir).unwrap(); // FIXME: error

    // create stdout / stderr files
    let stdout_path = daemon_dir.join("stdout.txt");
    let stderr_path = daemon_dir.join("stderr.txt");
    let stdout = File::create(&stdout_path).unwrap();
    let stderr = File::create(&stderr_path).unwrap();

    // PID file
    let pid_file = daemon_dir.join("pid");

    daemonize::Daemonize::new()
      .stdout(stdout)
      .stderr(stderr)
      .pid_file(pid_file)
      .start()
      .expect("daemon");

    let daemon = Daemon::new(daemon_dir);
    println!("daemon started: {daemon:?}");

    daemon.run();
  }

  // Wait for incoming client and handle their requests.
  fn run(self) {
    let mut req_handler = Handler::new();

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

        if resp.should_shutdown() {
          break;
        }
      }
    }

    println!("bye!");
  }

  pub fn send_request(request: impl Into<String>) {
    // connect and send the request to the daemon
    UnixStream::connect(Self::daemon_dir().join("socket"))
    .unwrap() // FIXME: unwrap()
    .write_all(request.into().as_bytes())
    .unwrap(); // FIXME: unwrap()
  }
}

impl Drop for Daemon {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.daemon_dir);
  }
}
