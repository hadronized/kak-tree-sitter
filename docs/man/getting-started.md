# Getting started

Welcome! If you are reading this page, it is likely it is the first time you hear about this project. First and
foremost, thank you for considering using this project. This section will provide you with a bunch of explanations about
how the project is made, how it works, what you should expect and what you should not.

## Prerequisites

As the name implies, this project bridges [tree-sitter] and [Kakoune]. You do not have to know anything about
the former, but obviously, you need to have [Kakoune] installed. The rest is handled by this project.

## What is this?

This project bridges [tree-sitter] and [Kakoune] via a server, that can be run on the CLI or inside [Kakoune] as a
daemon. The server is unique to your machine, so it doesn’t matter that you start it from several Kakoune sessions, only
one server will be up (this is implemented via a _PID file_). When the server is up, it remains up until the last
session exits (if started from within Kakoune), or when you send it the `SIGINT` signal (_Ctrl-C_) if started on the
CLI.

When the server is started from Kakoune, it will install some commands and hooks to be able to communicate with the
server. The configuration allows to decide which commands, hooks, user-modes etc. you want to have installed.

A running server will wait for a session to communicate with it requests, and will reply asynchronously. That prevents
your editor from freezing and allows the server to maintain a state / view of your per-session buffers. Because it uses
[tree-sitter] under the hood, it will parse the content of your buffers as _trees_, allowing for fast operations.

> Several features are supported / will be supported. You can refer to the [Features](features.md) document to know more
> about what you can do with the [tree-sitter] integration.

In order to work, the server requires some runtime resources to be around. This project doesn’t ship with those
resources, and you are free to use whichever sources you want. Two main kind of resources are needed:

- _Grammars_, which are C dynamic libraries loaded at runtime by the server and used to parse your buffer and operate on
  trees.
- _Queries_, which are [scm] strings allowing to _query_ parsed trees with a Lisp-like syntax. We use those to, mostly,
  unify the type of queries to all languages/grammars, such as highlighting, text-objects, indents, etc.

You can either find those resources yourself, or use the provided controller companion of the server that provides that
for free for you. See the [ktsctl](ktsctl.md) document.

## What you should expect

The project was made in a way that supports most of the features that are natively available in other editors regarding
[tree-sitter], such as [Helix]. It tries to be frustration-free, shipping with a default sane configuration; you should
and will probably never modify the configuration, but if you need to, you can. The server and the [Kakoune] side of the
project is made in a way that puts performance at top priority. Because we are using a UNIX approach, we try to minimize
as much as possible process starts, memory writes, etc., approaching as much as possible memory-to-memory communication.
Finally, the controller should do everything for you, provided you instruct what you want. You are you not required to
know _anything_ about [tree-sitter], nor how a query works or even how to compile a grammar. We do that for you. Just
focus on what’s important; your own projects.

## What you should not expect

The server is mostly about _transformations_. It doesn’t ship with _data_. Hence, you will not get grammars and queries
shipped with the binary by default. It is important that you understand that; it works out of the box, but you still
need to tell it what you want at first.

Colorschemes support is hard. Using this project won’t automatically make your colorscheme look super nice. You will
have to adapt and use a new, [tree-sitter]-powered colorscheme. The community tries its best to provide more and more
colorschemes. You shall want to visit the [Colorscheme](colorscheme.md) document for further information.

## What’s the next thing to read?

Now that you are aware about what the project is, what it is not, and the prerequisites, you may want to go to
[How to install](how-to-install.md).

[tree-sitter]: https://tree-sitter.github.io/tree-sitter/
[Kakoune]: https://kakoune.org/
[scm]: https://en.wikipedia.org/wiki/SCM_(Scheme_implementation)
[Helix]: https://helix-editor.com
