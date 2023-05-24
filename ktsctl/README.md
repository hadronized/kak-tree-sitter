# ktsctl, `kak-tree-sitter` CLI controller

`ktsctl` is, as the name implies, a controller for `kak-tree-sitter`. It’s the tool you should be using to interact
with the data files `kak-tree-sitter` will be using to operate correctly (grammars, queries, etc.).

- [Special note](#special-note)
- [Features](#features)

## Special note

`ktsctl` is _optional_, it is **not mandatory to use it to use `kak-tree-sitter`**. However, it is highly recommended,
because it will perform boring operations for you automatically, and it comes with good defaults.

## Features

- Automatically fetch online resources. It uses `git clone` (`git` from your system) for that. It currently supports
  two types of resources:
  - Grammars.
  - Queries.
- Compile and link grammars. Requires `cc` to be available on your system.
- Install grammars and queries inside your data directories — in `$XDG_DATA_DIR/kak-tree-sitter`.
- Share the same configuration file as `kak-tree-sitter`.
