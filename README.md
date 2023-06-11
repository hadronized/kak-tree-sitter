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

Currently, the only installation channel, which is not ideal, is via `cargo`:

```sh
cargo install kak-tree-sitter
```

Optionally, you can install the CLI controller:

```sh
cargo install ktsctl
```

## Usage

See the [wiki](https://github.com/phaazon/kak-tree-sitter/wiki).

## Design

- [Overall design](./doc/design.md)

## Alternatives

- [tree-sitter.kak](https://github.com/enricozb/tree-sitter.kak): a similar project, with the same motivations. It’s
  currently the only viable alternative with both features (semantic highlighting / selections).

## Credits

This program was heavily inspired by [kak-tree](https://github.com/ul/kak-tree), by [@ul](https://github.com/ul).
