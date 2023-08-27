# kak-tree-sitter

This is a binary server that interfaces [tree-sitter](https://tree-sitter.github.io/) with
[kakoune](https://kakoune.org/).

- [Features](#features)
- [Install](#install)
- [Usage](#usage)
- [Design](#design)
- [Alternatives](#alternatives)
- [Credits](#credits)

## Features

- [x] **Semantic highlighting.**
- [ ] **Semantic selections (types, functions, declarations, etc.)**
- Efficient parsing via `tree-sitter`, with partial parsing, etc.
- Shared between Kakoune sessions.

## Roadmap

See [the milestones](https://github.com/phaazon/kak-tree-sitter/milestones).

## Install

See the wiki section about [how to install](https://github.com/phaazon/kak-tree-sitter/wiki/Install).

## Usage

See the wiki part about [the usage](https://github.com/phaazon/kak-tree-sitter/wiki/Usage).

## Design

- [Overall design](./doc/design.md)

## Alternatives

- [tree-sitter.kak](https://github.com/enricozb/tree-sitter.kak): a similar project, with the same motivations. It’s
  currently the only viable alternative with both features (semantic highlighting / selections).

## Credits

This program was heavily inspired by [kak-tree](https://github.com/ul/kak-tree), by [@ul](https://github.com/ul).
