use std::{
  collections::HashSet,
  env,
  fmt::Display,
  fs, io,
  path::{Path, PathBuf},
  process::Command,
};

use clap::Parser;
use cli::Cli;
use colored::Colorize;
use kak_tree_sitter_config::{source::Source, Config, LanguageConfig};
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

/// Flags taken out from the CLI to fetch, compile and/or install resources.
#[derive(Debug)]
struct ManageFlags {
  fetch: bool,
  compile: bool,
  install: bool,
}

impl ManageFlags {
  fn new(fetch: bool, compile: bool, install: bool) -> Self {
    Self {
      fetch,
      compile,
      install,
    }
  }
}

fn main() {
  if let Err(err) = start() {
    eprintln!("{}", err.to_string().red());
    std::process::exit(1);
  }
}

fn msg(msg: impl AsRef<str>) {
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
  let runtime_dir = runtime_dir()?;
  fs::create_dir_all(&runtime_dir).map_err(|err| AppError::CannotCreateDir {
    dir: runtime_dir.clone(),
    err,
  })?;

  // check the kak-tree-sitter data dir exists
  let install_dir = kak_tree_sitter_data_dir()?;
  fs::create_dir_all(&install_dir).map_err(|err| AppError::CannotCreateDir {
    dir: install_dir.clone(),
    err,
  })?;

  match cli.cmd {
    cli::Cmd::Manage {
      fetch,
      compile,
      install,
      lang,
    } => manage(
      &config,
      &runtime_dir,
      &install_dir,
      ManageFlags::new(fetch, compile, install),
      lang,
    ),

    cli::Cmd::Info { has, all } => info(&config, &install_dir, has, all),
  }
}

/// Manage mode.
fn manage(
  config: &Config,
  runtime_dir: &Path,
  install_dir: &Path,
  manage_flags: ManageFlags,
  lang: String,
) -> Result<(), AppError> {
  let lang_config =
    config
      .languages
      .get_lang_conf(&lang)
      .ok_or_else(|| AppError::MissingLangConfig {
        lang: lang.to_owned(),
      })?;

  // grammar
  match lang_config.grammar.source {
    Source::Path { ref dir } => {
      msg(format!(
        "using local grammar {lang} at {dir}",
        dir = dir.display()
      ));
    }

    Source::Git { ref url, ref pin } => manage_git_grammar(
      lang_config,
      runtime_dir,
      install_dir,
      url,
      pin.as_deref(),
      &manage_flags,
      &lang,
    )?,
  }

  // queries
  match lang_config.queries.source {
    Some(Source::Path { ref dir }) => {
      msg(format!(
        "using local queries {lang} at {dir}",
        dir = dir.display()
      ));
    }

    Some(Source::Git { ref url, ref pin }) => manage_git_queries(
      runtime_dir,
      install_dir,
      url,
      pin.as_deref(),
      &lang_config.queries.path,
      &manage_flags,
      &lang,
    )?,

    None => msg(format!(
      "no query configuration for {lang}; will be using the grammar directory"
    )),
  }

  Ok(())
}

/// Manage a git grammar.
///
/// For git repositories, we have to fetch, compile, link and install grammars.
fn manage_git_grammar(
  lang_config: &LanguageConfig,
  runtime_dir: &Path,
  install_dir: &Path,
  url: &str,
  pin: Option<&str>,
  manage_flags: &ManageFlags,
  lang: &str,
) -> Result<(), AppError> {
  let grammar_fetch_path = runtime_dir.join(format!("grammars/{lang}"));

  // fetch the language if required; it should be done at least once by the user, otherwise, the rest below will fail
  if manage_flags.fetch {
    msg(format!("fetching grammar for language {lang}"));
    fetch_via_git(url, pin, &grammar_fetch_path, lang)?;
  }

  let lang_build_dir = runtime_dir.join(format!(
    "{fetch_path}/{src_path}/build",
    fetch_path = grammar_fetch_path.display(),
    src_path = lang_config.grammar.path.display()
  ));

  if manage_flags.compile {
    msg(format!("compiling grammar for language {lang}"));

    // ensure the build dir exists
    fs::create_dir_all(&lang_build_dir).map_err(|err| AppError::CannotCreateDir {
      dir: lang_build_dir.clone(),
      err,
    })?;

    do_compile(lang_config, &lang_build_dir, lang)?;
  }

  if manage_flags.install {
    msg(format!("installing grammar for language {lang}"));
    install_grammar(install_dir, &lang_build_dir, lang)?;
  }

  Ok(())
}

/// Manage git-based queries.
///
/// For git repositories, we have to fetch, compile, link and install queries.
fn manage_git_queries(
  runtime_dir: &Path,
  install_dir: &Path,
  url: &str,
  pin: Option<&str>,
  path: &Path,
  manage_flags: &ManageFlags,
  lang: &str,
) -> Result<(), AppError> {
  let queries_fetch_path = runtime_dir.join(format!("queries/{lang}"));

  // fetch the language if required; it should be done at least once by the user, otherwise, the rest below will fail
  if manage_flags.fetch {
    msg(format!("fetching queries for {lang}"));
    fetch_via_git(url, pin, &queries_fetch_path, lang)?;
  }

  if manage_flags.install {
    msg(format!("installing queries for {lang}"));
    install_queries(install_dir, &queries_fetch_path.join(path), lang)?;
  }

  Ok(())
}

