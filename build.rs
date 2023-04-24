fn main() {
  // all the supported languages are stored in ./grammars; if you want to add support for another language, you shoul just
  // add the language grammar in that directory and simply just build again
  let grammars = std::fs::read_dir("./grammars/").expect("grammars");

  for grammar in grammars.into_iter().flatten() {
    // compile the grammar
    let dir = grammar.path().join("src");
    let grammar_name_os = grammar.file_name();
    let grammar_name = grammar_name_os.to_str().expect("grammar name");

    cc::Build::new()
      .include(&dir)
      .file(dir.join("parser.c"))
      .file(dir.join("scanner.c"))
      .compile(grammar_name);
  }
}
