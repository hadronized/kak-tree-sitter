use std::{
  env, fs, io,
  path::{Path, PathBuf},
  process::Command,
};

use clap::Parser;
use cli::Cli;
use colored::Colorize;
use kak_tree_sitter_config::{Config, LanguageConfig};
use thiserror::Error;

mod cli;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("no runtime directory available")]
  NoRuntimeDir,

  #[error("no data directory to hold grammars / queries")]
  NoDataDir,

  #[error("bad path")]
  BadPath,

  #[error("cannot create directory {dir}: {err}")]
  CannotCreateDir { dir: PathBuf, err: io::Error },

  #[error("cannot remove directory {dir}: {err}")]
  CannotRemoveDir { dir: PathBuf, err: io::Error },

  #[error("error while waiting for process {process} to end: {err}")]
  ErrorWhileWaitingForProcess { process: String, err: io::Error },

  #[error("error while fetching grammar for language {lang}: {err}")]
  GrammarFetchError { lang: String, err: io::Error },

  #[error("error while fetching queries for language {lang}: {err}")]
  QueriesFetchError { lang: String, err: io::Error },

  #[error("error while compiling grammar for language {lang}: {err}")]
  CompileError { lang: String, err: io::Error },

  #[error("error while linking grammar for language {lang}: {err}")]
  LinkError { lang: String, err: io::Error },

  #[error("cannot copy {src} to {dest}: {err}")]
  CannotCopyFile {
    src: PathBuf,
    dest: PathBuf,
    err: io::Error,
  },

  #[error("cannot recursively copy from {src} to {dest}: {err}")]
  CannotCopyDir {
    src: PathBuf,
    dest: PathBuf,
    err: io::Error,
  },
}

fn main() {
  if let Err(err) = start() {
    eprintln!("{}", err.to_string().red());
    std::process::exit(1);
  }
}

fn info(msg: impl AsRef<str>) {
  println!("{}", msg.as_ref().blue().bold());
}

fn runtime_dir() -> Result<PathBuf, AppError> {
  let dir = dirs::runtime_dir()
    .or_else(|| env::var("TMPDIR").map(PathBuf::from).ok())
    .ok_or_else(|| AppError::NoRuntimeDir)?
    .join("ktsctl");
  Ok(dir)
}

fn kak_tree_sitter_data_dir() -> Result<PathBuf, AppError> {
  let dir = dirs::data_dir()
    .ok_or_else(|| AppError::NoDataDir)?
    .join("kak-tree-sitter");
  Ok(dir)
}

fn start() -> Result<(), AppError> {
  let cli = Cli::parse();
  let config = Config::load();

  // check the runtime dir exists
  let dir = runtime_dir()?;
  fs::create_dir_all(&dir).map_err(|err| AppError::CannotCreateDir {
    dir: dir.clone(),
    err,
  })?;

  // check the kak-tree-sitter data dir exists
  let kak_data_dir = kak_tree_sitter_data_dir()?;
  fs::create_dir_all(&kak_data_dir).map_err(|err| AppError::CannotCreateDir {
    dir: kak_data_dir,
    err,
  })?;

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
    fetch_grammar(&lang_config, &grammar_fetch_path, &lang)?;

    // if cli.queries is passed, fetch the queries; otherwise, reuse the grammar path
    if cli.queries {
      info(format!("fetching queries for language {lang}"));
      fetch_queries(&lang_config, &queries_fetch_path, &lang)?;
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
    fs::create_dir_all(&lang_build_dir).map_err(|err| AppError::CannotCreateDir {
      dir: lang_build_dir.clone(),
      err,
    })?;
    compile(&lang_config, &lang_build_dir, &lang)?;
  }

  if cli.install {
    info(format!("installing grammar for language {lang}"));

    // ensure the build dir exists
    fs::create_dir_all(&lang_build_dir).map_err(|err| AppError::CannotCreateDir {
      dir: lang_build_dir.clone(),
      err,
    })?;
    install_grammar(&lang_build_dir, &lang)?;

    // install the queries
    info(format!("installing queries for language {lang}"));
    let queries_path = queries_fetch_path.join(&lang_config.queries.path);
    install_queries(&queries_path, &lang)?;
  }

  Ok(())
}

