//! Manage sub-commands.

use std::{fs, io, path::Path};

use colored::Colorize;
use kak_tree_sitter_config::{source::Source, Config, LanguageConfig};

use crate::{
  error::HellNo,
  git,
  process::Process,
  report::{Report, ReportIcon},
  resources::Resources,
};

/// Main flags to fetch, compile and/or install resources.
#[derive(Debug)]
pub struct ManageFlags {
  pub fetch: bool,
  pub compile: bool,
  pub install: bool,
  pub sync: bool,
}

impl ManageFlags {
  pub fn new(fetch: bool, compile: bool, install: bool, sync: bool) -> Self {
    Self {
      fetch,
      compile,
      install,
      sync,
    }
  }
}

#[derive(Debug)]
pub struct Manager {
  config: Config,
  flags: ManageFlags,
  resources: Resources,
}

impl Manager {
  pub fn new(config: Config, flags: ManageFlags) -> Result<Self, HellNo> {
    let resources = Resources::new()?;

    Ok(Self {
      config,
      flags,
      resources,
    })
  }

  pub fn manage(&self, lang: &str) -> Result<(), HellNo> {
    let lang_config =
      self
        .config
        .languages
        .get_lang_conf(lang)
        .ok_or_else(|| HellNo::MissingLangConfig {
          lang: lang.to_owned(),
        })?;

    self.manage_grammar(lang, lang_config)?;
    self.manage_queries(lang, lang_config)
  }

