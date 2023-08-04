# This file should be sourced only once by session. It is not recommended to source it yourself; instead, the KTS server
# will inject it via the kak’s UNIX socket when ready to accept commands for the session.

# Options used by KTS at runtime.

# FIFO command path; this is used by Kakoune to write commands to be executed by KTS for the current session.
declare-option str kts_fifo_cmd_path

# Highlight ranges used when highlighting buffers.
declare-option range-specs kts_highlighter_ranges

# Faces definition; defaults to catppuccin_macchiato (TODO: will be deleted when theme support lands)
declare-option str kts_rosewater 'rgb:f4dbd6'
declare-option str kts_flamingo 'rgb:f0c6c6'
declare-option str kts_pink 'rgb:f5bde6'
declare-option str kts_mauve 'rgb:c6a0f6'
declare-option str kts_red 'rgb:ed8796'
declare-option str kts_maroon 'rgb:ee99a0'
declare-option str kts_peach 'rgb:f5a97f'
declare-option str kts_yellow 'rgb:eed49f'
declare-option str kts_green 'rgb:a6da95'
declare-option str kts_teal 'rgb:8bd5ca'
declare-option str kts_sky 'rgb:91d7e3'
declare-option str kts_sapphire 'rgb:7dc4e4'
declare-option str kts_blue 'rgb:8aadf4'
declare-option str kts_lavender 'rgb:b7bdf8'
declare-option str kts_gray1 'rgb:6e738d'
declare-option str kts_text 'rgb:cad3f5'
declare-option str kts_subtext1 'rgb:b8c0e0'
declare-option str kts_subtext0 'rgb:a5adcb'
declare-option str kts_overlay2 'rgb:939ab7'
declare-option str kts_overlay1 'rgb:8087a2'
declare-option str kts_overlay0 'rgb:6e738d'
declare-option str kts_surface2 'rgb:5b6078'
declare-option str kts_surface1 'rgb:494d64'
declare-option str kts_surface0 'rgb:363a4f'

# Mark the session as non-active.
#
# This is typically sent when a session is about to die; see KakEnd for further details.
define-command -hidden kak-tree-sitter-end-session -docstring 'Mark the session as ended' %sh{
  kak-tree-sitter -r { "type": "session_exit", "name": "%val{session}" }
}

# Stop the kak-tree-sitter daemon.
#
# To restart the daemon, the daemon must explicitly be recreated with %sh{kak-tree-sitter -d -s $kak_session}.
define-command kak-tree-sitter-stop -docstring 'Ask the daemon to shutdown' %{
  evaluate-commands -no-hooks -buffer * %{
    remove-hooks buffer kak-tree-sitter
  }

  remove-hooks global kak-tree-sitter
  %sh{
    kak-tree-sitter -r { "type": "shutdown" }
  }
}

# Enabling highlighting for the current buffer.
# 
# This command does a couple of things, among removing the « default » highlighting (Kakoune based) of the buffer and
# installing some hooks to automatically highlight the buffer.
define-command -hidden kak-tree-sitter-highlight-enable -docstring 'Enable tree-sitter highlighting for this buffer' %{
  # remove regular highlighting, if any; we wrap this with try %{} because the highlighter might not even exist or is
  # named differently; in such a case we should probably have a mapping or something
  try %{
    remove-highlighter "window/%opt{filetype}"
  }

  # Add the tree-sitter highlighter
  set-option buffer kak_tree_sitter_highlighter_ranges
  add-highlighter -override buffer/kak-tree-sitter-highlighter ranges kts_highlighter_ranges

  # initial highlighting of the buffer
  kak-tree-sitter-highlight-buffer

  hook -group kak-tree-sitter buffer InsertIdle .* kak-tree-sitter-highlight-buffer
  hook -group kak-tree-sitter buffer NormalIdle .* kak-tree-sitter-highlight-buffer
}

