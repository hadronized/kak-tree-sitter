# How to contribute

Everyone is welcome to contribute. There is no small contributions. Please take the time to read this document before
starting.

## Important links

There are several places where you hang around to both know what you can work on, and how you can communicate about it.
First thing first, you need to know that communication happen by **plain text** emails, and so, you might want to
subscribe to the following mailing lists:

- The [discuss list](https://lists.sr.ht/~hadronized/kak-tree-sitter-discuss), that you can use to ask any question,
  share your experience, etc.
- The [devel list](https://lists.sr.ht/~hadronized/kak-tree-sitter-devel), that you will have to use to send patches and
  talk about the development of the project. More on that below.
- The [announcement list](https://lists.sr.ht/~hadronized/kak-tree-sitter-announce), which is used to communicate about
  new releases and various official announcements about the project.

Additionally, you want to have a look at the following trackers:

- The [feature tracker](https://todo.sr.ht/~hadronized/kak-tree-sitter-features), that you can use to create feature
  requests.
- The [bug tracker](https://todo.sr.ht/~hadronized/kak-tree-sitter-bugs), where you can report a bug.

## How to start contributing

There are two main ways of contributing:

- You read the feature/bug tracker and spotted something no one is actively working on.
- You have a new feature requests or you have found a new bug.

Whatever you decide, a ticket should exist for what you are working on, so that it’s easier for everyone to know what’s
being worked on. So if you find a bug, please open an issue on the bug tracker first before even trying to fix it. Who
knows, maybe that bug is already fixed by someone’s else patch?

## Contribution format

We implement an email-based git flow; summary:

1. Clone the project
2. Ensure to switch to the `master` branch, and keep in sync with the remote branch (`git pull --rebase` or
  `git fetch <upstream>` + `git rebase <upstream>/master`.
3. Create your commits, either on your `master` branch, or feature branches. That part of the workflow is totally local
  and up to you.
4. Once you are done, generate a patch and email it to the `~hadronized/kak-tree-sitter-devel@lists.sr.ht` development
  list.

More on how to contribute via email [here](https://git-send-email.io).

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

## General contribution advices

### Tooling

In terms of tooling, you should be using a recent Rust compiler, along with `rustfmt`. If you are not sure, update
your toolchain with `rustup update` and ensure you have `rustfmt` installed with `rustup component add rustfmt`.

Always format your code, as the CI will test that your code is correctly formatted. The CI also enforces `clippy`, so
you should have it installed and check your code with it.

### Commit and PR hygiene

Please refrain from creating gigantic commits. I reserve the right to refuse your patch if it’s not atomic enough: I
engage my spare-time to review and understand your code so **please** keep that in mind.

There is no limit on the number of commits per patch, but keep in mind that individual commits should still remain small
enough to be easily reviewable. Try to scope a patch down to a single ticket, or even subpart of a ticket if you think
it makes sense.

Also, remember to include the ticket link in your commit, and to write concise but acute commit messages. Those are
used for writing changelog, so please keep that in mind.

Finally, **merging `master` into your branch is not appreciated**. If you want to “synchronize” your work with the
recent changes, please use `git rebase origin/master` and force-push to your remote branch. Please read on the TODO
tooling.

