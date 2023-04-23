# Overall design

`kak-tree-sitter` implements a client/server architecture. The idea is that a single instance of `kak-tree-sitter`
should run as a server for a machine. Then, every Kakoune session can connect to the server and send requests via
the shell (`%sh{}`). Whenever the server has computed the response for the request, it provides it back to Kakoune
via the socket interface (`kak -p`) and the session identifier.

For a given Kakoune session, the `kak-tree-sitter` server knows all of the buffers that are required to be parsed
by `tree-sitter`. This allows efficient reusing of parsed data, which is an important aspect of `tree-sitter`: parse
once, then reuse the parsed data whenever we need to parse again to minimize each delta update.

When a client exists, it doesn’t ask the server to exit.

> Question: When does the server stop? I have no idea.

## Clientless run

When a Kakoune session starts, it is recommended to run `kak-tree-sitter` automatically with the session name.
If `kak-tree-sitter` is not already running, an instance will be started. Then, `kak-tree-sitter` will add an
entry for the Kakoune session and it will then be possible for clients of that session to start sending requests.
Note that if a Kakoune instance tries to connect to the (running) `kak-tree-sitter` server with a session name that
has already been registered, nothing happens, so there is no extra logic to do in the Kakoune configuration besides
just starting the server with the session name.

If users do not want to start `kak-tree-sitter` inside their Kakoune configuration, it is still possible to start it
like any other CLI application. They will have, however, to manually pass the Kakoune session (while this is done
automatically for you via `%val{session}` if you start it from your config.

## Client requests

Client can then connect to the server. They have to provide a couple of information when performing a request:

- The Kakoune session name, obviously.
- The Kakoune client name. That will be used to provide back feedback via info popups and other visual notifications.
