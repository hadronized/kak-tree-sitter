use std::{fmt::Write, process::Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut version = env!("CARGO_PKG_VERSION").to_owned();
  if let Some(sha1) = git_sha1() {
    write!(&mut version, "-{sha1}")?;
  }

  println!("cargo:rustc-env=VERSION={version}");

  Ok(())
}

fn git_sha1() -> Option<String> {
  Command::new("git")
    .args(["rev-parse", "--short", "HEAD"])
    .output()
    .ok()
    .filter(|stdout| stdout.status.success())
    .and_then(|stdout| String::from_utf8(stdout.stdout).ok())
    .map(|hash| hash.trim().to_owned())
}
