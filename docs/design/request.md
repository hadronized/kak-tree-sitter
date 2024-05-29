# Tree-sitter requests

The server supports many requests that clients can send.

## Session setup

When a session starts, it should call something like `kak-tree-sitter -dks …`.
The `-k` (`--kakoune`) flag indicates that the server is started from within
Kakoune. This flag is important for things like initial logging and injecting
commands into Kakoune, which is done via the `--init <SESSION>` flag. When this
flag is passed, the server writes commands to _stdout_, which is then
interpreted by Kakoune. **Those commands are mandatory for the integration to
work correctly.**

> _Why having two flags; i.e. `--kakoune` and `--init`?_
>
> `--kakoune` is used to state that we start from Kakoune. It’s used when
> initiating a session — typicall from inside your `kakrc` — but it is also used
> whenever a command is issued to the server. That allows the command to log
> directly into Kakoune, and have more feedback.
>
> `--init` is just used once when initiating a session.

## Initial request and response

Because the server is started with `--init $kts_session`, an initial request
is emitted once the server is started: `session_begin`. Upon receiving that
request, the KTS server knows that the session exists and that it needs to be
configured. Configuring a session implies:

- Tracking the session. This is needed so that when the session quits, we can
  perform the required checks to know whether we should shutdown the server.
- Enable support for enabled languages, which sets some hooks when a buffer is
  open for such languages.

## Session exit

When a session quits, it emits the `session_end` request so that the KTS server
can cleanup the resources (mostly FIFOs) of the session and check whether it
should quit.

## Configuration reload

The `reload` request can be used at anytime to ask the KTS server to reload
its configuration. Upon reloading, languages, grammars and queries are also
replaced with the fresh configuration.

If reloading fails, no change is applied to the running KTS server.

## Explicit shutdown

It’s possible to explicitly ask the server to shutdown with the `shutdown`
request.

## Buffer metadata / setup

Once a session is fully initiated with KTS, it has some hooks for configured
languages. Opening a buffer for such language will trigger such hooks, which
will eventually send the `buffer_metadata` request.

The `buffer_metadata` request passes some information about the buffer required
for tree-sitter parsing. Currently, only the language the buffer is written in
is passed, read from the `tree_sitter_lang` `window` option.

It is possible to make a `buffer_metadata` request several time for the same
buffer, as the request should be idempotent. However, if the language has
changed, the buffer will be reset and setup again.

KTS will send responses to `buffer_metadata` that will:

- Set the FIFO path to stream the buffer content to.
- Set the _buffer sentinel_ for this buffer (see the buffer update section).

## Buffer update

Buffer update is a central concern in KTS. Doing it right is hard, because
Kakoune doesn’t provide a way to easily share buffers. Once a buffer has
changed, we can make KTS know about it by:

- Writing the content to a given path. This is not ideal, because it induces a
  lot of I/O, especially on systems where _tmpfs_ is not supported (e.g. macOS).
- Writing the content to a FIFO. FIFO are special files that act as
  _rendez-vous_ gates between two ends (the producer and consumer, the producer
  being Kakoune and the consumer being the KTS server here). In terms of
  performance, this solution is much better than writing to regular files.
- Using shared memory. This solution would require Kakoune to expose a shared
  read-only memory segment of its internal representation of a buffer, which
  would be the best solution, but probably hard to work with even if it was
  available.

FIFOs are currently the best way for us to minimize the overhead of having to
stream buffers between two processes (Kakoune and KTS). Editors with a non-UNIX
design typically push features into the editor directly, which has the benefit
of putting all the features near the same memory region, but with Kakoune we
cannot do that, since the editor itself doesn’t get new features in; we have to
build them _externally_.

Once a buffer needs to stream its updated content to the KTS server, it does two
operations:

- `write` to the FIFO. The `write` Kakoune command writes the content of the
  buffer to the specified file, so here, we write the content into the FIFO.
- Once the content is written to the FIFO (which is open in non-blocking on the
  KTS side), the _buffer sentinel_ is written to the FIFO.

The _buffer sentinel_ is a special string (UUID v4) set when the buffer is
setup with tree-sitter, and marks the end of the buffer. This is a protocol
detail that is required to know when the buffer is fully streamed to the FIFO.

> It is possible that buffer updates trigger more asynchronous responses from
> the KTS server; for instance if it was started with `--with-highlighting`.

## Buffer close

`buffer_close` can be passed when a buffer is closed, which cleans resources
from KTS for this buffer.

## TODO Text objects

## TODO Nav
