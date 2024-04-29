# kak-tree-sitter

> **Archive note: the project is still active, but has moved to <https://sr.ht/~hadronized/kak-tree-sitter/>.**

This is a binary server that interfaces [tree-sitter](https://tree-sitter.github.io/) with
[kakoune](https://kakoune.org/).

> Important note: by default, no colorscheme supporting tree-sitter is set for you. You have to pick one or write your
> own. See [this section from the man](./docs/man/highlighting.md) for further information.

[![asciicast](https://asciinema.org/a/606062.svg)](https://asciinema.org/a/606062)

- [Features](#features)
- [Install](#install)
- [Usage](#usage)
- [Contributing](#contributing)
- [Credits](#credits)

## Features

- [x] Semantic highlighting.
  - Automatically detects whether a buffer language type can be highlighted.
  - Removes any default highlighter and replaces them with a tree-sitter based.
- [x] Semantic selections (types, functions, parameters, comments, tests, etc.)
  - Similar features to `f`, `?`, `<a-/>`, etc.
  - Full _object_ mode support (i.e. `<a-i>`, `{`, `<a-]>`, etc.)
- [ ] Indents
- [ ] Indent guidelines
- [ ] Incremental parsing
- [x] Fetch, compile and install grammars / queries with ease (via the use of the `ktsctl` controller companion)
- [x] Ships with no mappings, defined options, but allows to use well-crafted values, user-modes, mappings and
  commands by picking them by hand.
- [x] Transformation-oriented; actual data (i.e. grammars, queries, etc.) can be used from any sources.

## User manual

See the [User manual](docs/man) to know how to install, use, configure and get runtime resources.

## Contributing

Whether you want to fix a bug, make a feature request, help improving something or add support for a new language by
changing the default configuration, you should read the [CONTRIBUTING.md](./CONTRIBUTING.md) file.

## Credits

This program was inspired by:

- [Helix](https://helix-editor.com)
- [kak-tree](https://github.com/ul/kak-tree)
