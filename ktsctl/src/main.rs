use std::{
  collections::HashSet,
  fmt::Display,
  fs, io,
  path::{Path, PathBuf},
};

use clap::Parser;
use cli::Cli;
use colored::Colorize;
use error::HellNo;
use kak_tree_sitter_config::{
  source::Source, Config, LanguageConfig, LanguageGrammarConfig, LanguageQueriesConfig,
};

use crate::commands::manage::{ManageFlags, Manager};

mod cli;
mod commands;
mod error;
mod git;
mod process;
mod report;
mod resources;

fn main() {
  if let Err(err) = start() {
    eprintln!("{}", err.to_string().red());
    std::process::exit(1);
  }
}

fn start() -> Result<(), HellNo> {
  let cli = Cli::parse();

  if cli.verbose {
    simple_logger::init_with_level(log::Level::Debug)?;
  }

  let config = Config::load_default_user()?;
  log::debug!("ktsctl configuration:\n{config:#?}");

  match cli.cmd {
    cli::Cmd::Manage {
      fetch,
      compile,
      install,
      sync,
      lang,
      all,
    } => {
      let manage_flags = ManageFlags::new(fetch, compile, install, sync);
      let manager = Manager::new(config, manage_flags)?;

      if let Some(lang) = lang {
        manager.manage(&lang)?;
      } else if all {
        manager.manage_all(config.languages.language.keys().map(String::as_str));
      }
    }

    cli::Cmd::Info { lang, all } => info(&config, &install_dir, lang.as_deref(), all)?,
  }

  Ok(())
}

/// Info mode.
fn info(config: &Config, install_dir: &Path, lang: Option<&str>, all: bool) -> Result<(), HellNo> {
  if let Some(lang) = lang {
    display_lang_info(config, install_dir, lang)?;
  } else if all {
    display_all_lang_info(config, install_dir)?;
  }

  Ok(())
}

/// Display information about all languages.
fn display_all_lang_info(config: &Config, install_dir: &Path) -> Result<(), HellNo> {
  println!(
    "{lang_header:^20}| {g} | {h} | {i} | {l} | {t} | {z}",
    lang_header = "Language".bold(),
    g = "Grammar".bold(),
    h = "Highlights".bold(),
    i = "Injections".bold(),
    l = "Locals".bold(),
    t = "Text-objects".bold(),
    z = "Indents".bold(),
  );
  println!(
    "{f:-^20}|---------|------------|------------|--------|--------------|---------",
    f = "-"
  );

  let mut langs = config.languages.language.iter().collect::<Vec<_>>();
  langs.sort_by(|(a, _), (b, _)| a.cmp(b));

  for (lang, lang_config) in langs {
    let grammar_path = get_grammar_path(&lang_config.grammar, install_dir, lang);

    let lang = &lang[..lang.len().min(20)];
    if let Some(queries_path) = get_queries_path(&lang_config.queries, install_dir, lang) {
      print!("{lang:^20}");

      if let Ok(true) = grammar_path.try_exists() {
        print!("|{:^9}", check_sign());
      } else {
        print!("|{:^9}", no_sign());
      }

      if let Ok(true) = queries_path.join("highlights.scm").try_exists() {
        print!("|{:^12}", check_sign());
      } else {
        print!("|{:^12}", no_sign());
      }

      if let Ok(true) = queries_path.join("injections.scm").try_exists() {
        print!("|{:^12}", check_sign());
      } else {
        print!("|{:^12}", no_sign());
      }

      if let Ok(true) = queries_path.join("locals.scm").try_exists() {
        print!("|{:^8}", check_sign());
      } else {
        print!("|{:^8}", no_sign());
      }

      if let Ok(true) = queries_path.join("textobjects.scm").try_exists() {
        print!("|{:^14}", check_sign());
      } else {
        print!("|{:^14}", no_sign());
      }

      if let Ok(true) = queries_path.join("indents.scm").try_exists() {
        print!("|{:^9}", check_sign());
      } else {
        print!("|{:^9}", no_sign());
      }

      println!();
    } else {
      println!(
        "{lang:^20}|{no:^9}|{no:^12}|{no:^12}|{no:^8}|{no:^14}|{no:^9}",
        no = no_sign()
      );
    }
  }

  Ok(())
}

