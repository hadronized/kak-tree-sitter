# How to contribute

Everyone is welcome to contribute. There is no small contributions. Please take
the time to read this document before starting.

## Table of Content

- [Prerequisites](#prerequisites)
- [Setup](#setup)
- [How to find what to work on](#how-to-find-what-to-work-on)
- [Contribution workflow](#contribution-workflow)
  - [Contributing language support](#contributing-language-support)
- [Commit hygiene](#commit-hygiene)

## Prerequisites

Before contributing, some prerequisites:

- You must have [git] installed, as this project uses it as VCS.
- Rust is used to compile everything; you should have [rustup] installed.
- Especially, you want to have `rustc`, `cargo`, `clippy`, `rust-analyzer` and
  `rustfmt` installed.
- This project accepts contributions via _git patches_. It is likely that you
  are not used to this workflow. A mail client that can send emails in
  plain-text mode is highly advised — for instance, [aerc]. More on that in
  the [Guidelines](#guidelines) section.
- Not mandatory but highly recommended; you should have a GPG key hosted on a
  third-party location — for instance, [keys.openpgp.org] — and sign your
  emails with it. More on that in the the [Guidelines](#guidelines) section.

## Setup

Before starting up, you need to setup your tools.

### git send-email

You should follow [this link](https://git-send-email.io/) as a first source of
information on how to configure `git send-email`. Additionally, you want to
setup the per-project part.

Contributions must be sent to <~hadronized/kak-tree-sitter-devel@lists.sr.ht>.
Instead of using the `--to` flag everytime you use `git send-email`, you should
edit the local configuration of your repository with:

```sh
git config --local sendemail.to "~hadronized/kak-tree-sitter-devel@lists.sr.ht"
```

You also must set the prefix to `PATCH kak-tree-sitter` — that helps reviewing
and it is also mandatory for the CI to run:

```sh
git config --local format.subjectprefix "PATCH kak-tree-sitter"
```

Once this is done, all you have to do is to use `git send-email` normally.

> Note: if you would rather go your webmail instead, **ensure it does plain
> text**, and use `git format-patch` accordingly.

## How to find what to work on

You can first check the list of [bugs] and [features] on the bug trackers. If
you cannot find your issue there, you should open one. You can use the UI, or
simply send an email to the appropriate tracker:

- For bugs, send your email to <~hadronized/kak-tree-sitter-bugs@todo.sr.ht>.
- For features, send your email to <~hadronized/kak-tree-sitter-features@todo.sr.ht>.

If you are not sure, you can still open a discussion on the
[discuss mailing list], by using the UI or sending an email to
<~hadronized/kak-tree-sitter-discuss@lists.sr.ht>.

## Contribution workflow

You have found something to work on and want to start contributing. Follow these
simple steps:

1. Ensure you have followed the steps in the [Setup](#setup) section.
2. Clone the repository.
3. Create a branch; it will help when sending patches upstream.
4. Make your changes and make some commits!
5. Once ready to get your changes reviewed, send them with
  `git send-email master`.
6. Wait for the review and check your inbox.

If your change was accepted, you should have an email telling you it was
applied. If not, you should repeat the process.

> Note: please use the `--annotate -v2` flag of `git send-enail` if pushing a
> new version. `-v3` for the next one, etc. etc.

### Contributing language support

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

There is no limit on the number of commits per patch, but keep in mind that
individual commits should still remain small enough to be easily reviewable. Try
to scope a patch down to a single ticket, or even subpart of a ticket if you
think it makes sense.

Also, remember to include the ticket link in your commit, and to write concise
but acute commit messages. Those are used for writing changelog, so please keep
that in mind. Keep the line width to 80-char if possible.

Finally, **merging `master` into your branch is not appreciated**, and will end
up with your patch refused. If you want to “synchronize” your work with the
recent changes, please use `git rebase origin/master`.

[git]: https://git-scm.com/
[rustup]: https://rustup.rs/
[aerc]: https://aerc-mail.org/
[keys.openpgp.org]: https://keys.openpgp.org/
[bugs]: https://todo.sr.ht/~hadronized/kak-tree-sitter-bugs
[features]: https://todo.sr.ht/~hadronized/kak-tree-sitter-features
[discuss mailing list]: https://lists.sr.ht/~hadronized/kak-tree-sitter-discuss
