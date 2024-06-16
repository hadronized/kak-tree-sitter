//! Module to remove resources.

use std::fs;

use colored::Colorize;
use kak_tree_sitter_config::Config;

use crate::{
  error::HellNo,
  resources::Resources,
  ui::{report::Report, status_icon::StatusIcon},
};

/// Delete resources associated with a given language.
pub fn remove(
  config: &Config,
  resources: &Resources,
  grammar: bool,
  queries: bool,
  prune: bool,
  lang: impl AsRef<str>,
) -> Result<(), HellNo> {
  let lang = lang.as_ref();
  let lang_config = config.languages.get_lang_config(lang)?;
  let report = Report::new(StatusIcon::Sync, format!("removing resources for {lang}"));
  let mut errors = Vec::new();

  if grammar {
    remove_grammar(resources, lang, lang_config, prune, &report, &mut errors);
  }

  if queries {
    remove_queries(resources, lang, lang_config, prune, &report, &mut errors);
  }

  if errors.is_empty() {
    report.success(format!("{lang} removed"));
  } else {
    report.error(format!("cannot remove {lang}"));

    for err in errors {
      eprintln!("{}", err.red());
    }
  }

  Ok(())
}

fn remove_grammar(
  resources: &Resources,
  lang: &str,
  lang_config: &kak_tree_sitter_config::LanguageConfig,
  prune: bool,
  report: &Report,
  errors: &mut Vec<String>,
) {
  if prune {
    let dir = resources.grammars_dir(lang);
    if let Ok(true) = dir.try_exists() {
      report.info(format!("removing {lang} grammar"));

      if let Err(err) = fs::remove_dir_all(dir) {
        errors.push(format!("cannot remove {lang} grammar: {err}"));
      }
    }
  } else {
    let grammar_path = resources.grammar_path_from_config(lang, lang_config);

    if let Ok(true) = grammar_path.try_exists() {
      report.info(format!("removing {lang} grammar"));

      if let Err(err) = fs::remove_file(grammar_path) {
        errors.push(format!("cannot remove {lang} grammar: {err}"));
      }
    }
  }
}

fn remove_queries(
  resources: &Resources,
  lang: &str,
  lang_config: &kak_tree_sitter_config::LanguageConfig,
  prune: bool,
  report: &Report,
  errors: &mut Vec<String>,
) {
  let dir = if prune {
    Some(resources.queries_dir(lang))
  } else {
    resources.queries_dir_from_config(lang, lang_config)
  };

  if let Some(dir) = dir {
    if let Ok(true) = dir.try_exists() {
      report.info(format!("removing {lang} queries"));

      if let Err(err) = fs::remove_dir_all(dir) {
        errors.push(format!("cannot remove {lang} queries: {err}"));
      }
    }
  }
}
