# How to contribute

Everyone is welcome to contribute. There is no small contributions. Have a look at the
[issue tracker](https://github.com/hadronized/kak-tree-sitter/issues), comment on it that you would like to work on the
topic and open a PR linking to the issue once you are ready to get your work merged.

## Contributing language support

Language support is mainly done via two files:

- The default configuration file.
- The `kak-tree-sitter.kak` file, if you need to add capture groups.

The configuration file already contains a wide variety of examples. Just clone the two sections for a given language
(including `language.<lang>.grammar` and `language.<lang>.queries`) and adjust as needed.

**An important guideline here**: [Helix](https://helix-editor.com/) is a well appreciated editor and their queries are
pretty excellent. You are highly suggested to reuse their work and point the `url` of the configuration you add to
their repository. For `queries`, the `url` is most likely to always be `https://github.com/helix-editor/helix`, and the
`path` something like `runtime/queries/<lang>`. Please fill in the `pin` option for both `queries` and `grammar`.

For the grammar to use, you are advised to look in
[this Helix’ languages.toml](https://github.com/helix-editor/helix/blob/master/languages.toml) file. It even has the
`rev` (i.e. what we call `pin`) to use.

Sometimes, the queries from Helix are not enough, because it contains Helix’ specificities (like capture nodes for
injections that is not exactly what we use, or `; inherits` annotations). In such cases, you need to check the queries
in in [our runtime/queries](./runtime/queries) directory. Refer to already existing queries for the process **but you
must credit the source, license terms and copy the `LICENSE` file in the language directory**.

## Contributing to Rust code

### Tooling

In terms of tooling, you should be using a recent Rust compiler, along with `rustfmt`. Always format your code, as the
CI will test that your code is correctly formatted.

### Commit and PR hygiene

Please refrain from creating gigantic commits. I reserve the right to refuse your PR if it’s not atomic enough: I
engage my spare-time to review and understand your code so **please please** keep that in mind.

There is no limit on the number of commits per PR, but keep in mind that PR should still remain small enough to be
easily reviewable. Try to scope a PR down to a single “domain.”
