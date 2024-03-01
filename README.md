# kak-tree-sitter

This is a binary server that interfaces [tree-sitter](https://tree-sitter.github.io/) with
[kakoune](https://kakoune.org/).

[![asciicast](https://asciinema.org/a/606062.svg)](https://asciinema.org/a/606062)

- [Features](#features)
- [Install](#install)
- [Usage](#usage)
- [Alternatives](#alternatives)
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

## Roadmap

See [the milestones](https://github.com/hadronized/kak-tree-sitter/milestones).

## Install

See the wiki section about [how to install](https://github.com/hadronized/kak-tree-sitter/wiki/Install).

## Usage

See the wiki part about [the usage](https://github.com/hadronized/kak-tree-sitter/wiki/Usage).

## Alternatives

- [tree-sitter.kak](https://github.com/enricozb/tree-sitter.kak): a similar project, with the same motivations. It’s
  currently the only viable alternative with both features (semantic highlighting / selections).

## Credits

This program was heavily inspired by [kak-tree](https://github.com/ul/kak-tree), by [@ul](https://github.com/ul).
