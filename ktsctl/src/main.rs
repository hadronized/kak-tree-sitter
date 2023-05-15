use std::{
  env, fs,
  path::{Path, PathBuf},
  process::Command,
};

use clap::Parser;
use cli::Cli;
use colored::Colorize;
use kak_tree_sitter_config::{Config, LanguageConfig};

mod cli;

fn info(msg: impl AsRef<str>) {
  println!("{}", msg.as_ref().blue().bold());
}

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

  let grammar_fetch_path = dir.join(format!("grammars/{lang}"));
  let queries_fetch_path = if cli.queries {
    dir.join(format!("queries/{lang}"))
  } else {
    grammar_fetch_path.clone()
  };

  // fetch the language if required; it should be done at least once by the user, otherwise, the rest below will fail
  if cli.fetch {
    info(format!(
      "fetching grammar {maybe_queries} for language {lang}",
      maybe_queries = if cli.queries { "" } else { "/ queries" }
    ));
    fetch_grammar(&lang_config, &grammar_fetch_path, &lang);

    // if cli.queries is passed, fetch the queries; otherwise, reuse the grammar path
    if cli.queries {
      info(format!("fetching queries for language {lang}"));
      fetch_queries(&lang_config, &queries_fetch_path, &lang);
    }
  }

  let lang_build_dir = dir.join(format!(
    "{fetch_path}/{extra_path}/build",
    fetch_path = grammar_fetch_path.display(),
    extra_path = lang_config.grammar.path.display()
  ));

  if cli.compile {
    info(format!("compiling grammar for language {lang}"));

    // ensure the build dir exists
    fs::create_dir_all(&lang_build_dir).unwrap(); // FIXME: unwrap()
    compile(&lang_config, &lang_build_dir, &lang);
  }

  if cli.install {
    info(format!("installing grammar for language {lang}"));

    // ensure the build dir exists
    fs::create_dir_all(&lang_build_dir).unwrap(); // FIXME: unwrap()
    install_grammar(&lang_build_dir, &lang);

    // install the queries
    info(format!("installing queries for language {lang}"));
    let queries_path = queries_fetch_path.join(&lang_config.queries.path);
    install_queries(&queries_path, &lang);
  }
}

fn fetch_grammar(lang_config: &LanguageConfig, fetch_path: &Path, lang: &str) {
  let uri = lang_config.grammar.uri_fmt.replace("{lang}", lang);

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
    .spawn()
    .unwrap()
    .wait()
    .unwrap(); // FIXME: unwrap()
}

/// Compile and link the grammar.
fn compile(lang_config: &LanguageConfig, lang_build_dir: &Path, lang: &str) {
  // compile into .o
  let args = lang_config
    .grammar
    .compile_args
    .iter()
    .map(|arg| arg.replace("{lang}", lang))
    .chain(lang_config.grammar.compile_flags.clone());
  Command::new(&lang_config.grammar.compile)
    .args(args)
    .current_dir(lang_build_dir)
    .spawn()
    .unwrap()
    .wait()
    .unwrap(); // FIXME: unwrap()

  // link into {lang}.so
  let args = lang_config
    .grammar
    .link_args
    .iter()
    .map(|arg| arg.replace("{lang}", lang))
    .chain(lang_config.grammar.link_flags.clone());
  Command::new(&lang_config.grammar.link)
    .args(args)
    .current_dir(lang_build_dir)
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

  // ensure the grammars directory exists
  fs::create_dir_all(grammar_dir).unwrap();
  fs::copy(source_path, install_path).unwrap(); // FIXME: unwrap()
}

fn fetch_queries(lang_config: &LanguageConfig, fetch_path: &Path, lang: &str) {
  let uri = lang_config.queries.uri_fmt.replace("{lang}", lang);

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
    .spawn()
    .unwrap()
    .wait()
    .unwrap(); // FIXME: unwrap()
}

fn install_queries(query_dir: &Path, lang: &str) {
  // ensure the queries directory exists
  let install_path = kak_tree_sitter_data_dir().join(format!("queries/{lang}"));
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
