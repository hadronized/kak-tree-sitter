use std::{
  collections::HashSet,
  env,
  fmt::Display,
  fs,
  io::{self, stdout, Write},
  path::{Path, PathBuf},
  process::{Command, Stdio},
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

#[derive(Debug)]
struct Report;

impl Report {
  fn new(icon: ReportIcon, msg: impl AsRef<str>) -> Self {
    print!("\x1b[?7l");
    Self::to_stdout(icon, msg);
    Self
  }

  fn to_stdout(icon: ReportIcon, msg: impl AsRef<str>) {
    print!("{} {msg}", icon, msg = msg.as_ref());
    stdout().flush().unwrap();
  }

  fn report(&self, icon: ReportIcon, msg: impl AsRef<str>) {
    print!("\x1b[2K\r");
    Self::to_stdout(icon, msg);
  }
}

impl Drop for Report {
  fn drop(&mut self) {
    println!("\x1b[?7h");
    stdout().flush().unwrap();
  }
}

#[derive(Debug)]
enum ReportIcon {
  Fetch,
  Compile,
  Link,
  Install,
  Success,
  Error,
  Info,
}

impl Display for ReportIcon {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ReportIcon::Fetch => write!(f, "{}", "".magenta()),
      ReportIcon::Compile => write!(f, "{}", "".cyan()),
      ReportIcon::Link => write!(f, "{}", "".cyan()),
      ReportIcon::Install => write!(f, "{}", "".cyan()),
      ReportIcon::Success => write!(f, "{}", "".green()),
      ReportIcon::Error => write!(f, "{}", "".red()),
      ReportIcon::Info => write!(f, "{}", "󰈅".blue()),
    }
  }
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

    cli::Cmd::Info { lang, all } => info(&config, &install_dir, lang, all),
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
    Source::Local { ref path } => {
      Report::new(
        ReportIcon::Info,
        format!(
          "using local grammar {lang} at {path}",
          path = path.display()
        ),
      );
    }

    Source::Git { ref url, ref pin } => manage_git_grammar(
      lang_config,
      runtime_dir,
      install_dir,
      url,
      pin,
      &manage_flags,
      &lang,
    )?,
  }

  // queries
  match lang_config.queries.source {
    Some(Source::Local { ref path }) => {
      Report::new(
        ReportIcon::Info,
        format!(
          "using local queries {lang} at {path}",
          path = path.display()
        ),
      );
    }

    Some(Source::Git { ref url, ref pin }) => manage_git_queries(
      runtime_dir,
      install_dir,
      url,
      pin,
      &lang_config.queries.path,
      &manage_flags,
      &lang,
    )?,

    None => {
      Report::new(
        ReportIcon::Error,
        format!("no query configuration for {lang}; will be using the grammar directory"),
      );
    }
  }

  Ok(())
}

/// Generate a source directory for a given URL.
fn sources_dir(runtime_dir: &Path, url: &str) -> Result<PathBuf, AppError> {
  let url_dir = PathBuf::from(
    url
      .trim_start_matches("http")
      .trim_start_matches('s')
      .trim_start_matches("://"),
  );
  let path = runtime_dir.join("sources").join(url_dir);

  Ok(path)
}

