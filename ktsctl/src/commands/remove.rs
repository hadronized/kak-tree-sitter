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
  lang: impl AsRef<str>,
) -> Result<(), HellNo> {
  let lang = lang.as_ref();
  let lang_config = config.languages.get_lang_config(lang)?;
  let report = Report::new(StatusIcon::Sync, format!("removing resources for {lang}"));
  let mut errors = Vec::new();

  if grammar {
    remove_grammar(resources, lang, lang_config, &report, &mut errors);
  }

  if queries {
    remove_queries(resources, lang, lang_config, &report, &mut errors);
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

fn remove_queries(
  resources: &Resources,
  lang: &str,
  lang_config: &kak_tree_sitter_config::LanguageConfig,
  report: &Report,
  errors: &mut Vec<String>,
) {
  if let Some(queries_dir) = resources.queries_dir_from_config(lang, lang_config) {
    if let Ok(true) = queries_dir.try_exists() {
      report.info(format!("removing {lang} queries"));

      if let Err(err) = fs::remove_dir_all(queries_dir) {
        errors.push(format!("cannot remove {lang} queries: {err}"));
      }
    }
  }
}

fn remove_grammar(
  resources: &Resources,
  lang: &str,
  lang_config: &kak_tree_sitter_config::LanguageConfig,
  report: &Report,
  errors: &mut Vec<String>,
) {
  let grammar_path = resources.grammar_path_from_config(lang, lang_config);

  if let Ok(true) = grammar_path.try_exists() {
    report.info(format!("removing {lang} grammar"));

    if let Err(err) = fs::remove_file(grammar_path) {
      errors.push(format!("cannot remove {lang} grammar: {err}"));
    }
  }
}
