# This file should be sourced only once by session. It is not recommended to source it yourself; instead, the KTS server
# will inject it via the kak’s UNIX socket when ready to accept commands for the session.

# Options used by KTS at runtime.

# FIFO command path; this is used by Kakoune to write commands to be executed by KTS for the current session.
declare-option str kts_cmd_fifo_path /dev/null

# FIFO buffer path; this is used by Kakoune to write the content of buffers to be highlighted / analyzed by KTS for the
# current session.
declare-option str kts_buf_fifo_path /dev/null

# Highlight ranges used when highlighting buffers.
declare-option range-specs kts_highlighter_ranges

# Tree-sitter language to use to parse buffers’ content with tree-sitter.
declare-option str kts_lang

# Mark the session as non-active.
#
# This is typically sent when a session is about to die; see KakEnd for further details.
define-command -hidden kak-tree-sitter-req-end-session -docstring 'Mark the session as ended' %{
  nop %sh{
    kak-tree-sitter -r "{ \"type\": \"session_exit\", \"name\": \"$kak_session\" }"
  }
}

# Deinit KTS for the current session.
define-command -hidden kak-tree-sitter-deinit %{
  evaluate-commands -no-hooks -buffer * %{
    remove-hooks buffer kak-tree-sitter
  }

  remove-hooks global kak-tree-sitter
  set-option global kts_buf_fifo_path '/dev/null'
  set-option global kts_cmd_fifo_path '/dev/null'

}

# Stop the kak-tree-sitter daemon.
#
# To restart the daemon, the daemon must explicitly be restarted with a %sh{} block.
define-command kak-tree-sitter-req-stop -docstring 'Ask the daemon to shutdown' %{
  kak-tree-sitter-deinit

  nop %sh{
    kak-tree-sitter -r '{ "type": "shutdown" }'
  }
}

# Reload KTS.
define-command kak-tree-sitter-req-reload -docstring 'Reload kak-tree-sitter config, grammars and queries' %{
  nop %sh{
    kak-tree-sitter -r '{ "type": "reload" }'
  }
}

# Send a single request to highlight the current buffer.
#
# This will first send the command to highlight the buffer to KTS and then will write the content of the buffer through
# the same FIFO.
define-command kak-tree-sitter-req-highlight-buffer -docstring 'Highlight the current buffer' %{
  evaluate-commands -no-hooks %{
    echo -to-file %opt{kts_cmd_fifo_path} -- "{ ""type"": ""highlight"", ""client"": ""%val{client}"", ""buffer"": ""%val{bufname}"", ""lang"": ""%opt{kts_lang}"", ""timestamp"": %val{timestamp} }"
    write %opt{kts_buf_fifo_path}
  }
}

# Send a single request to modify selections with text-objects.
#
# The pattern must be full; e.g. 'function.inside'.
define-command kak-tree-sitter-req-text-objects -params 2 %{
  evaluate-commands -no-hooks %{
    echo -to-file %opt{kts_cmd_fifo_path} -- "{ ""type"": ""text_objects"", ""client"": ""%val{client}"", ""buffer"": ""%val{bufname}"", ""lang"": ""%opt{kts_lang}"", ""pattern"": ""%arg{1}"", ""selections"": ""%val{selections_desc}"", ""mode"": ""%arg{2}"" }"
    write %opt{kts_buf_fifo_path}
  }
}

# Send a single request to modify selections with text-objects in object-mode.
#
# The pattern must be expressed without the level — e.g. 'function' — as the level is deduced from %val{object_flags}.
define-command kak-tree-sitter-req-object-text-objects -params 1 %{
  evaluate-commands -no-hooks %{
    echo -to-file %opt{kts_cmd_fifo_path} -- "{ ""type"": ""text_objects"", ""client"": ""%val{client}"", ""buffer"": ""%val{bufname}"", ""lang"": ""%opt{kts_lang}"", ""pattern"": ""%arg{1}"", ""selections"": ""%val{selections_desc}"", ""mode"": { ""object"": { ""mode"": ""%val{select_mode}"", ""flags"": ""%val{object_flags}"" } } }"
    write %opt{kts_buf_fifo_path}
  }
}