/// Manage a git grammar.
///
/// For git repositories, we have to fetch, compile, link and install grammars.
fn manage_git_grammar(
  lang_config: &LanguageConfig,
  runtime_dir: &Path,
  install_dir: &Path,
  url: &str,
  pin: &str,
  manage_flags: &ManageFlags,
  lang: &str,
) -> Result<(), AppError> {
  let sources_path = sources_dir(runtime_dir, url)?;

  // fetch the language if required; it should be done at least once by the user, otherwise, the rest below will fail
  if manage_flags.fetch {
    let report = Report::new(ReportIcon::Fetch, format!("fetching {lang} grammar…"));
    let fetched = fetch_via_git(&report, url, pin, &sources_path, lang)?;

    if fetched {
      report.report(
        ReportIcon::Success,
        format!(
          "fetched {lang} grammar at {path}",
          path = sources_path.display(),
        ),
      );
    } else {
      report.report(
        ReportIcon::Success,
        format!(
          "already fetched {lang} grammar at {path} (cached)",
          path = sources_path.display(),
        ),
      );
    }
  }

  let lang_build_dir = runtime_dir.join(format!(
    "{fetch_path}/{src_path}/build",
    fetch_path = sources_path.display(),
    src_path = lang_config.grammar.path.display()
  ));

  if manage_flags.compile {
    // ensure the build dir exists
    fs::create_dir_all(&lang_build_dir).map_err(|err| AppError::CannotCreateDir {
      dir: lang_build_dir.clone(),
      err,
    })?;

    do_compile(lang_config, &lang_build_dir, lang)?;
  }

  if manage_flags.install {
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
  pin: &str,
  path: &Path,
  manage_flags: &ManageFlags,
  lang: &str,
) -> Result<(), AppError> {
  let sources_path = sources_dir(runtime_dir, url)?;

  // fetch the language if required; it should be done at least once by the user, otherwise, the rest below will fail
  if manage_flags.fetch {
    let report = Report::new(ReportIcon::Fetch, format!("fetching {lang} queries",));
    let fetched = fetch_via_git(&report, url, pin, &sources_path, lang)?;

    if fetched {
      report.report(
        ReportIcon::Success,
        format!(
          "fetched {lang} queries at {path}",
          path = sources_path.display(),
        ),
      );
    } else {
      report.report(
        ReportIcon::Success,
        format!(
          "already fetched {lang} queries at {path} (cached)",
          path = sources_path.display(),
        ),
      );
    }
  }

  if manage_flags.install {
    install_queries(install_dir, &sources_path.join(path), lang)?;
  }

  Ok(())
}

/// Info mode.
fn info(
  config: &Config,
  install_dir: &Path,
  lang: Option<String>,
  all: bool,
) -> Result<(), AppError> {
  if let Some(lang) = lang {
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
  format!("· {section}").bold()
}

fn config_field(field: impl Display) -> impl Display {
  format!("{field}{}", delim(":")).blue()
}

fn delim(d: impl Display) -> impl Display {
  format!("{d}").black()
}

fn display_source(source: &Source) {
  match source {
    Source::Local { path } => {
      println!(
        "  {} {}",
        config_field("Source (path)"),
        format!("{}", path.display()).green()
      );
    }

    Source::Git { url, pin } => {
      println!(
        "  {} {} {}{}{}",
        config_field("Source (git)"),
        url.green(),
        "(".black(),
        pin.red(),
        ")".black()
      );
    }
  }
}

fn display_list(list: &[impl AsRef<str>]) {
  print!("{}", delim("["));

  if !list.is_empty() {
    print!("{}", list[0].as_ref().green());

    for elem in &list[1..] {
      print!("{} {}", delim(","), elem.as_ref().green());
    }
  }

  println!("{}", delim("]"));
}

fn display_lang_config(config: &LanguageConfig) {
  // grammar first
  let grammar = &config.grammar;
  println!("{}", config_section("Grammar configuration"));
  display_source(&grammar.source);

  // path
  println!(
    "  {} {} ",
    config_field("Path"),
    format!("{}", grammar.path.display()).green()
  );

  // compilation arguments
  print!(
    "  {} {} ",
    config_field("Compilation command"),
    grammar.compile.green()
  );
  display_list(&grammar.compile_args);

  // compilation flags
  print!("  {} ", config_field("Compilation flags"));
  display_list(&grammar.compile_flags);

  // link arguments
  print!(
    "  {} {} ",
    config_field("Link command"),
    grammar.link.green()
  );
  display_list(&grammar.link_args);

  // link flags
  print!("  {} ", config_field("Link flags"));
  display_list(&grammar.link_flags);

  // then queries
  let queries = &config.queries;
  println!("{}", config_section("Queries configuration"));

  if let Some(ref source) = queries.source {
    display_source(source);
  }

  println!(
    "  {path_field} {path}",
    path_field = config_field("Path"),
    path = format!("{}", queries.path.display()).green(),
  );

  // then the rest
  println!("{}", config_section("Remove default highlighter"));
  println!(
    "  {field} {remove}",
    field = config_field("Value"),
    remove = bool::from(config.remove_default_highlighter)
      .to_string()
      .green()
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
      "   {sign} {grammar}{delim} {path}",
      sign = check_sign(),
      grammar = format!("{lang} grammar").blue(),
      delim = delim(":"),
      path = format!("{}", grammar_path.display()).green()
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
    let mut scm_expected_count = 0;
    let mut prefix_mark = |s: &str, desc: &str| {
      scm_expected_count += 1;

      if scm_files.contains(s) {
        scm_count += 1;
        format!(
          "     {sign} {desc}",
          sign = check_sign(),
          desc = desc.blue()
        )
      } else {
        format!("     {sign} {desc}", sign = no_sign(), desc = desc.blue())
      }
    };

    let queries = [
      prefix_mark("highlights.scm", "highlights"),
      prefix_mark("indents.scm", "indents"),
      prefix_mark("injections.scm", "injections"),
      prefix_mark("locals.scm", "locals"),
      prefix_mark("textobjects.scm", "text-objects"),
    ];

    if scm_count == scm_expected_count {
      println!(
        "   {sign} {queries}{delim} {path}",
        sign = check_sign(),
        queries = format!("{lang} queries").blue(),
        delim = delim(":"),
        path = format!("{}", path.display()).green(),
      );
    } else if scm_count > 0 {
      println!(
        "   {sign} {queries}{delim} {path}",
        sign = warn_sign(),
        queries = format!("{lang} queries").blue(),
        delim = delim(":"),
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
///
/// Return `Ok(true)` if something was fetched; `Ok(false)` if it was already there.
fn fetch_via_git(
  report: &Report,
  url: &str,
  pin: &str,
  fetch_path: &Path,
  lang: &str,
) -> Result<bool, AppError> {
  // check if the fetch path already exists; if so, we abort
  // cleanup / remove the {runtime_dir}/{lang} directory, if exists
  if let Ok(true) = fetch_path.try_exists() {
    return Ok(false);
  }

  fs::create_dir_all(fetch_path).map_err(|err| AppError::CannotCreateDir {
    dir: fetch_path.to_owned(),
    err,
  })?;

  let git_clone_args = vec![
    "clone",
    url,
    "-n",
    fetch_path
      .as_os_str()
      .to_str()
      .ok_or_else(|| AppError::BadPath)?,
  ];

  report.report(ReportIcon::Fetch, format!("cloning {url}"));
  Command::new("git")
    .args(git_clone_args)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
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

  report.report(ReportIcon::Info, format!("checking out {pin}"));
  Command::new("git")
    .args(["reset", "--hard", pin])
    .current_dir(fetch_path)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
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

  Ok(true)
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

  let report = Report::new(ReportIcon::Compile, format!("compiling {lang} grammar"));
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

  report.report(ReportIcon::Success, format!("compiled {lang} grammar"));
  drop(report);

  // link into {lang}.so
  let args = lang_config
    .grammar
    .link_args
    .iter()
    .map(|arg| arg.replace("{lang}", lang))
    .chain(lang_config.grammar.link_flags.clone());
  let report = Report::new(ReportIcon::Link, format!("linking {lang} grammar",));
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
    })?;

  report.report(ReportIcon::Success, format!("linked {lang} grammar"));

  Ok(())
}

fn install_grammar(install_dir: &Path, lang_build_dir: &Path, lang: &str) -> Result<(), AppError> {
  let lang_so = format!("{lang}.so");
  let source_path = lang_build_dir.join(&lang_so);
  let grammar_dir = install_dir.join("grammars");
  let install_path = grammar_dir.join(lang_so);
  let report = Report::new(ReportIcon::Install, format!("installing {lang} grammar"));

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

  report.report(ReportIcon::Success, format!("installed {lang} grammar"));

  Ok(())
}

fn install_queries(install_dir: &Path, query_dir: &Path, lang: &str) -> Result<(), AppError> {
  // ensure the queries directory exists
  let install_path = install_dir.join(format!("queries/{lang}"));
  let report = Report::new(ReportIcon::Install, format!("installing {lang} queries"));

  fs::create_dir_all(&install_path).map_err(|err| AppError::CannotCreateDir {
    dir: install_path.clone(),
    err,
  })?;

  copy_dir(query_dir, &install_path).map_err(|err| AppError::CannotCopyDir {
    src: query_dir.to_owned(),
    dest: install_path,
    err,
  })?;

  report.report(ReportIcon::Success, format!("installed {lang} queries"));

  Ok(())
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
