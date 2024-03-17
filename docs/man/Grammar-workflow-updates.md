If you do not want to get into the rabbit hole of writing your own grammars and queries, you will most likely want to
reuse datasets from other projects, such as the GitHub repositories of the grammars themselves (for instance,
[tree-sitter-rust] for Rust), [Neovim] or [Helix].

Whatever you decide to use, you need to update the [config.toml](Configuration) file according to the languages,
grammars and queries. **However**, as explained in the linked section, 99% of people will just be satisfied with the
default settings shipped with `kak-tree-sitter`’s `config.toml`, which you can find at the root of the repository;
[here](https://github.com/phaazon/kak-tree-sitter/blob/master/config.toml).

This file is not included with the releases of `kak-tree-sitter` nor `ktsctl`, which means that you need to keep it in
sync if you want to add support for new languages or fix grammars and queries you have already installed
(`ktsctl -fci rust` should be enough to update the Rust runtime files after updating `config.toml`).

# A note for release channels

Release channels ([AUR], [brew], etc.) have the right to ship the `config.toml` file (or any of their liking) along with
both binaries. It is highly recommended that they follow SemVer (if they bundle `kak-tree-sitter v0.4.3`, they should be
named `kak-tree-sitter-0.4.3` for instance). However, adding an extra number for the bump of the `config.toml` could be
a good idea (so `kak-tree-sitter-0.4.3-1`, `kak-tree-sitter-0.4.3-2`, etc.).

[tree-sitter-rust]: https://github.com/tree-sitter/tree-sitter-rust/tree/master/queries
[Neovim]: https://github.com/nvim-treesitter/nvim-treesitter/tree/master/queries
[Helix]: https://github.com/helix-editor/helix/tree/master/runtime/queries
[AUR]: https://aur.archlinux.org
[brew]: https://brew.sh
