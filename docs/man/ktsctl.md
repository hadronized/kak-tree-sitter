# ktsctl

`ktsctl` is the CLI controller for `kak-tree-sitter`. It provides features such as downloading, compiling / linking and
installing grammars and queries. It is _optional_ but highly recommended.

You can configure it via the [configuration file](configuration.md).

## Commands

> Prerequisites:
>
> - The `cc` compiler, which should default to `gcc` on Linux for instance.
> - `git`, to download resources.
> - Some tree-sitter grammars require C++, so you will need to have the `libstd++` library installed.

Usage: `ktsctl [OPTIONS] <LANG>`

- `--help` should provide enough information on how to get started.
- `-f, --fetch` will make `ktsctl` cleanup fetch directory and download any resources required by the other commands. You
  can decide to first fetch resources for a given language, and then call the other commands later without fetching
  anymore; or you can combine everything at once.
- `-c, --compile` compiles and link grammars.
- `-i, --install` installs grammars and queries into `$XDG_DATA_HOME/kak-tree-sitter`. The install path can change
  depending on the operating system (for instance, it might be `$HOME/Library/Application\ Support` for macOS).
- `<LANG>` is the name of the language to install.

The list of language names you can instal can be found in your configuration file — e.g. `language.<LANG>`.

For instance, to fetch, compile and install the grammars for the Rust programming language:

```sh
ktsctl -fci rust
```

## By-passing `ktsctl` and using your own runtime resources

If you do not want to get into the rabbit hole of writing your own grammars and queries, you will most likely want to
reuse datasets from other projects, such as the GitHub repositories of the grammars themselves (for instance,
[tree-sitter-rust] for Rust), [Neovim] or [Helix].

Whatever you decide to use, you need to update your [user configuration](configuration.md) file according to the
languages, grammars and queries. **However**, as explained in the linked section, 99% of people will just be satisfied
with the default settings shipped with `kak-tree-sitter`’s `default-config.toml`, which you can find at the root of the
repository, [here](https://github.com/hadronized/kak-tree-sitter/blob/master/default-config.toml).

# A note for release channels

Release channels ([AUR], [brew], etc.) have the right to ship the `default-config.toml` file (or any of their liking)
along with both binaries. It is highly recommended that they follow SemVer (if they bundle `kak-tree-sitter v0.4.3`,
they should be named `kak-tree-sitter-0.4.3` for instance). However, adding an extra number for the bump of the
`default-config.toml` could be a good idea (so `kak-tree-sitter-0.4.3-1`, `kak-tree-sitter-0.4.3-2`, etc.).

[tree-sitter-rust]: https://github.com/tree-sitter/tree-sitter-rust/tree/master/queries
[Neovim]: https://github.com/nvim-treesitter/nvim-treesitter/tree/master/queries
[Helix]: https://github.com/helix-editor/helix/tree/master/runtime/queries
[AUR]: https://aur.archlinux.org
[brew]: https://brew.sh
