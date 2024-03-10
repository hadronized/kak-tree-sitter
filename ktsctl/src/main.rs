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

  #[error("configuration error: {err}")]
  ConfigError {
    #[from]
    err: kak_tree_sitter_config::ConfigError,
  },

  #[error("no configuration for language {lang}")]
  MissingLangConfig { lang: String },

  #[error("error while waiting for process {process} to end: {err}")]
  ErrorWhileWaitingForProcess { process: String, err: io::Error },

  #[error("error while fetching resource for language {lang}: {err}")]
  FetchError { lang: String, err: io::Error },

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
  let config = Config::load_default_user()?;

  // check the runtime dir exists
  let dir = runtime_dir()?;
  fs::create_dir_all(&dir).map_err(|err| AppError::CannotCreateDir {
    dir: dir.clone(),
    err,
  })?;

  // check the kak-tree-sitter data dir exists
  let kak_data_dir = kak_tree_sitter_data_dir()?;
  fs::create_dir_all(&kak_data_dir).map_err(|err| AppError::CannotCreateDir {
    dir: kak_data_dir.clone(),
    err,
  })?;

  let lang = cli.lang;
  let lang_config =
    config
      .languages
      .get_lang_conf(&lang)
      .ok_or_else(|| AppError::MissingLangConfig {
        lang: lang.to_owned(),
      })?;

  let grammar_fetch_path = dir.join(format!("grammars/{lang}"));
  let queries_fetch_path = if lang_config.queries.url.is_some() {
    dir.join(format!("queries/{lang}"))
  } else {
    grammar_fetch_path.clone()
  };

  if cli.has {
    let grammars_path = kak_data_dir.join(format!("grammars/{lang}.so"));
    let grammar = if let Ok(true) = grammars_path.try_exists() {
      "".green()
    } else {
      "".red()
    };

    let queries_path = kak_data_dir.join(format!("queries/{lang}"));
    let queries = if let Ok(true) = queries_path.try_exists() {
      "".green()
    } else {
      "".red()
    };

    println!(
      "{grammar} {lang} grammar {}",
      grammars_path.display().to_string().black()
    );
    println!(
      "{queries} {lang} queries {}",
      queries_path.display().to_string().black()
    );
  }

  // fetch the language if required; it should be done at least once by the user, otherwise, the rest below will fail
  if cli.fetch {
    info(format!("fetching grammar for language {lang}",));
    fetch_grammar(lang_config, &grammar_fetch_path, &lang)?;

    // if cli.queries is passed, fetch the queries; otherwise, reuse the grammar path
    if let Some(ref url) = lang_config.queries.url {
      info(format!("fetching queries for language {lang}"));
      fetch_via_git(
        url,
        lang_config.queries.pin.as_deref(),
        &queries_fetch_path,
        &lang,
      )?;
    }
  }

  let lang_build_dir = dir.join(format!(
    "{fetch_path}/{src_path}/build",
    fetch_path = grammar_fetch_path.display(),
    src_path = lang_config.grammar.path.display()
  ));

  if cli.compile {
    info(format!("compiling grammar for language {lang}"));

    // ensure the build dir exists
    fs::create_dir_all(&lang_build_dir).map_err(|err| AppError::CannotCreateDir {
      dir: lang_build_dir.clone(),
      err,
    })?;
    compile(lang_config, &lang_build_dir, &lang)?;
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
    install_queries(&queries_fetch_path.join(&lang_config.queries.path), &lang)?;
  }

  Ok(())
}

/// Fetch an URL via git, and support pinning.
fn fetch_via_git(
  url: &str,
  pin: Option<&str>,
  fetch_path: &Path,
  lang: &str,
) -> Result<(), AppError> {
  // cleanup / remove the {runtime_dir}/{lang} directory, if exists
  if let Ok(true) = fetch_path.try_exists() {
    fs::remove_dir_all(fetch_path).map_err(|err| AppError::CannotRemoveDir {
      dir: fetch_path.to_owned(),
      err,
    })?;
  }

  let git_clone_args = if pin.is_some() {
    vec![
      "clone",
      url,
      "-n",
      fetch_path
        .as_os_str()
        .to_str()
        .ok_or_else(|| AppError::BadPath)?,
    ]
  } else {
    vec![
      "clone",
      url,
      "--depth",
      "1",
      fetch_path
        .as_os_str()
        .to_str()
        .ok_or_else(|| AppError::BadPath)?,
    ]
  };
  Command::new("git")
    .args(git_clone_args)
    .spawn()
    .map_err(|err| AppError::FetchError {
      lang: lang.to_owned(),
      err,
    })?
    .wait()
    .map(|_| ())
    .map_err(|err| AppError::ErrorWhileWaitingForProcess {
      process: "git clone".to_owned(),
      err,
    })?;

  if let Some(pin) = pin {
    Command::new("git")
      .args(["reset", "--hard", pin])
      .current_dir(fetch_path)
      .spawn()
      .map_err(|err| AppError::FetchError {
        lang: lang.to_owned(),
        err,
      })?
      .wait()
      .map(|_| ())
      .map_err(|err| AppError::ErrorWhileWaitingForProcess {
        process: "git reset".to_owned(),
        err,
      })?;
  }

  Ok(())
}

fn fetch_grammar(
  lang_config: &LanguageConfig,
  fetch_path: &Path,
  lang: &str,
) -> Result<(), AppError> {
  let url = lang_config.grammar.url.replace("{lang}", lang);
  let pin = lang_config.grammar.pin.as_ref();

  fetch_via_git(&url, pin.map(|pin| pin.as_str()), fetch_path, lang)
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

fn install_queries(query_dir: &Path, lang: &str) -> Result<(), AppError> {
  // ensure the queries directory exists
  let install_path = kak_tree_sitter_data_dir()?.join(format!("queries/{lang}"));
  fs::create_dir_all(&install_path).map_err(|err| AppError::CannotCreateDir {
    dir: install_path.clone(),
    err,
  })?;

  copy_dir(query_dir, &install_path).map_err(|err| AppError::CannotCopyDir {
    src: query_dir.to_owned(),
    dest: install_path,
    err,
  })
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), io::Error> {
  for entry in from.read_dir()?.flatten() {
    let new_to = to.join(entry.file_name());

    if entry.file_type()?.is_file() {
      fs::copy(&entry.path(), &new_to)?;
    }
  }

  Ok(())
}