# Send a single request to highlight the current buffer.
define-command kak-tree-sitter-highlight-buffer -docstring 'Highlight the current buffer' %{
  nop %sh{
    echo "evaluate-commands -no-hooks -verbatim write $kak_response_fifo" > $kak_command_fifo
    kak-tree-sitter -s $kak_session -c $kak_client -r "{\"type\":\"highlight\",\"buffer\":\"$kak_bufname\",\"lang\":\"$kak_opt_filetype\",\"timestamp\":$kak_timestamp,\"payload\":\"$kak_response_fifo\"}"
  }
}

# Enable automatic tree-sitter highlights.
hook -group kak-tree-sitter global WinCreate .* %{
  hook -group kak-tree-sitter buffer -once WinDisplay .* %{
    # Check whether this filetype is supported
    echo -to-file %opt{kts_fifo_cmd_path} -- "{""session"": {""session_name"": ""%val{session}"", ""client_name"": ""%val{client}""}, ""payload"": {""type"": ""try_enable_highlight"", ""lang"": ""%opt{filetype}""}};"
  }
}

# Make kak-tree-sitter know the session has ended whenever we end it.
hook -group kak-tree-sitter global KakEnd .* kak-tree-sitter-end-session

#set-face global ts_unknown                     red+ub
set-face global ts_attribute                    "%opt{kts_teal}"
set-face global ts_comment                      "%opt{kts_overlay1}+i"
set-face global ts_comment_block                "ts_comment"
set-face global ts_comment_line                 "ts_comment"
set-face global ts_conceal                      "%opt{kts_mauve}+i"
set-face global ts_constant                     "%opt{kts_peach}"
set-face global ts_constant_builtin_boolean     "%opt{kts_sky}"
set-face global ts_constant_character           "%opt{kts_yellow}"
set-face global ts_constant_character_escape    "ts_constant_character"
set-face global ts_constant_macro               "%opt{kts_mauve}"
set-face global ts_constant_numeric             "%opt{kts_peach}"
set-face global ts_constant_numeric_float       "ts_constant_numeric"
set-face global ts_constant_numeric_integer     "ts_constant_numeric"
set-face global ts_constructor                  "%opt{kts_sapphire}"
set-face global ts_diff_plus                    "%opt{kts_green}"
set-face global ts_diff_minus                   "%opt{kts_red}"
set-face global ts_diff_delta                   "%opt{kts_blue}"
set-face global ts_diff_delta_moved             "%opt{kts_mauve}"
set-face global ts_error                        "%opt{kts_red}+b"
set-face global ts_function                     "%opt{kts_blue}"
set-face global ts_function_builtin             "%opt{kts_blue}+i"
set-face global ts_function_macro               "%opt{kts_mauve}"
set-face global ts_function_method              "ts_function"
set-face global ts_function_special             "ts_function"
set-face global ts_hint                         "%opt{kts_blue}+b"
set-face global ts_info                         "%opt{kts_green}+b"
set-face global ts_keyword                      "%opt{kts_mauve}"
set-face global ts_keyword_control              "ts_keyword"
set-face global ts_keyword_conditional          "%opt{kts_mauve}+i"
set-face global ts_keyword_control_conditional  "%opt{kts_mauve}+i"
set-face global ts_keyword_control_directive    "%opt{kts_mauve}+i"
set-face global ts_keyword_control_import       "%opt{kts_mauve}+i"
set-face global ts_keyword_control_repeat       "ts_keyword"
set-face global ts_keyword_control_return       "ts_keyword"
set-face global ts_keyword_control_except       "ts_keyword"
set-face global ts_keyword_control_exception    "ts_keyword"
set-face global ts_keyword_directive            "%opt{kts_mauve}+i"
set-face global ts_keyword_function             "ts_keyword"
set-face global ts_keyword_operator             "ts_keyword"
set-face global ts_keyword_special              "ts_keyword" 
set-face global ts_keyword_storage              "ts_keyword" 
set-face global ts_keyword_storage_modifier     "ts_keyword" 
set-face global ts_keyword_storage_modifier_mut "ts_keyword" 
set-face global ts_keyword_storage_modifier_ref "ts_keyword" 
set-face global ts_keyword_storage_type         "ts_keyword" 
set-face global ts_label                        "%opt{kts_sapphire}+i"
set-face global ts_markup_bold                  "%opt{kts_peach}+b"
set-face global ts_markup_heading               "%opt{kts_red}"
set-face global ts_markup_heading_1             "%opt{kts_red}"
set-face global ts_markup_heading_2             "%opt{kts_mauve}"
set-face global ts_markup_heading_3             "%opt{kts_green}"
set-face global ts_markup_heading_4             "%opt{kts_yellow}"
set-face global ts_markup_heading_5             "%opt{kts_pink}"
set-face global ts_markup_heading_6             "%opt{kts_teal}"
set-face global ts_markup_heading_marker        "%opt{kts_peach}+b"
set-face global ts_markup_italic                "%opt{kts_pink}+i"
set-face global ts_markup_list_checked          "%opt{kts_green}"
set-face global ts_markup_list_numbered         "%opt{kts_blue}+i"
set-face global ts_markup_list_unchecked        "%opt{kts_teal}"
set-face global ts_markup_list_unnumbered       "%opt{kts_mauve}"
set-face global ts_markup_link_label            "%opt{kts_blue}"
set-face global ts_markup_link_url              "%opt{kts_teal}+u"
set-face global ts_markup_link_uri              "%opt{kts_teal}+u"
set-face global ts_markup_link_text             "%opt{kts_blue}"
set-face global ts_markup_quote                 "%opt{kts_gray1}"
set-face global ts_markup_raw                   "%opt{kts_sky}"
set-face global ts_markup_raw_block             "%opt{kts_sky}"
set-face global ts_markup_raw_inline            "%opt{kts_green}"
set-face global ts_markup_strikethrough         "%opt{kts_gray1}+s"
set-face global ts_namespace                    "%opt{kts_blue}+i"
set-face global ts_operator                     "%opt{kts_sky}"
set-face global ts_property                     "%opt{kts_sky}"
set-face global ts_punctuation                  "%opt{kts_overlay2}"
set-face global ts_punctuation_bracket          "ts_punctuation"
set-face global ts_punctuation_delimiter        "ts_punctuation"
set-face global ts_punctuation_special          "%opt{kts_sky}"
set-face global ts_special                      "%opt{kts_blue}"
set-face global ts_spell                        "%opt{kts_mauve}"
set-face global ts_string                       "%opt{kts_green}"
set-face global ts_string_regex                 "%opt{kts_peach}"
set-face global ts_string_regexp                "%opt{kts_peach}"
set-face global ts_string_escape                "%opt{kts_mauve}"
set-face global ts_string_special               "%opt{kts_blue}"
set-face global ts_string_special_path          "%opt{kts_green}"
set-face global ts_string_special_symbol        "%opt{kts_mauve}"
set-face global ts_string_symbol                "%opt{kts_red}"
set-face global ts_tag                          "%opt{kts_teal}"
set-face global ts_tag_error                    "%opt{kts_red}"
set-face global ts_text                         "%opt{kts_text}"
set-face global ts_text_title                   "%opt{kts_mauve}"
set-face global ts_type                         "%opt{kts_yellow}"
set-face global ts_type_builtin                 "ts_type"
set-face global ts_type_enum_variant            "%opt{kts_flamingo}"
set-face global ts_variable                     "%opt{kts_text}"
set-face global ts_variable_builtin             "%opt{kts_red}"
set-face global ts_variable_other_member        "%opt{kts_teal}"
set-face global ts_variable_parameter           "%opt{kts_maroon}+i"
set-face global ts_warning                      "%opt{kts_peach}+b"