/// Display information about a given language.
fn display_lang_info(config: &Config, install_dir: &Path, lang: &str) -> Result<(), HellNo> {
  // first, display the config
  let Some(lang_config) = config.languages.get_lang_conf(lang) else {
    return Err(HellNo::MissingLangConfig {
      lang: lang.to_owned(),
    });
  };

  display_lang_config(lang_config);
  display_lang_install_stats(lang_config, install_dir, lang);

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

fn display_lang_install_stats(lang_config: &LanguageConfig, install_dir: &Path, lang: &str) {
  println!("{section}", section = config_section("Install stats"));

  display_grammar_info(&lang_config.grammar, install_dir, lang);
  display_queries_info(&lang_config.queries, install_dir, lang);
}

fn get_grammar_path(config: &LanguageGrammarConfig, install_dir: &Path, lang: &str) -> PathBuf {
  match config.source {
    Source::Local { ref path } => path.clone(),
    Source::Git { ref pin, .. } => install_dir.join(format!("grammars/{lang}/{pin}.so")),
  }
}

fn get_queries_path(
  config: &LanguageQueriesConfig,
  install_dir: &Path,
  lang: &str,
) -> Option<PathBuf> {
  let path = match config.source.as_ref()? {
    Source::Local { ref path } => path.clone(),
    Source::Git { ref pin, .. } => install_dir.join(format!("queries/{lang}/{pin}")),
  };

  Some(path)
}

/// Check install grammar and report status.
fn display_grammar_info(config: &LanguageGrammarConfig, install_dir: &Path, lang: &str) {
  let grammar_path = match config.source {
    Source::Local { ref path } => path.clone(),
    Source::Git { ref pin, .. } => install_dir.join(format!("grammars/{lang}/{pin}.so")),
  };

  if let Ok(true) = grammar_path.try_exists() {
    println!(
      "   {sign} {grammar}",
      sign = check_sign(),
      grammar = format!("{lang} grammar").blue(),
    );
  } else {
    let grammars_path = install_dir.join(format!("grammars/{lang}"));

    if let Ok(true) = grammars_path.try_exists() {
      // we might have a list of stuff
      println!(
        "   {sign} {lang} grammar out of sync; synchronize with {help}",
        sign = no_sign(),
        help = format!("ktsctl manage -cis {lang}").bold()
      );
    } else {
      println!(
        "   {sign} {lang} grammar missing; install with {help}",
        sign = no_sign(),
        help = format!("ktsctl manage -fci {lang}").bold()
      );
    }
  };
}

/// Check installed queries and report status.
fn display_queries_info(config: &LanguageQueriesConfig, install_dir: &Path, lang: &str) {
  let Some(source) = config.source.as_ref() else {
    return;
  };
  let queries_path = match source {
    Source::Local { path } => path.clone(),
    Source::Git { pin, .. } => install_dir.join(format!("queries/{lang}/{pin}")),
  };

  if let Ok(true) = queries_path.try_exists() {
    let scm_files: HashSet<_> = queries_path
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
        "   {sign} {queries}",
        sign = check_sign(),
        queries = format!("{lang} queries").blue(),
      );
    } else if scm_count > 0 {
      println!(
        "   {sign} {queries}",
        sign = warn_sign(),
        queries = format!("{lang} queries").blue(),
      );
    } else {
      println!(
        "   {sign} {lang} queries missing; install with {help}",
        sign = no_sign(),
        help = format!("ktsctl manage -fi {lang}").bold()
      );
    }

    for q in queries {
      println!("{q}");
    }
  } else {
    let queries_path = install_dir.join(format!("queries/{lang}"));

    if let Ok(true) = queries_path.try_exists() {
      println!(
        "   {sync} {lang} queries out of sync; synchronize with {help}",
        sync = no_sign(),
        help = format!("ktsctl manage -is {lang}").bold()
      );
    } else {
      println!(
        "   {sign} {lang} queries missing; install with {help}",
        sign = no_sign(),
        help = format!("ktsctl manage -fci {lang}").bold()
      );
    }
  }
}

fn install_grammar(
  install_dir: &Path,
  lang_build_dir: &Path,
  lang: &str,
  pin: &str,
) -> Result<(), HellNo> {
  let lang_so = format!("{lang}.so");
  let source_path = lang_build_dir.join(lang_so);
  let grammar_dir = install_dir.join(format!("grammars/{lang}"));
  let install_path = grammar_dir.join(format!("{pin}.so"));
  let report = Report::new(ReportIcon::Install, format!("installing {lang} grammar"));

  // ensure the grammars directory exists
  fs::create_dir_all(&grammar_dir).map_err(|err| HellNo::CannotCreateDir {
    dir: grammar_dir,
    err,
  })?;
  fs::copy(&source_path, &install_path).map_err(|err| HellNo::CannotCopyFile {
    src: source_path,
    dest: install_path,
    err,
  })?;

  report.report(ReportIcon::Success, format!("installed {lang} grammar"));

  Ok(())
}

fn install_queries(
  install_dir: &Path,
  query_dir: &Path,
  lang: &str,
  pin: &str,
) -> Result<(), HellNo> {
  // ensure the queries directory exists
  let install_path = install_dir.join(format!("queries/{lang}/{pin}"));
  let report = Report::new(ReportIcon::Install, format!("installing {lang} queries"));

  fs::create_dir_all(&install_path).map_err(|err| HellNo::CannotCreateDir {
    dir: install_path.clone(),
    err,
  })?;

  copy_dir(query_dir, &install_path).map_err(|err| HellNo::CannotCopyDir {
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
