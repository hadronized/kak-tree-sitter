# Highlighting

In order to enable highlighting, you must pass the `--with-highlighting` flag when starting `kak-tree-sitter`. Then,
**after `kak-tree-sitter --kakoune` is called**, you can use the `colorscheme` command with a tree-sitter compatible
colorschemes. [See this section](#tree-sitter-enabled-colorschemes) for further information.

## Automatic highlighting of buffers

Once the server is run, if your buffer can be highlighted, Kakoune will send (via hooks) requests to `kak-tree-sitter`
to highlight your buffer. The current mechanism to detect whether a buffer can be highlighted uses `%opt{kts_lang}`.
This option is automatically set by a hook for you, but you can override the default behavior (see below). Furthermore,
highlighting is currently performed on `NormalIdle` and `InsertIdle`.

## Override the `%opt{kts_lang}` setting

The default setting forwards `%opt{filetype}` to `%opt{kts_lang}`. Depending on your setup, that might not be enough.
The forwarding is done via a hidden command, called `kak-tree-sitter-set-lang`. To implement the default behavior, you
could write something like this:

```kakrc
define-command -override kak-tree-sitter-set-lang %{
  set-option buffer kts_lang %opt{filetype}
}
```

The only requirement for implementing this function is to eventually `set-option buffer kts_lang` to something, and that
the function shouldn’t be long to return. It should be run once for each buffer after being created but you should try
to keep that function as fast as possible.

# Tree-sitter-enabled colorschemes

Colorscheme support is provided by the various capture-groups taken from grammars and queries, which get
translated to Kakoune `set-face` commands. You have two options:

- Roam around and look for tree-sitter-enabled colorschemes. A starting point is [kakoune-tree-sitter-themes].
- Write your own colorscheme. You may want to read on this page.

## How to make your colorscheme tree-sitter aware

The way tree-sitter colorschemes work is by calling `set-face` for the particular capture-groups you want to set the
highlight for. Faces are organized in a _cascaded_ way, which means that by default, faces might have a parental
relationship with others. For instance, the `ts_keyword_storage_modifier` face is defined as `ts_keyword_storage`, which
is defined as `ts_keyword`. When a keyword doesn’t have any parent, by default, it’s set to `default`.

> This behavior is _wanted_ and will make things look odd if you are not using a proper tree-sitter colorschemes.

It is recommended to set, at least, the top-level faces. If you want more granularity — for instance, a
different color for `ts_keyword_storage` and `ts_keyword_storage_modifier` — you should specialize faces as well.

You will need the list of faces to set, which can be find below in the [faces list section](#faces)

## Faces

The following faces can and should be set in tree-sitter-enabled colorschemes. Cascaded faces inherit from their parent
by default, so if you want all underlying faces to have the same highlight as their parent, you do not need to set them
at all.

- `ts_attribute`
- `ts_comment`
  - `ts_comment_block`
  - `ts_comment_line`
- `ts_conceal`
- `ts_constant`
  - `ts_constant_builtin_boolean`
  - `ts_constant_character`
    - `ts_constant_character_escape`
  - `ts_constant_macro`
  - `ts_constant_numeric`
    - `ts_constant_numeric_float`
    - `ts_constant_numeric_integer`
- `ts_constructor`
- `ts_diff_plus`
- `ts_diff_minus`
- `ts_diff_delta`
  - `ts_diff_delta_moved`
- `ts_error`
- `ts_function`
  - `ts_function_builtin`
  - `ts_function_macro`
  - `ts_function_method`
  - `ts_function_special`
- `ts_hint`
- `ts_info`
- `ts_keyword`
  - `ts_keyword_control`
  - `ts_keyword_conditional`
  - `ts_keyword_control_conditional`
  - `ts_keyword_control_directive`
  - `ts_keyword_control_import`
  - `ts_keyword_control_repeat`
  - `ts_keyword_control_return`
  - `ts_keyword_control_except`
  - `ts_keyword_control_exception`
  - `ts_keyword_directive`
  - `ts_keyword_function`
  - `ts_keyword_operator`
  - `ts_keyword_special`
  - `ts_keyword_storage`
    - `ts_keyword_storage_modifier`
      - `ts_keyword_storage_modifier_mut`
      - `ts_keyword_storage_modifier_ref`
    - `ts_keyword_storage_type`
- `ts_label`
- `ts_markup_bold`
- `ts_markup_heading`
  - `ts_markup_heading_1`
  - `ts_markup_heading_2`
  - `ts_markup_heading_3`
  - `ts_markup_heading_4`
  - `ts_markup_heading_5`
  - `ts_markup_heading_6`
  - `ts_markup_heading_marker`
- `ts_markup_italic`
- `ts_markup_list_checked`
- `ts_markup_list_numbered`
- `ts_markup_list_unchecked`
- `ts_markup_list_unnumbered`
- `ts_markup_link_label`
- `ts_markup_link_url`
- `ts_markup_link_uri`
- `ts_markup_link_text`
- `ts_markup_quote`
- `ts_markup_raw`
  - `ts_markup_raw_block`
  - `ts_markup_raw_inline`
- `ts_markup_strikethrough`
- `ts_namespace`
- `ts_operator`
- `ts_property`
- `ts_punctuation`
  - `ts_punctuation_bracket`
  - `ts_punctuation_delimiter`
  - `ts_punctuation_special`
- `ts_special`
- `ts_spell`
- `ts_string`
  - `ts_string_regex`
  - `ts_string_regexp`
  - `ts_string_escape`
  - `ts_string_special`
    - `ts_string_special_path`
    - `ts_string_special_symbol`
  - `ts_string_symbol`
- `ts_tag`
- `ts_tag_error`
- `ts_text`
  - `ts_text_title`
- `ts_type`
  - `ts_type_builtin`
  - `ts_type_enum_variant`
- `ts_variable`
  - `ts_variable_builtin`
  - `ts_variable_other_member`
  - `ts_variable_parameter`
- `ts_warning`

[kakoune-tree-sitter-themes]: https://git.sr.ht/~hadronized/kakoune-tree-sitter-themes
