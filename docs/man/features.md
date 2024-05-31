# Features

This document presents features planned by the project and their status.

| Feature           | Description                                                                                | Status          | Available since | Default config | CLI flag to enable                                      |
| ---               | ---                                                                                        | ---             | ---             | ---            | ---                                                     |
| [Highlighting]    | Asynchronous automatic highlighting of session buffers.                                    | **Implemented** | `v0.2`          | `true`         | `--with-highlighting`                                   |
| [Text-objects]    | Modify Kakoune selections with text-objects (`function.inside`, `parameter.around`, etc.). | **Implemented** | `v0.6`          | `true`         | Default, and `--with-text-objects` for additional setup |
| Indents           | Automatically indent your buffer.                                                          | Not started     |                 |                | `--with-indenting`                                      |
| Indent guidelines | Display a guideline showing the level of indentation left to lines.                        | Not started     |                 |                | `--with-indent-guidelines`                              |

[Highlighting]: highlighting.md
[Text-objects]: text-objects.md
