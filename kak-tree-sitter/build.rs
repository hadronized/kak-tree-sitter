use std::process::Command;

fn main() {
  let output = Command::new("git")
    .args(["rev-parse", "--short", "HEAD"])
    .output()
    .unwrap();

  let key = "GIT_HEAD";
  let value = String::from_utf8(output.stdout).unwrap();
  println!("cargo:rustc-env={key}={value}");
}
