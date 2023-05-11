use std::{
  env, fs,
  path::{Path, PathBuf},
  process::Command,
};

use clap::Parser;
use cli::Cli;
use kak_tree_sitter_config::{Config, LanguageConfig};

mod cli;

fn runtime_dir() -> PathBuf {
  dirs::runtime_dir()
    .or_else(|| env::var("TMPDIR").map(PathBuf::from).ok())
    .unwrap()
    .join("ktsctl") // FIXME: unwrap()
}

fn kak_tree_sitter_data_dir() -> PathBuf {
  dirs::data_dir().unwrap().join("kak-tree-sitter") // FIXME: unwrap()
}

fn main() {
  let cli = Cli::parse();
  let config = Config::load_from_xdg();

  // check the runtime dir exists
  let dir = runtime_dir();
  fs::create_dir_all(&dir).unwrap();

  // check the kak-tree-sitter data dir exists
  let kak_data_dir = kak_tree_sitter_data_dir();
  fs::create_dir_all(&kak_data_dir).unwrap();

  let lang = cli.lang;
  let lang_config = config.languages.get_lang_conf(&lang);

  let fetch_path = PathBuf::from(format!(
    "{runtime_dir}/grammars/{lang}",
    runtime_dir = dir.display(),
  ));

  // fetch the language if required; it should be done at least once by the user, otherwise, the rest below will fail
  if cli.fetch {
    fetch_grammar(&lang_config, &fetch_path, &dir, &lang);
  }

  let lang_build_dir = dir.join(format!(
    "{fetch_path}/{extra_path}/build",
    fetch_path = fetch_path.display(),
    extra_path = lang_config.grammar.path.display()
  ));

  if cli.compile {
    // ensure the build dir exists
    fs::create_dir_all(&lang_build_dir).unwrap(); // FIXME: unwrap()
    compile(&lang_config, &lang_build_dir, &lang);
  }

  if cli.install {
    // ensure the build dir exists
    fs::create_dir_all(&lang_build_dir).unwrap(); // FIXME: unwrap()
    install_grammar(&lang_build_dir, &lang);
  }

  if cli.queries {
    fetch_queries(&dir);

    if cli.install {
      let queries_dir = dir.join("helix/runtime/queries");
      install_queries(&queries_dir);
    }
  }
}

/// Fetch lang’s grammars and queries by targetting https://github.com/tree-sitter/tree-sitter-{lang}.
fn fetch_grammar(lang_config: &LanguageConfig, fetch_path: &Path, runtime_dir: &Path, lang: &str) {
  let uri = lang_config.grammar.uri_fmt.replace("{lang}", lang);
  println!("fetching grammar for language {lang} from {uri}");

  // cleanup / remove the {runtime_dir}/{lang} directory, if exists
  if let Ok(true) = fetch_path.try_exists() {
    fs::remove_dir_all(fetch_path).unwrap(); // FIXME: unwrap()
  }

  Command::new("git")
    .args([
      "clone",
      uri.as_str(),
      "--depth",
      "1",
      fetch_path.as_os_str().to_str().unwrap(), // FIXME: unwrap()
    ])
    .current_dir(runtime_dir)
    .spawn()
    .unwrap()
    .wait()
    .unwrap(); // FIXME: unwrap()
}

/// Compile the grammar.
fn compile(lang_config: &LanguageConfig, lang_build_dir: &Path, lang: &str) {
  // compile into .o
  Command::new(&lang_config.grammar.compile)
    .args(
      lang_config
        .grammar
        .compile_args
        .iter()
        .map(|arg| arg.replace("{lang}", lang)),
    )
    .current_dir(&lang_build_dir)
    .spawn()
    .unwrap()
    .wait()
    .unwrap(); // FIXME: unwrap()

  // link into {lang}.so
  Command::new(&lang_config.grammar.link)
    .args(
      lang_config
        .grammar
        .link_args
        .iter()
        .map(|arg| arg.replace("{lang}", lang)),
    )
    .current_dir(&lang_build_dir)
    .spawn()
    .unwrap()
    .wait()
    .unwrap(); // FIXME: unwrap()
}

fn install_grammar(lang_build_dir: &Path, lang: &str) {
  let lang_so = format!("{lang}.so");
  let source_path = lang_build_dir.join(&lang_so);
  let grammar_dir = kak_tree_sitter_data_dir().join("grammars");
  let install_path = grammar_dir.join(lang_so);
  println!(
    "installing grammar for {lang} from {source_path} in {install_path}",
    source_path = source_path.display(),
    install_path = install_path.display()
  );

  // ensure the grammars directory exists
  fs::create_dir_all(grammar_dir).unwrap();
  fs::copy(source_path, install_path).unwrap(); // FIXME: unwrap()
}

fn fetch_queries(runtime_dir: &Path) {
  // cleanup / remove the helix directory
  let hx_dir = PathBuf::from(format!(
    "{runtime_dir}/helix",
    runtime_dir = runtime_dir.display()
  ));

  if let Ok(true) = hx_dir.try_exists() {
    fs::remove_dir_all(hx_dir).unwrap(); // FIXME: unwrap()
  }

  let url = format!("https://github.com/helix-editor/helix");
  Command::new("git")
    .args(["clone", url.as_str(), "--depth", "1"])
    .current_dir(runtime_dir)
    .spawn()
    .unwrap()
    .wait()
    .unwrap(); // FIXME: unwrap()
}

fn install_queries(query_dir: &Path) {
  // ensure the queries directory exists
  let install_path = kak_tree_sitter_data_dir().join("queries");
  fs::create_dir_all(&install_path).unwrap();

  rec_copy_dir(query_dir, &install_path);
}

fn rec_copy_dir(from: &Path, to: &Path) {
  for entry in from.read_dir().unwrap().flatten() {
    let new_to = to.join(entry.file_name());

    if entry.file_type().unwrap().is_dir() {
      fs::create_dir_all(&new_to).unwrap();
      rec_copy_dir(&entry.path(), &new_to);
    } else {
      fs::copy(&entry.path(), &new_to).unwrap();
    }
  }
}
