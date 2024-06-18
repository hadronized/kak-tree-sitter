# v2.1.0

## Language support

- Add support for Koka. [6bab165](https://git.sr.ht/~hadronized/kak-tree-sitter/commit/6bab165)

# v2.0.0

This release is needed for a small fix in the API for error handling purposes.
The `ConfigError` now contains a `MissingLang` variant that is used when
retrieving the configuration of a language
(`Result<LanguageConfig, ConfigError>`), while it used to be
`Option<LanguageConfig>`.

This change has implication on `ktsctl`, but doesn’t have any on
`kak-tree-sitter`.

## API

- Move missing language error as part of `kak-tree-sitter-config`. [cd35f75](https://git.sr.ht/~hadronized/kak-tree-sitter/commit/cd35f75)

# v1.0.0

- Fix tests.
- Add features in the config.
  This allows to automatically set highlighting and text-objects by default,
  preventing users from having to pass `--with-highlighting` and
  `--with-text-objects` all the time.

  The CLI still has precedence.
- Add astro to default config.
- Update tree-sitter-llvm patch version.
- Add LLVM config.
- Update MSRV and dependencies.
- Zig grammar / queries.
- Move to sr.ht.
- Add `nim` to default-config.toml

# v0.5.0

- Introduce user-only configuration. [fc7c5c6](https://git.sr.ht/~hadronized/kak-tree-sitter/commit/fc7c5c6)
- Introduce `ktsctl` sources. [e083aad](https://git.sr.ht/~hadronized/kak-tree-sitter/commit/e083aad)

# v0.4.0

- Add `remove_default_highlighter` option [d78abc0](https://git.sr.ht/~hadronized/kak-tree-sitter/commit/d78abc0)

# v0.3.0

- `<lost changelog, sorry :(>`

# v0.2.0

- Introduce proper error handling.
- Rework the config.
  - Most of the configuration doesn’t have a default value anymore, but a file is provided in the Git repository.
  - `uri_fmt` is replaced by `url` in both grammars and queries configuration.

# v0.1.0

- Initial release.