# Send a single request to modify selections with tree-sitter navigation.
define-command kak-tree-sitter-req-nav -params 1 %{
  evaluate-commands -no-hooks %{
    echo -to-file %opt{kts_cmd_fifo_path} -- "{ ""type"": ""nav"", ""client"": ""%val{client}"", ""buffer"": ""%val{bufname}"", ""lang"": ""%opt{kts_lang}"", ""selections"": ""%val{selections_desc}"", ""dir"": %arg{1} }"
    write %opt{kts_buf_fifo_path}
  }
}

# Enable highlighting for the current buffer.
#
# This command does a couple of things, among removing the « default » highlighting (Kakoune based) of the buffer and
# installing some hooks to automatically highlight the buffer.
define-command -hidden kak-tree-sitter-highlight-enable -docstring 'Enable tree-sitter highlighting for this buffer' %{
  # Add the tree-sitter highlighter
  add-highlighter -override buffer/kak-tree-sitter-highlighter ranges kts_highlighter_ranges

  # Initial highlighting of the buffer
  kak-tree-sitter-req-highlight-buffer

  # Main hooks when enabling highlighting
  hook -group kak-tree-sitter buffer InsertIdle .* kak-tree-sitter-req-highlight-buffer
  hook -group kak-tree-sitter buffer NormalIdle .* kak-tree-sitter-req-highlight-buffer
}

# Set %opt{kts_lang} for the current buffer.
#
# The default implementation forwards %opt{filetype}.
define-command -hidden kak-tree-sitter-set-lang %{
  set-option buffer kts_lang %opt{filetype}
}

# Send a request to KTS to enable kak-tree-sitter.
define-command kak-tree-sitter-req-enable -docstring 'Send request to enable tree-sitter support' %{
  kak-tree-sitter-set-lang
  echo -to-file %opt{kts_cmd_fifo_path} -- "{ ""type"": ""try_enable_highlight"", ""lang"": ""%opt{kts_lang}"", ""client"": ""%val{client}"" }"
}

# Initiate request.
#
# This is used to ask the server to tell us where to write commands and other various data. It’s also where the server
# returns additional code, depending on enabled features.
define-command -hidden kak-tree-sitter-req-init %{
  nop %sh{
    kak-tree-sitter -r "{ \"type\": \"register_session\", \"name\": \"$kak_session\", \"client\": \"$kak_client\" }"
  }
}

# Command inserting highlighting hook.
define-command -hidden kak-tree-sitter-enable-highlighting %{
  hook -group kak-tree-sitter global WinCreate .* %{
    hook -group kak-tree-sitter buffer -once WinDisplay .* kak-tree-sitter-req-enable
  }
}

# Wait to have a client and then ask the server to initiate.
hook -group kak-tree-sitter global -once ClientCreate .* %{
  kak-tree-sitter-req-init

  # Make kak-tree-sitter know the session has ended whenever we end it.
  hook -group kak-tree-sitter global KakEnd .* kak-tree-sitter-req-end-session
}

