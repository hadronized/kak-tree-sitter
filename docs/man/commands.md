# Commands

Commands are separated into three categories:

- [Controlling kak-tree-sittter](#controlling-kak-tree-sitter)
- [Highlighting](#highlighting)
- [Text-objects](#text-objects)

## Controlling kak-tree-sitter

| Command                             | Description                                                                                                          |
| -------                             | -----------                                                                                                          |
| `kak-tree-sitter-req-enable`        | Send a request to enable integration with `kak-tree-sitter`.                                                         |
| `kak-tree-sitter-req-stop`          | Send a request to make `kak-tree-sitter` quit.                                                                       |
| `kak-tree-sitter-req-reload`        | Send a request to make `kak-tree-sitter` reload its configuration, grammars and queries.                             |
| `kak-tree-sitter-set-lang` (hidden) | Hidden function used to set `%opt{kts_lang}`. See [Kakoune interaction](Kakoune-interaction) for further information |

## Highlighting

| Command                            | Description                                     |
| -------                            | -----------                                     |
| `kak-tree-sitter-highlight-buffer` | Force a highlight request on the current buffer |

## Text-objects

| Command                                                 | Description                                                                                                                  |
| -------                                                 | -----------                                                                                                                  |
| `kak-tree-sitter-req-text-objects <text-object> <mode>` | Alter every selections by matching `<text-object>` according to `<mode>`. See [the text-objects section](./Text-objects.md). |
| `kak-tree-sitter-req-object-text-objects <text-object>` | Alter every selections by matching `<text-object>` in _object_ mode. See [the text-objects section](./Text-objects.md).      |
