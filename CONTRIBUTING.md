# How to contribute

Everyone is welcome to contribute. There is no small contributions. Have a look at the
[issue tracker](https://github.com/hadronized/kak-tree-sitter/issues), comment on it that you would like to work on the
topic and open a PR linking to the issue once you are ready to get your work merged.

## Issues

Bugs, small improvements, feature requests, etc. are welcome, but because the scope of the project needs to remain
small, **you should open an issue to discuss both the matter and the potential design beforehand**. It’s up to you to
start working on an implementation, but remember that if your ideas / features don’t align with the project, your
contribution might get rejected and you will have worked on something for nothing.

That even applies to bug fixes, because sometimes, fixing something by slightly changing the design is far better than
blindly patching here and there.

## Contribution format

We use GitHub Issues and Pull Requests. You should create a branch that targets `master`, and name your branch
`<issue-nb>-<description>`. `<description>` should be small, and is free — it can contain `fix-blabla`, `design-foo`,
or even just `blabla` or `foo`. However, `issue-nb` should be a valid issue number, without the leading hash.

If you want to start something on something that doesn’t have an issue yet, please:

1. Search for a similar issue. It’s not impossible someone else opened an issue already.
2. If you haven’t found anything, please create an issue. That will give everyone the context of your PR, and you will
  get an issue number.
3. If you are fixing a small bug, you don’t have to wait for feedback. For anything else, you should explain the problem
  and synchronize with the maintainers; maybe there’s a simpler solution to your problem.
4. If your solution still seems to be the right approach, start working on it and open a PR when ready.

> Note: if you really insist on opening a PR to show some implementatino because you are almost sure we want to go that
> way, feel free to open a draft PR as you go through the implementation to show how you progress on it.

## Contributing language support

Language support is mainly done via two files:

- The default configuration file.
- The rc file, if you need to add capture groups.

The configuration file already contains a wide variety of examples. Just clone the two sections for a given language
(grammar and queries), and adjust as needed.

**An important guideline here**: [Helix](https://helix-editor.com/) is a well appreciated editor and their queries are
pretty excellent. You are highly suggested to reuse their work and point the `url` of the configuration you add to
their repository. For `queries`, the `url` is most likely to always be `https://github.com/helix-editor/helix`, and the
`path` something like `runtime/queries/<lang>`. **Please fill in the `pin` option for both `queries` and `grammar`**.

For the grammar to use, you are advised to look in
[this Helix’ languages.toml](https://github.com/helix-editor/helix/blob/master/languages.toml) file. It even has the
`rev` (i.e. what we call `pin`) to use.

Sometimes, the queries from Helix are not enough, because it contains Helix’ specificities (like capture nodes for
injections that is not exactly what we use, or `; inherits` annotations). In such cases, you need to check the queries
in [our runtime/queries](./runtime/queries) directory. Refer to already existing queries for the process **but you
must credit the source, license terms and copy the `LICENSE` file in the language directory**. FOSS should be respected;
let us be an example.

## Contributing to Rust code

### Tooling

In terms of tooling, you should be using a recent Rust compiler, along with `rustfmt`. Always format your code, as the
CI will test that your code is correctly formatted.

The CI also enforces `clippy`, so you should have it installed and check your code with it.

### Commit and PR hygiene

Please refrain from creating gigantic commits. I reserve the right to refuse your PR if it’s not atomic enough: I
engage my spare-time to review and understand your code so **please please** keep that in mind.

There is no limit on the number of commits per PR, but keep in mind that PR should still remain small enough to be
easily reviewable. Try to scope a PR down to a single “domain.”

By _gigantic commit_ we mean touching on too many different things. Examples of non-atomic commits:

- A commit that optimizes the usage of a scarce resource but also changes configuration.
- A commit that introduces a new feature but also improves some code somewhere else.
- A commit that bumps several dependencies’ version at once.
- A commit that bumps a dependency version and that contains changes not related to any breaking change of that bump.
- Etc.

Also, remember to include the issue number in your commit, and to write concise but acute commit messages. Those are
used for writing changelog, so please keep that in mind.

Finally, **merging `master` into your branch is not appreciated**. If you want to “synchronize” your work with the recent
changes, please use `git rebase origin/master` and force-push to your remote branch.