#set-face global ts_unknown                     red+ub
set-face global ts_attribute                    default
set-face global ts_comment                      default
set-face global ts_comment_block                ts_comment
set-face global ts_comment_line                 ts_comment
set-face global ts_conceal                      default
set-face global ts_constant                     default
set-face global ts_constant_builtin_boolean     ts_constant
set-face global ts_constant_character           ts_constant
set-face global ts_constant_character_escape    ts_constant_character
set-face global ts_constant_macro               ts_constant
set-face global ts_constant_numeric             ts_constant_macro
set-face global ts_constant_numeric_float       ts_constant_numeric
set-face global ts_constant_numeric_integer     ts_constant_numeric
set-face global ts_constructor                  default
set-face global ts_diff_plus                    default
set-face global ts_diff_minus                   default
set-face global ts_diff_delta                   default
set-face global ts_diff_delta_moved             ts_diff_delta
set-face global ts_error                        default
set-face global ts_function                     default
set-face global ts_function_builtin             ts_function
set-face global ts_function_macro               ts_function
set-face global ts_function_method              ts_function
set-face global ts_function_special             ts_function
set-face global ts_hint                         default
set-face global ts_info                         default
set-face global ts_keyword                      default
set-face global ts_keyword_control              ts_keyword
set-face global ts_keyword_conditional          ts_keyword
set-face global ts_keyword_control_conditional  ts_keyword
set-face global ts_keyword_control_directive    ts_keyword
set-face global ts_keyword_control_import       ts_keyword
set-face global ts_keyword_control_repeat       ts_keyword
set-face global ts_keyword_control_return       ts_keyword
set-face global ts_keyword_control_except       ts_keyword
set-face global ts_keyword_control_exception    ts_keyword
set-face global ts_keyword_directive            ts_keyword
set-face global ts_keyword_function             ts_keyword
set-face global ts_keyword_operator             ts_keyword
set-face global ts_keyword_special              ts_keyword
set-face global ts_keyword_storage              ts_keyword
set-face global ts_keyword_storage_modifier     ts_keyword_storage
set-face global ts_keyword_storage_modifier_mut ts_keyword_storage_modifier
set-face global ts_keyword_storage_modifier_ref ts_keyword_storage_modifier
set-face global ts_keyword_storage_type         ts_keyword_storage
set-face global ts_label                        default
set-face global ts_markup_bold                  default
set-face global ts_markup_heading               default
set-face global ts_markup_heading_1             ts_markup_heading
set-face global ts_markup_heading_2             ts_markup_heading
set-face global ts_markup_heading_3             ts_markup_heading
set-face global ts_markup_heading_4             ts_markup_heading
set-face global ts_markup_heading_5             ts_markup_heading
set-face global ts_markup_heading_6             ts_markup_heading
set-face global ts_markup_heading_marker        ts_markup_heading
set-face global ts_markup_italic                default
set-face global ts_markup_list                  default
set-face global ts_markup_list_checked          ts_markup_list
set-face global ts_markup_list_numbered         ts_markup_list
set-face global ts_markup_list_unchecked        ts_markup_list
set-face global ts_markup_list_unnumbered       ts_markup_list
set-face global ts_markup_link                  default
set-face global ts_markup_link_label            ts_markup_link
set-face global ts_markup_link_url              ts_markup_link
set-face global ts_markup_link_uri              ts_markup_link
set-face global ts_markup_link_text             ts_markup_link
set-face global ts_markup_quote                 default
set-face global ts_markup_raw                   default
set-face global ts_markup_raw_block             ts_markup_raw
set-face global ts_markup_raw_inline            ts_markup_raw
set-face global ts_markup_strikethrough         default
set-face global ts_namespace                    default
set-face global ts_operator                     default
set-face global ts_property                     default
set-face global ts_punctuation                  default
set-face global ts_punctuation_bracket          ts_punctuation
set-face global ts_punctuation_delimiter        ts_punctuation
set-face global ts_punctuation_special          ts_punctuation
set-face global ts_special                      default
set-face global ts_spell                        default
set-face global ts_string                       default
set-face global ts_string_regex                 ts_string
set-face global ts_string_regexp                ts_string
set-face global ts_string_escape                ts_string
set-face global ts_string_special               ts_string
set-face global ts_string_special_path          ts_string_special
set-face global ts_string_special_symbol        ts_string_special
set-face global ts_string_symbol                ts_string
set-face global ts_tag                          default
set-face global ts_tag_error                    ts_tag
set-face global ts_text                         default
set-face global ts_text_title                   ts_text
set-face global ts_type                         default
set-face global ts_type_builtin                 ts_type
set-face global ts_type_enum_variant            ts_type
set-face global ts_variable                     default
set-face global ts_variable_builtin             ts_variable
set-face global ts_variable_other_member        ts_variable
set-face global ts_variable_parameter           ts_variable
set-face global ts_warning                      default
