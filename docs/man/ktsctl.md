# ktsctl

`ktsctl` is the CLI controller for `kak-tree-sitter`. It provides features such
as downloading, compiling / linking and installing grammars and queries. It is
**optional** but highly recommended.

You can configure it via the [configuration file](configuration.md).

## Commands

> Prerequisites:
>
> - The `cc` compiler, which should default to `gcc` on Linux for instance.
> - `git`, to download resources.
> - Some tree-sitter grammars require C++, so you will need to have the `libstd++` library installed.

Usage: `ktsctl <COMMAND>`. Commands can be:

- `fetch` used to fetch runtime resources.
- `compile` used to compile runtime resources. You should have fetched them
  first though.
- `install` used to install runtime resources. For grammars, you should compile
  them first.
- `sync` used to synchronize resources — fetching, compiling, installing and
  checking pinned resources all at once.
- `query`, used to get information about the languages configuration, installed
- resources, etc.

### Managing resources

You can fetch, compile and install resources manually with the `ktsctl fetch`,
`ktsctl compile` and `ktsctl install` commands. However, a much more
straightforward command to use is `ktsctl sync`, which does all of those at
once, plus the additional feature of being idempotent.

A typical one-liner to install a language is `ktsctl sync <LANG>`, or

> The list of language names you can installed can be found with the
> [info command](#getting-information).

For instance, to fetch, compile and install the grammar and queries for the Rust
programming language:

```sh
ktsctl sync rust
```

Once a language is installed, you will probably eventually update `ktsctl`, and
if you haven’t set a specific `pin` for this language, you can benefit from the
default shipped configuration updates by using the `sync` command again.

#### Synchronizing everything

If there was one command you should remember, it’s this one:

```sh
ktsctl sync -a
```

It synchronizes everything. Basically, it will:

- Ensure all the languages from the configuration are cloned / fetched.
- If you don’t have a resource for one of them, the resource is fetched,
  compiled, installed.
- If you do, the resource is checked against the pinned version of the
  configuration. If it’s too old, a new version is fetched, compiled, etc.

> **Important note**: for `--all`, the runtime directory might get completely
> filled, as grammars weigh a lot. If you encounter error, you can cleanup the
> runtime directory, and run the command again.

### Getting information

The `query` command allows to get information about tree-sitter resources. As
with `manage`, you can use `--help` to know what you can do, but here are some
useful commands:

- `ktsctl query rust` will provide information about a specific language
  (here Rust). It will print out various configuration options, as well as
  whether the grammar and queries are installed.
- `ktsctl query -a` will display a short summary for all available and
  recognized languages (based on the configuration).

## By-passing `ktsctl` and using your own runtime resources

If you do not want to get into the rabbit hole of writing your own grammars and
queries, you will most likely want to reuse datasets from other projects, such
as the GitHub repositories of the grammars themselves (for instance,
[tree-sitter-rust] for Rust), [Neovim] or [Helix].

Whatever you decide to use, you need to update your
[user configuration](configuration.md) file according to the languages, grammars
and queries. **However**, as explained in the linked section, 99% of people will
just be satisfied with the default settings shipped with `kak-tree-sitter`’s
`default-config.toml`, which you can find at the root of the repository,
[here](https://git.sr.ht/~hadronized/kak-tree-sitter/tree/master/item/kak-tree-sitter-config/default-config.toml).
That configuration file is shipped with the binary (`kak-tree-sitter`), so you
do not need to copy it; just add what you need!

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
