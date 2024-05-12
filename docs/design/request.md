# Tree-sitter requests

The server supports many requests that clients can send.

## Session setup

When a session starts, it should call something like `kak-tree-sitter -dks …`.
The `-k` (`--kakoune`) flag injects some options, hooks and commands to prepare
the session to work with KTS. It allows the server to send replies with KTS
commands.

## Initial request and response

Because the server is started with `--session $kts_session`, an initial request
is emitted once the server is started: `register_session`. Upon receiving that
request, the KTS server knows that the session exists and that it needs to be
configured. Configuring a session implies:

- Creating a command FIFO so that the session can communicate with the server.
  Every session has a dedicated command FIFO to write to, so that concurrent
  sessions can send isolated commands. The path of that FIFO is stored for the
  reply.
- Track the session. This is needed so that when the session quits, we can
  perform the required checks to know whether we should shutdown the server.

And the server replies to the session to:

- Set the `kts_cmd_fifo_path` global option in the session.

## Session exit

When a session quits, it emits the `session_exit` request so that the KTS server
can cleanup the resource of the session and check whether it should quit.

## Configuration reload

The `reload` request can be used at anytime to ask the KTS server to reload
its configuration. Upon reloading, languages, grammars and queries are also
replaced with the fresh configuration.

If reloading fails, no change is applied to the running KTS server.

## Explicit shutdown

It’s possible to explicitly ask the server to shutdown with the `shutdown`
request. Doing so will cause the server to send special responses, `deinit`, to
all connected sessions.

## WIP Buffer update

> Note: this section is a work-in-progress.

Buffer update is a central concern in KTS. Doing it right is hard, because
Kakoune doesn’t provide a way to easily share buffers. Once a buffer has
changed, we can make KTS know about it by:

- Writing the content to a given path. This is not ideal, because it induces a
  lot of I/O, especially on systems where _tmpfs_ is not supported (e.g. macOS).
- Writing the content to a FIFO. FIFO are special files that act as
  _rendez-vous_ gates between two peers (the producer and consumer, the producer
  being Kakoune and the consumer being the KTS server here). In terms of
  performance, this solution is much better than writing to regular files.
- Using shared memory. This solution would require Kakoune to expose a shared
  read-only memory segment of its internal representation of a buffer, which
  would be the best solution, but probably hard to work with even if it was
  available.

### Using FIFO

FIFOs are currently the best way for use to minimize the overhead of having to
stream buffers between two processes (Kakoune and KTS). Editors with a non-UNIX
design typically push features in the editor directly, which has the benefit of
putting all the features near the same memory region, but with Kakoune we cannot
do that, since the editor itself doesn’t get new features in; we have to build
them _externally_.

Once a buffer needs to stream its updated content to the KTS server, it first
sends a request to KTS with the FIFO to read from, along with some other
metadata. Then, it uses the `write` command to write the content of the buffer.
The KTS server can then reads the content of the FIFO, and once that is done,
the `write` unlocks in Kakoune.

> It is possible that buffer updates trigger more asynchronous responses from
> the KTS server; for instance if it was started with `--with-highlighting`.
