# Text-objects

Text-objects are automatically supported, but a default, feature-full setup is
available by passing the `--with-text-objects` flag to `kak-tree-sitter` when
starting. That will create several user-modes, among the top-level `tree-sitter`
mode (the others are accessible through it), and it will put some mappings in
_object_ mode:

- `a` for arguments.
- `f` for functions.
- `t` for types.
- `T` for tests.

If you want to customize and create your own mappings, it’s advised to read
[Commands](./commands.md) to know which commands to call.

## Capture groups

What people call “text-objects” are, in **tree-sitter**, referred to as
_capture groups_. Even though everyone can use whatever they want, there are, at
some extent, some standards. For text-objects, capture groups are expressed as
`<type>.<level>`, where `<level>` is either `inside` or `around`, and `<type>`
is the type of object you want to match against (`class`, `function`,
`parameter`, etc.).

Hence, a capture group to pattern _inside_ (i.e. bodies) of functions is
`function.inside`. Matching on whole functions, including the signature, is
`function.around`.

## Operational modes

`kak-tree-sitter` has the concept of _operational modes_. When matching against
a capture-group (e.g. `function.inside`), we still don’t know exactly what to
do; do we want to select the next one? the previous one? extend to the next
function? select inside the current function? etc.

All those options are encoded as operational modes. There are many:

- `search_next`: search for the next text-object after the cursor. Similar to
  `/`.
- `search_prev`: search for the previous text-object before the cursor. Similar
  to `<a-/>`.
- `search_extend_next`: search and extend to the next text-object after the
  cursor. Similar to `?`.
- `search_extend_prev`: search and extend to the previous text-object before the
  cursor. Similar to `<a-?>`.
- `find_next`: select onto the next text-object after the cursor. Similar to
  `f`.
- `find_prev`: select onto the previous text-object before the cursor. Similar
  to `<a-f>`.
- `extend_next`: extend onto the next text-object after the cursor. Similar to
  `F`.
- `extend_prev`: extend onto the previous text-object before the cursor. Similar
  to `<a-F>`.

## Navigation

Additionally to text-objects, you also get _tree-sitter-based navigation_. The
way it works is by first selecting a node and then moving into a semantic
direction.

> As with regular text-objects, navigation is applied to **all selections**, not
> only the primary one.

The following directions are currently implemented:

- `parent`: parent of the current node.
- `first_child`: first child of the current node.
- `last_child`: last child of the current node.
- `first_sibling`: first sibling of the current node.
- `last_sibling`: last sibling of the current node.
- `prev_sibling`: previous sibling of the current node. Can include cousins.
- `next_sibling`: next sibling of the current node. Can include cousins.

The `tree-sitter` user-mode contains some mappings to start experimenting with
navigation.

If you would rather write your own commands, you need to use the
`tree-sitter-nav <dir>` command. `<dir>` is one of the items described above as
a string — except for directions that can include cousins. In such a case, you
need to turn the direction into an object with a single attribute, `cousin`,
which is of type `bool`. Setting `cousin` to `true` will allow prev/next sibling
navigation to “jump” over parents and try to go to cousins instead of direct
siblings.

### Navigation examples

- `tree-sitter-nav '"parent"'`: go to the parent of the current selections.
- `tree-sitter-nav "'{ ""prev_sibling"": { ""cousin"": false } }'"`: go to the
  previous sibling.

### Bonus: sticky navigation

A handy `tree-sitter-nav-sticky` user-mode is available to navigate without
having to re-enter the user-mode. If you run the server with text-objects
support, the mode is available via `T`.
