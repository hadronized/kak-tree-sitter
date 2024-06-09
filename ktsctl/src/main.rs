use std::collections::HashSet;

use clap::Parser;
use cli::Cli;
use colored::Colorize;
use error::HellNo;
use kak_tree_sitter_config::Config;

use crate::commands::{
  manage::{ManageFlags, Manager},
  query::Query,
};

mod cli;
mod commands;
mod error;
mod git;
mod process;
mod resources;
mod ui;

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
    cli::Cmd::Fetch { all, lang } => {
      manage(config, true, false, false, false, all, lang.as_deref())?
    }

    cli::Cmd::Compile { all, lang } => {
      manage(config, false, true, false, false, all, lang.as_deref())?
    }

    cli::Cmd::Install { all, lang } => {
      manage(config, false, false, true, false, all, lang.as_deref())?
    }

    cli::Cmd::Sync { all, lang } => {
      manage(config, false, false, false, true, all, lang.as_deref())?
    }

    cli::Cmd::Query { lang, all } => {
      let query = Query::new(config)?;
      if let Some(lang) = lang {
        let sections = query.lang_info_sections(lang.as_str());
        for sct in sections {
          println!("{sct}");
        }
      } else if all {
        let all_tbl = query.all_lang_info_tbl();
        println!("{all_tbl}");
      }
    }
  }

  Ok(())
}

fn manage(
  config: Config,
  fetch: bool,
  compile: bool,
  install: bool,
  sync: bool,
  all: bool,
  lang: Option<&str>,
) -> Result<(), HellNo> {
  let manage_flags = ManageFlags::new(fetch, compile, install, sync);

  if let Some(lang) = lang {
    let manager = Manager::new(config, manage_flags)?;
    manager.manage(lang)?;
  } else if all {
    let all_langs: HashSet<_> = config.languages.language.keys().cloned().collect();
    let manager = Manager::new(config, manage_flags)?;
    manager.manage_all(all_langs.iter().map(|s| s.as_str()));
  }

  Ok(())
}
