# Usage

`kak-tree-sitter` is fairly straightforward to use. You can run it as a
standalone server by running the following command in a terminal:

```sh
kak-tree-sitter --server
```

Or you can run a similar command directly in your _kakrc_:

```kakrc
eval %sh{ kak-tree-sitter -dks --init $kak_session }
```

When you start from the command line, the server will wait for sessions to
connect and will persist even if no more Kakoune session exists. If you start
from within Kakoune, the last session exiting will cause the server to stop and
exit.

> For more information about the command line interface and the possible
> configuration options you have, please refer to the
> [Kakoune configuration section](Kakoune-config.md).

Wherever you start the server in your configuration, ensure that you do it
**before picking your colorscheme**, because some Kakoune commands are injected
and add face definitions. If your theme supports some of those faces, you don’t
want them to be overriden by the default values of `kak-tree-sitter`.

Some explanations:

- `kak-tree-sitter` is the binary server. Refer to
  [the installation section](install.md) if you haven’t installed it yet.
- `-d --daemon` starts the server in daemon mode. That means that if you start
  several sessions, the first session can exit before the new one. You should
  always use this flag when starting the server from Kakoune.
- `-k --kakoune` is used when starting from Kakoune to log to its `*debug*`
  buffer, and is required to start the server from Kakoune anyway.
- `-s --server` starts the server.
- `--init <SESSION>` tells the server to initiate a special request to get all
  the required configuration to communicate with the started server. Some
  important hooks are inserted, as well as face definitions and some internal
  options used to highlight your buffers). This flag writes directly to Kakoune
  without using any async operations, so this should only be used once in your
  config when a session starts.

## Feature picking

There are more flags available to use. Refer to the [Features](features.md)
document to know which flag to use to enable which features. If you do not pick
any feature, some default ones are enabled for you for a default experience.

## Set the logging level

Whether you are starting the server from Kakoune or from the CLI, you might
encounter issues. In such cases, it’s recommended to change the verbosity of
logs to be able to provide more information. You can use the `-v --verbose` flag
to do so. Accumulating will increase the verbosity:

- `-v` will print error messages.
- `-vv` will print error and warning messages.
- `-vvv` will print error, warning and info messages.
- `-vvvv` will print error, warning, info and debug messages.
- `-vvvvv` will print error, warning, info, debug and trace messages.

### Logging backends

When started from the CLI, logs will be written to _stdout_. When started from
within Kakoune, logs will be written to the `*debug*` buffer.

# Next

You may want to read the [Configuration](configuration.md) document.
