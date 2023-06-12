# Runtime tree-sitter queries

The queries are not shipped with `kak-tree-sitter`. Instead, you have to manually install them. You have two solutions:

1. Deal with installing queries by yourself, or even hand-craft them. **This is not recommended.**
2. Use `ktsctl` and the `config.toml` file to install the queries by fetching them (`git clone`).

For the second option, the default [config.toml](/config.toml) is already configured to point to well-working grammars
and queries. You should copy it as `$XDG_CONFIG_HOME/kak-tree-sitter/config.toml`.

## Special case for locally modified queries

Some queries required to be checked-in in this repository, and manually modified. Most of the queries are taken
from [Helix](https://github.com/helix-editor/helix/tree/master/runtime/queries), and sometimes we have to adapt them
a little bit to make them work with `kak-tree-sitter`.

Those queries are available in [the runtime/queries](./queries) directory, along with a `README.md` explaining their
sources, the license used and a `LICENSE` file applying to all of the `.scm` files in the associated language directory.