/// Info mode.
fn info(
  config: &Config,
  install_dir: &Path,
  has: Option<String>,
  all: bool,
) -> Result<(), AppError> {
  if let Some(lang) = has {
    display_lang_info(config, install_dir, &lang)?;
  } else if all {
    unimplemented!();
  }

  Ok(())
}

/// Display information about a given language.
fn display_lang_info(config: &Config, install_dir: &Path, lang: &str) -> Result<(), AppError> {
  // first, display the config
  let Some(lang_config) = config.languages.get_lang_conf(lang) else {
    return Err(AppError::MissingLangConfig {
      lang: lang.to_owned(),
    });
  };

  display_lang_config(lang_config);
  display_lang_install_stats(install_dir, lang);

  Ok(())
}

fn config_section(section: impl Display) -> impl Display {
  format!("-- {section} --").blue()
}

fn config_field(field: impl Display) -> impl Display {
  format!("{field}").cyan()
}

fn display_lang_config(config: &LanguageConfig) {
  // grammar first
  let grammar = &config.grammar;
  println!(
    r#"{section}
   {source_field}: {source:?}
   {path_field}: {path}
   {compile_field}: {compile} {compile_args:?}
   {compile_flags_field}: {compile_flags:?}
   {link_field}: {link} {link_args:?}
   {link_flags_field}: {link_flags:?}"#,
    section = config_section("Grammar configuration"),
    source_field = config_field("Source"),
    source = grammar.source,
    path_field = config_field("Path"),
    path = grammar.path.display(),
    compile_field = config_field("Compilation command"),
    compile = grammar.compile,
    compile_args = grammar.compile_args,
    compile_flags_field = config_field("Compilation flags"),
    compile_flags = grammar.compile_flags,
    link_field = config_field("Link command"),
    link = grammar.link,
    link_args = grammar.link_args,
    link_flags_field = config_field("Link flags"),
    link_flags = grammar.link_flags,
  );

  // then queries
  let queries = &config.queries;
  println!(
    r#"{section}
{source}   {path_field}: {path}"#,
    section = config_section("Queries configuration"),
    source = if let Some(ref source) = queries.source {
      let url_field = config_field("URL");
      format!("   {url_field}: {source:?}\n")
    } else {
      String::default()
    },
    path_field = config_field("Path"),
    path = queries.path.display()
  );

  // then the rest
  println!(
    r#"{section}
   {field}: {remove:?}"#,
    section = config_section("Remove default highlighter"),
    field = config_field("Value"),
    remove = bool::from(config.remove_default_highlighter)
  );
}

fn check_sign() -> impl Display {
  "".green()
}

fn warn_sign() -> impl Display {
  "".yellow()
}

fn no_sign() -> impl Display {
  "".red()
}

fn display_lang_install_stats(install_dir: &Path, lang: &str) {
  println!("{section}", section = config_section("Install stats"));

  let grammar_path = install_dir.join(format!("grammars/{lang}.so"));
  if let Ok(true) = grammar_path.try_exists() {
    println!(
      "   {sign} {lang} grammar: {path}",
      sign = check_sign(),
      path = grammar_path.display()
    );
  } else {
    println!(
      "   {sign} {lang} grammar missing; install with {help}",
      sign = no_sign(),
      help = format!("ktsctl manage -fci {lang}").bold()
    );
  };

  let queries_path = install_dir.join(format!("queries/{lang}"));
  display_queries_info(&queries_path, lang);
}

/// Check installed queries and report status.
fn display_queries_info(path: &Path, lang: &str) {
  if let Ok(true) = path.try_exists() {
    let scm_files: HashSet<_> = path
      .read_dir()
      .into_iter()
      .flatten()
      .flatten()
      .flat_map(|dir| dir.file_name().into_string())
      .collect();

    let mut scm_count = 0;
    let mut prefix_mark = |s, desc| {
      if scm_files.contains(s) {
        scm_count += 1;
        format!("     {sign} {desc}", sign = check_sign())
      } else {
        format!("     {sign} {desc}", sign = no_sign())
      }
    };

    let queries = [
      prefix_mark("highlights.scm", "highlights"),
      prefix_mark("indents.scm", "indents"),
      prefix_mark("injections.scm", "injections"),
      prefix_mark("locals.scm", "locals"),
      prefix_mark("textobjects.scm", "text-objects"),
    ];

    if scm_count == scm_files.len() {
      println!(
        "   {sign} {lang} queries installed: {path}",
        sign = check_sign(),
        path = path.display()
      );
    } else if scm_count > 0 {
      println!(
        "   {sign} {lang} queries partially installed: {path}",
        sign = warn_sign(),
        path = path.display()
      );
    } else {
      println!(
        "   {sign} {lang} queries missing; install with {help}",
        sign = no_sign(),
        help = format!("ktsctl manage -fci {lang}").bold()
      );
    }

    for q in queries {
      println!("{q}");
    }
  } else {
    println!(
      "   {sign} {lang} queries missing; install with {help}",
      sign = no_sign(),
      help = format!("ktsctl manage -fci {lang}").bold()
    );
  }
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

/// Compile and link the grammar.
fn do_compile(
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

fn install_grammar(install_dir: &Path, lang_build_dir: &Path, lang: &str) -> Result<(), AppError> {
  let lang_so = format!("{lang}.so");
  let source_path = lang_build_dir.join(&lang_so);
  let grammar_dir = install_dir.join("grammars");
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

fn install_queries(install_dir: &Path, query_dir: &Path, lang: &str) -> Result<(), AppError> {
  // ensure the queries directory exists
  let install_path = install_dir.join(format!("queries/{lang}"));
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
