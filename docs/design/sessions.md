# Session support

## One process for all sessions

KTS works on a per-session principle. A single server replies to requests from
all the Kakoune sessions; effectively, a single server process handles all the
Kakoune sessions on your machine.

## Buffer isolation

If you open the same file in two different sessions, they will be handled as
two different buffers by KTS, since they truly are two different buffers in
Kakoune too. KTS does not make any assumption about the buffers; they can be
associated with a filesystem file, or not.

## Session and KTS lifecycle

When started from within Kakoune (i.e. `-k --kakoune`), sessions play an
important role in the lifecycle of KTS. Indeed, a first initial session is
needed to start KTS from Kakoune (passed with `--session $kak_session`). If a
new session starts, it will not start a KTS server but instead will join the
already existing one.

Once a session exits, the KTS server takes it into account and keeps track of
active session. When the number of session reaches zero, the server exits.

> This is true even if the last session is not the first initial one.

## Recovering session data

If KTS abruptly quits or is explicitely shutdown, sessions might froze (because
of pending FIFO writes; the server not being able to reply, it blocks the FIFO
operations and Kakoune hangs). In such a case, restarting the KTS server will
see that some sessions were previously alive, and will recover by using the data
for those sessions.

That effectively allows for two main features:

- Unfreezing in case of a KTS bug.
- Upgrading KTS while Kakoune still runs.

This is especially important if you are running other utilities, like a LSP
server on a big project that takes time to index.
