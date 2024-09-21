# How to contribute

Everyone is welcome to contribute. There is no small contributions. Please take
the time to read this document before starting. We are using [git] on [GitHub].
Please ensure to get familiarized with both before starting to contribute.

## Contributing language support

**An important guideline here**: [Helix](https://helix-editor.com/) is a well
appreciated editor and their queries are pretty excellent. You are highly
suggested to reuse their work and point the `url` of the configuration you add
to their repository. For `queries`, the `url` is most likely to always be
`https://github.com/helix-editor/helix`, and the `path` something like
`runtime/queries/<lang>`. **Please fill in the `pin` option for both `queries`
and `grammar`**.

For the grammar to use, you are advised to look in
[this Helix’ languages.toml](https://github.com/helix-editor/helix/blob/master/languages.toml)
file. It even has the `rev` (i.e. what we call `pin`) to use.

Sometimes, the queries from Helix are not enough, because it contains Helix’
specificities (like capture nodes for injections that is not exactly what we
use, or `; inherits` annotations). In such cases, you need to check the queries
in [our runtime/queries](./runtime/queries) directory. Refer to already existing
queries for the process **but you must credit the source, license terms and copy
the `LICENSE` file in the language directory**. FOSS should be respected; let us
be an example.

## Commit hygiene

Please refrain from creating gigantic commits. I reserve the right to refuse
your patch if it’s not atomic enough: I engage my spare-time to review and
understand your code so **please** keep that in mind.

There is no limit on the number of commits per PR, but keep in mind that
individual commits should still remain small enough to be easily reviewable. Try
to scope a PR down to a single issue, or even subpart of a issue if you think
it makes sense. Remember that PRs are often reviewed commits by commits, so
ensure a certain coherenc between what to put and what not to put in a commit.

Also, remember to include the issue number at the end of your commit message
with a leading dash — e.g. `#123` – and to write concise yet acute commit
messages. Those are used for writing changelog, so please keep them short.

Finally, **merging `master` into your branch is not appreciated**, and will end
up with your patch refused. If you want to “synchronize” your work with the
recent changes, please use `git fetch origin && git rebase origin/master` in
your branch.

## Sign your work

GPG signatures are used to sign our work. The value of signing a piece of code
doesn’t imply _you_ write it, but it implies you _validated that code_, and in
the end, we don’t really care whether you wrote the code or whether you
generated with a fancy A.I. generator. It’s the code you bring and it’s the code
you sign.

If you plan on contributing more than just one-shot contributions, feel free
to open a PR to modify the [MAINTAINERS.md] file by providing your name, email
address and PGP fingerprint.

[git]: https://git-scm.com/
[GitHub]: https://github.com/hadronized/kak-tree-sitter
[keys.openpgp.org]: https://keys.openpgp.org/
[MAINTAINERS.md]: MAINTAINERS.md
