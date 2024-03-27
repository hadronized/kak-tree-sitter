# Configuration

Both `kak-tree-sitter` and `ktsctl` ship with a default configuration. It is possible to override the default
configuration via the user configuration.

The `$XDG_CONFIG_HOME/kak-tree-sitter/config.toml` contains the user configuration of both `kak-tree-sitter` and
`ktsctl`, which is shared. If you want to tweak something, you can have a look at the
[default configuration file](https://github.com/hadronized/kak-tree-sitter/blob/master/default-config.toml) to know which
path and values to pick from.

> The user and default configurations get merged, so you do not have to copy the default configuration to tweak it.

# Option paths

## `highlight.groups`

The `highlight` section currently contains a single list, `groups`, which is used to list every capture groups used by
language queries. If you install a language with queries containing new capture groups not already listed there, you
need to add them at the end of the list.

> Please consider contributing if you find a hole / missing capture group.

## `language`

The `language` table contains language-keyed configuration — e.g. `language.rust`. Every language-keyed configuration
contains more objects.

- `remove_default_highlighter`, for removing the default highlighter set by the Kakoune distribution when enabling
  kak-tree-sitter support in a buffer.
- `grammar`, for defining a grammar.
- `queries`, for defining the language queries.

### `language.<lang>.remove_default_higlighter`

> Default value: `true`
 
Remove the default highlighter set by the Kakoune “standard library” (i.e. `window/<lang>`). For instance, for `rust`
filetypes, the default highlighter is `window/rust`. Setting this option to `true` will remove this highlighter, which
is almost always wanted (otherwise, the highlighting from KTS might not be applied properly).

Some languages might have an incomplete tree-sitter support; in such a case, you might not want to remove the default
highlighter. Set this option to `false` in such cases, then.

### `language.<lang>.grammar`

This section contains various information about how to fetch, compile and link a grammar:

- `source`: the source from where to pick the grammar; see the [Sources](#sources) section.
- `path`: path where to find the various source files. Should always be `src` but can require adjustments for
  monorepositories.
- `compile`: compile command to use. Should always be `cc`.
- `compile_args`: arguments to pass to `compile` to compile the grammar.
- `compile_flags`: optimization / debug flags.
- `link`: link command to use. Should alwas be `cc`.
- `link_args`: arguments to pass to `link` to link the grammar.
- `link_flags`: optimization / debug / additional libraries to link flags.

### `language.<lang>.queries`

This section provides the required data to know how to fetch queries.

- `source`: optional source from where to pick the queries; see the [Sources](#sources) section. If you omit it, the
  same `source` object is used for both the grammar and queries.
- `path`: path where to find the queries (the `.scm` files) directory.

# Sources

Sources are a way to provide information from where runtime resources come from. We currently support two sources:

- Local paths (`local.path`).
- And Git repositories (`git`), which is an object containing the following fields:
  - `url`: the URL to fetch from. Will use `git clone`.
  - `pin`: _pin ref_, such as a commit, branch name or tag.

If you decide to use a `git` source:

- Grammars must be _fetched_, _compiled_ and _installed_. `ktsctl` can do that automatically for you, provided you have
  the right configuration, by using the appropriate flags. See the documentation of [ktsctl](ktsctl.md).
- Queries must be _fetched_ and _installed_, the same way as with grammars.
- When you decide to install a “language”, both the grammars and queries might be fetched, compiled and installed if
  the configuration requires both to be. Hence, a single CLI command should basically do everything for you.

If you decide to use a `local` source, **`ktsctl` will do nothing for you** and will simply display a message explaining
that it will use a path. Nothing will be fetched, compiled nor installed. It’s up to you to do so.

For users installing `ktsctl` by using a binary release or compiling it themselves, the default configuration (which
uses mainly `git` sources) is enough. However, if you ship with a distributed set of grammars and queries, you might
want to override the languages’ configurations and use `local` sources. You can also mix them: a `git` source for the
grammar, and a `local` one for the queries. It’s up to you.
