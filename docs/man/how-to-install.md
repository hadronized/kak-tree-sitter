# How to install

The project contains two binaries:

- `kak-tree-sitter`, which is both the UNIX server and client.
- `ktsctl`, the CLI controller. It is used to install various runtime resources related to `kak-tree-sitter`, such as
  grammars and queries. It is **optional** and you can install all the required resources manually if you’d rather like.

Depending on your operating system, you should target a _release channel_ and a package manager to use.

- [crates.io](#crates-io)
- [brew](#brew)

## crates.io

This is the initial way of installing the project, and should only be used if you have no other option.

```sh
cargo install kak-tree-sitter
cargo install ktsctl
```

## brew

A [brew](https://brew.sh) formula by [@rosingrind](https://github.com/rosingrind).

```sh
brew tap rosingrind/kak-tree-sitter
brew install kak-tree-sitter
brew install ktsctl
```

Head over to [the README](https://github.com/rosingrind/homebrew-kak-tree-sitter) for futher information.

# What’s next

Once you have installed the server and the optional controller, go on reading with [Usage](usage.md).
