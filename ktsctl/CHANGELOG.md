# v0.3.3

- Support for kak-tree-sitter-config-v0.4.0

# v0.3.2

- Add --has to ktsctl. [baaf735](https://github.com/hadronized/kak-tree-sitter/commit/baaf735)

# v0.3.1

- Enhance CLI of ktsctl. [1b8fbbd](https://github.com/hadronized/kak-tree-sitter/commit/1b8fbbd)

# v0.3.0

- `<lost changelog, sorry :(>`

# v0.2.0

- Proper error handling.
- Remove `--query -q`. It is now inferred from the other arguments.
- The meaning of the various `path` configuration option has changed a bit. We do not magically insert more
  indirections from there. For instance, if the query config has a given directory set for `path`, that directory
  content will be copied to `$XDG_DATA_DIR/kak-tree-sitter/queries/<lang>`.


# v0.1.0

- Initial release.