fn fetch_grammar(
  lang_config: &LanguageConfig,
  fetch_path: &Path,
  lang: &str,
) -> Result<(), AppError> {
  let uri = lang_config.grammar.uri_fmt.replace("{lang}", lang);

  // cleanup / remove the {runtime_dir}/{lang} directory, if exists
  if let Ok(true) = fetch_path.try_exists() {
    fs::remove_dir_all(fetch_path).map_err(|err| AppError::CannotRemoveDir {
      dir: fetch_path.to_owned(),
      err,
    })?;
  }

  Command::new("git")
    .args([
      "clone",
      uri.as_str(),
      "--depth",
      "1",
      fetch_path
        .as_os_str()
        .to_str()
        .ok_or_else(|| AppError::BadPath)?,
    ])
    .spawn()
    .map_err(|err| AppError::GrammarFetchError {
      lang: lang.to_owned(),
      err,
    })?
    .wait()
    .map(|_| ())
    .map_err(|err| AppError::ErrorWhileWaitingForProcess {
      process: "git".to_owned(),
      err,
    })
}

/// Compile and link the grammar.
fn compile(
  lang_config: &LanguageConfig,
  lang_build_dir: &Path,
  lang: &str,
) -> Result<(), AppError> {
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
    .map_err(|err| AppError::CompileError {
      lang: lang.to_owned(),
      err,
    })?
    .wait()
    .map_err(|err| AppError::ErrorWhileWaitingForProcess {
      process: "git".to_owned(),
      err,
    })?;

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
    .map_err(|err| AppError::LinkError {
      lang: lang.to_owned(),
      err,
    })?
    .wait()
    .map(|_| ())
    .map_err(|err| AppError::ErrorWhileWaitingForProcess {
      process: "git".to_owned(),
      err,
    })
}

fn install_grammar(lang_build_dir: &Path, lang: &str) -> Result<(), AppError> {
  let lang_so = format!("{lang}.so");
  let source_path = lang_build_dir.join(&lang_so);
  let grammar_dir = kak_tree_sitter_data_dir()?.join("grammars");
  let install_path = grammar_dir.join(lang_so);

  // ensure the grammars directory exists
  fs::create_dir_all(&grammar_dir).map_err(|err| AppError::CannotCreateDir {
    dir: grammar_dir,
    err,
  })?;
  fs::copy(&source_path, &install_path).map_err(|err| AppError::CannotCopyFile {
    src: source_path,
    dest: install_path,
    err,
  })?;

  Ok(())
}

fn fetch_queries(
  lang_config: &LanguageConfig,
  fetch_path: &Path,
  lang: &str,
) -> Result<(), AppError> {
  let uri = lang_config.queries.uri_fmt.replace("{lang}", lang);

  // cleanup / remove the {runtime_dir}/{lang} directory, if exists
  if let Ok(true) = fetch_path.try_exists() {
    fs::remove_dir_all(fetch_path).map_err(|err| AppError::CannotRemoveDir {
      dir: fetch_path.to_owned(),
      err,
    })?;
  }

  Command::new("git")
    .args([
      "clone",
      uri.as_str(),
      "--depth",
      "1",
      fetch_path.as_os_str().to_str().ok_or(AppError::BadPath)?,
    ])
    .spawn()
    .map_err(|err| AppError::QueriesFetchError {
      lang: lang.to_owned(),
      err,
    })?
    .wait()
    .map(|_| ())
    .map_err(|err| AppError::ErrorWhileWaitingForProcess {
      process: "git".to_owned(),
      err,
    })
}

fn install_queries(query_dir: &Path, lang: &str) -> Result<(), AppError> {
  // ensure the queries directory exists
  let install_path = kak_tree_sitter_data_dir()?.join(format!("queries/{lang}"));
  fs::create_dir_all(&install_path).map_err(|err| AppError::CannotCreateDir {
    dir: install_path.clone(),
    err,
  })?;

  rec_copy_dir(query_dir, &install_path).map_err(|err| AppError::CannotCopyDir {
    src: query_dir.to_owned(),
    dest: install_path,
    err,
  })
}

fn rec_copy_dir(from: &Path, to: &Path) -> Result<(), io::Error> {
  for entry in from.read_dir()?.flatten() {
    let new_to = to.join(entry.file_name());

    if entry.file_type()?.is_dir() {
      fs::create_dir_all(&new_to)?;
      rec_copy_dir(&entry.path(), &new_to)?;
    } else {
      fs::copy(&entry.path(), &new_to)?;
    }
  }

  Ok(())
}
