# Overall design

`kak-tree-sitter` implements a client/server architecture. The idea is that a single instance of `kak-tree-sitter`
should run as a server for a machine. Then, every Kakoune session can connect to the server and send requests. Whenever
the server has computed the response for the request, it provides it back to Kakoune via the socket interface (`kak -p`)
and the session identifier.

For a given Kakoune session, the `kak-tree-sitter` server knows all of the buffers that are required to be parsed
by `tree-sitter`. This allows efficient reusing of parsed data, which is an important aspect of `tree-sitter`: parse
once, then reuse the parsed data whenever we need to parse again to minimize each delta update.

## Clientless run

When a Kakoune session starts, it is recommended to run `kak-tree-sitter` — via your `kakrc` for instance. If
`kak-tree-sitter` is not already running, either daemonized or not, an instance will be started. Then, `kak-tree-sitter`
will add an entry for the Kakoune session and it will then be possible for clients of that session to start sending
requests.

> Note: if a Kakoune instance tries to connect to the (running) `kak-tree-sitter` server with a session name that has
> already been registered, nothing happens, so there is no extra logic to do in the Kakoune configuration besides
> just starting the server with the session name.

If users do not want to start `kak-tree-sitter` inside their Kakoune configuration, it is still possible to start it
like any other CLI application. They will have to source the `rc` files manually to be able to talk to the server,
though.

> Note: sourcing without starting the daemon is easy with `kak-tree-sitter --kakoune`.

## Client requests

Client can then connect to the server. They have to provide a couple of information when performing a request:

- The Kakoune session name, obviously.
- The Kakoune client name, optional. Whenever a user wants to perform an operation that implies a Kakoune instance,
  the client is necessary. For instance, highlighting or selecting requires a client.

## Active and inactive sessions

The default `rc` file, if sourced manually (or injected with `kak-tree-sitter --kakoune`), will install some important
hooks to deal with sessions. At the first request a Kakoune does, its session will be recorded and marked as _active_ by
`kak-tree-sitter`.

A hook on `KakEnd` sends a special request to `kak-tree-sitter` to let it know that the session is over, marking it
_inactive_ — it’s actually completely removed from the server.

Once the server has an empty set of active session, it automatically exits. This allows to control precisely when the
server should quit.

> Note: this is only true if started from Kakoune, via `--kakoune`. If you start the server from the CLI, the server
> remains up even if no session is connected to it.

If a Kakoune session is killed and the `KakEnd` hook cannot run, the server will stay up until explicitely killed. The
`kak-tree-sitter-req-stop` can be used to shutdwon the server.

## Automatic highlighting hooks

A hook is automatically inserted on `WinCreate` (and transitively `WinDisplay`) to send a special request to the
server to try and highlight the current buffer. That request is a two-step process:

1. A request of type `try_highlight` is sent to the server with the content of `%opt{kts_lang}`.
2. If `%opt{kts_lang}` has an associated grammar and highlight queries (highlights, locals, injections, etc.), then
  the server sends back some more Kakoune commands to run. Otherwise, it just does nothing and the process ends there.
3. If the server sent back highlighting commands to run, they are executed and install a `buffer` local set of hooks to
  highlight the buffer in the current session. Those hooks will react to user input to automatically re-highlight the
  buffer.

## `ktsctl`, the companion controller of `kak-tree-sitter`

`ktsctl` is the controller CLI of `kak-tree-sitter`. It allows to run a variety of operations on the server and
especially its runtime configuration.

## Grammars and queries sources

By default, `kak-tree-sitter` ships with no tree-sitter grammar and query. This is a deliberate choice. Instead of
statically compiling with them (and then forcing users to use a specific version of a grammar and queries),
`kak-tree-sitter` dynamically loads grammars and runs queries at runtime. Both are located in
`$XDG_DATA_DIR/kak-tree-sitter/{grammars,queries}`.

Users have the choice to populate those directories by themselves, or use `ktsctl` to fetch, compile, link and install
grammars / queries.

An important aspect, though: grammars and especially queries are a best effort in `kak-tree-sitter`. What
that means is that, even though `ktsctl` will fetch and install things for you, the _default source_ (i.e. where to
fetch grammars and queries) is completely arbritrary. We default to
[https://gituhb.com/tree-sitter](https://github.com/tree-sitter) for famous grammars and
[https://github.com/helix-editor/helix](https://github.com/helix-editor/helix) for queries, but that is not an
obligation.

The end goal is to make `ktsctl` always target
[https://github.com/phaazon/kak-tree-sitter](https://github.com/phaazon/kak-tree-sitter) for both grammars and queries,
but that will require adding support for the languages manually / hand-crafting a bit.

## Colorschemes

When highlighting a buffer, `kak-tree-sitter` traverses tree-sitter capture groups and transforms them to make them
compatible with Kakoune. It does a couple of simple transformations:

1. For a given capture group — e.g. `variable.other.member`, it replaces dots with underscores — e.g.
  `variable_other_member`.
2. Then, it prepends `ts_` — e.g. `ts_variable_other_member`.
3. The final face `ts_variable_other_member` is used in highlighting regions of the buffer.

Face definitions is currently performed in the `rc` file. If you notice a missing face, please open a PR to add it to
the list. It is possible to disable inserting the faces in the `rc` file if your coloscheme defines those faces.

In terms of what we define a face to, this is an on-going topic, but the idea is:

1. Prevent using absolute values, like `rgb` strings, etc.
2. Prefer linking to well known faces, such as colors or anything described in `:doc faces`.
3. The face should be documented in the [doc/faces.md](../doc/faces.md) document. That is important so that colorschemes
  that decide to support `kak-tree-sitter` know exactly which faces they must override.

## Configuration

Most of the configuration should be done in `$XDG_CONFIG_DIR/kak-tree-sitter/config.toml`. The configuration options
are detailed in a specific document (TODO: which one?).