  pub fn manage_all<'a>(&self, langs: impl Iterator<Item = &'a str>) {
    for lang in langs {
      println!("working {}", lang.blue());
      let r = self.manage(lang);
      println!();

      if let Err(err) = r {
        println!("{err}");
      }
    }
  }

  fn manage_grammar(&self, lang: &str, lang_config: &LanguageConfig) -> Result<(), HellNo> {
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

      Source::Git { ref url, ref pin } => self.manage_git_grammar(lang, lang_config, url, pin)?,
    }

    Ok(())
  }

  fn manage_git_grammar(
    &self,
    lang: &str,
    lang_config: &LanguageConfig,
    url: &str,
    pin: &str,
  ) -> Result<(), HellNo> {
    let sources_path = self.resources.sources_dir(url);

    if self.flags.sync {
      let report = Report::new(ReportIcon::Sync, format!("syncing {lang} grammar"));
      self.sync_git_grammar(&report, lang, lang_config, &sources_path, url, pin)?;
      return Ok(());
    }

    if self.flags.fetch {
      let report = Report::new(ReportIcon::Fetch, format!("cloning {lang} grammar…"));
      Self::git_clone(&report, lang, &sources_path, url, pin)?;
      report.success(format!("cloned {lang} grammar"));
    }

    let lang_build_dir = self
      .resources
      .lang_build_dir(&sources_path, &lang_config.grammar.path);

    if self.flags.compile {
      let report = Report::new(ReportIcon::Compile, format!("compiling {lang} grammar"));
      Self::compile_git_grammar(&report, lang, lang_config, &lang_build_dir)?;
      report.success(format!("built {lang} grammar"));
    }

    if self.flags.install {
      let report = Report::new(ReportIcon::Install, format!("installing {lang} grammar"));
      self.install_git_grammar(&report, lang, &lang_build_dir, pin)?;
      report.success(format!("installed {lang} grammar"));
    }

    Ok(())
  }

  fn sync_git_grammar(
    &self,
    report: &Report,
    lang: &str,
    lang_config: &LanguageConfig,
    fetch_path: &Path,
    url: &str,
    pin: &str,
  ) -> Result<(), HellNo> {
    if self.resources.grammar_exists(lang, pin) {
      report.success(format!("grammar {lang} already installed ({pin})"));
      return Ok(());
    }

    Self::git_clone(&report, lang, fetch_path, url, pin)?;

    let lang_build_dir = self
      .resources
      .lang_build_dir(fetch_path, &lang_config.grammar.path);

    Self::compile_git_grammar(&report, lang, lang_config, &lang_build_dir)?;
    self.install_git_grammar(&report, lang, &lang_build_dir, pin)?;

    report.success(format!("synchronized {lang} grammar"));
    Ok(())
  }

  fn git_clone(
    report: &Report,
    lang: &str,
    fetch_path: &Path,
    url: &str,
    pin: &str,
  ) -> Result<(), HellNo> {
    let cloned = git::clone(&report, lang, fetch_path, url)?;

    if let git::Clone::Cloned = cloned {
      report.success(format!(
        "cloned {lang} at {path}",
        path = fetch_path.display(),
      ));
    } else {
      report.success(format!(
        "already cloned {lang} at {path} (cached)",
        path = fetch_path.display(),
      ));
    }

    git::fetch(report, lang, fetch_path, url, pin)
  }

  /// Compile and link the grammar.
  fn compile_git_grammar(
    report: &Report,
    lang: &str,
    lang_config: &LanguageConfig,
    lang_build_dir: &Path,
  ) -> Result<(), HellNo> {
    // ensure the build dir exists
    fs::create_dir_all(&lang_build_dir).map_err(|err| HellNo::CannotCreateDir {
      dir: lang_build_dir.to_owned(),
      err,
    })?;

    // compile
    let args: Vec<_> = lang_config
      .grammar
      .compile_args
      .iter()
      .map(|x| x.as_str())
      .chain(lang_config.grammar.compile_flags.iter().map(|x| x.as_str()))
      .collect();

    Process::new(&lang_config.grammar.compile).run(lang_build_dir, &args)?;

    report.info(format!("compiled {lang} grammar"));

    // link into {lang}.so
    let report = Report::new(ReportIcon::Link, format!("linking {lang} grammar",));
    let args: Vec<_> = lang_config
      .grammar
      .link_args
      .iter()
      .map(|x| x.as_str())
      .chain(lang_config.grammar.link_flags.iter().map(|x| x.as_str()))
      .collect();
    Process::new(&lang_config.grammar.link).run(lang_build_dir, &args)?;

    report.success(format!("linked {lang} grammar"));
    Ok(())
  }

  fn install_git_grammar(
    &self,
    report: &Report,
    lang: &str,
    lang_build_dir: &Path,
    pin: &str,
  ) -> Result<(), HellNo> {
    let report = Report::new(ReportIcon::Install, format!("installing {lang} grammar"));

    let lang_so = format!("{lang}.so");
    let source_path = lang_build_dir.join(lang_so);
    let grammar_dir = self.resources.data_dir().join(format!("grammars/{lang}"));
    let install_path = grammar_dir.join(format!("{pin}.so"));

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

    report.success(format!("installed {lang} grammar"));
    Ok(())
  }

  fn manage_queries(&self, lang: &str, lang_config: &LanguageConfig) -> Result<(), HellNo> {
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

      Some(Source::Git { ref url, ref pin }) => {
        self.manage_git_queries(lang, &lang_config, url, pin)?
      }

      None => {
        Report::new(
          ReportIcon::Error,
          format!("no query configuration for {lang}; will be using the grammar directory"),
        );
      }
    }

    Ok(())
  }

  fn manage_git_queries(
    &self,
    lang: &str,
    lang_config: &LanguageConfig,
    url: &str,
    pin: &str,
  ) -> Result<(), HellNo> {
    let sources_path = self.resources.sources_dir(url);

    if self.flags.sync {
      let report = Report::new(ReportIcon::Sync, format!("syncing {lang} queries"));
      self.sync_git_queries(&report, lang, lang_config, &sources_path, url, pin)?;
      report.success(format!("synchronized {lang} queries"));
      return Ok(());
    }

    if self.flags.fetch {
      let report = Report::new(ReportIcon::Fetch, format!("cloning {lang} queries"));
      Self::git_clone(&report, lang, &sources_path, url, pin)?;
    }

    if self.flags.install {
      let report = Report::new(ReportIcon::Install, format!("installing {lang} queries"));
      let query_dir = sources_path.join(&lang_config.queries.path);
      self.install_git_queries(&report, &query_dir, lang, pin)?;
    }

    Ok(())
  }

  fn sync_git_queries(
    &self,
    report: &Report,
    lang: &str,
    lang_config: &LanguageConfig,
    fetch_path: &Path,
    url: &str,
    pin: &str,
  ) -> Result<(), HellNo> {
    if self.resources.queries_exist(lang, pin) {
      report.success(format!("queries {lang} already installed ({pin})"));
      return Ok(());
    }

    Self::git_clone(&report, lang, fetch_path, url, pin)?;

    let path = &lang_config.queries.path;
    let query_dir = fetch_path.join(path);
    self.install_git_queries(&report, &query_dir, lang, pin)?;

    report.success(format!("synchronized {lang} grammar"));
    Ok(())
  }

  fn install_git_queries(
    &self,
    report: &Report,
    query_dir: &Path,
    lang: &str,
    pin: &str,
  ) -> Result<(), HellNo> {
    // ensure the queries directory exists
    let install_path = self.resources.queries_dir(lang, pin);
    let report = Report::new(ReportIcon::Install, format!("installing {lang} queries"));

    fs::create_dir_all(&install_path).map_err(|err| HellNo::CannotCreateDir {
      dir: install_path.clone(),
      err,
    })?;

    Self::copy_dir(query_dir, &install_path).map_err(|err| HellNo::CannotCopyDir {
      src: query_dir.to_owned(),
      dest: install_path,
      err,
    })?;

    report.success(format!("installed {lang} queries"));
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
}
