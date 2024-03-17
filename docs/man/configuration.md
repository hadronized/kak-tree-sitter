# Configuration

Both `kak-tree-sitter` and `ktsctl` ship with a default configuration. It is possible to override the default
configuration via the user configuration.

The `$XDG_CONFIG_HOME/kak-tree-sitter/config.toml` contains the user configuration of both `kak-tree-sitter` and
`ktsctl`, which is shared. If you want to tweak something, you can have a look at the
[default configuration file](https://github.com/phaazon/kak-tree-sitter/blob/master/default-config.toml) to know which
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

- `url`: the URL to fetch from. Will use `git clone`.
- `pin`: _pin ref_, such as a commit, branch name or tag. **Highly recommended** to prevent breakage.
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

- `url`: the URL to fetch from. Will use `git clone`.
- `pin`: _pin ref_, such as a commit, branch name or tag. **Highly recommended** to prevent breakage.
- `path`: path where to find the queries (the `.scm` files) directory.
