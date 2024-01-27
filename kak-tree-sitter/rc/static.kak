# This file should be sourced only once by session. It is not recommended to source it yourself; instead, the KTS server
# will inject it via the kak’s UNIX socket when ready to accept commands for the session.

# Options used by KTS at runtime.

# FIFO command path; this is used by Kakoune to write commands to be executed by KTS for the current session.
declare-option str kts_cmd_fifo_path /dev/null

# FIFO buffer path; this is used by Kakoune to write the content of buffers to be highlighted / analyzed by KTS for the
# current session. 
declare-option str kts_buf_fifo_path /dev/null

# The timestamp of when the buffer was most recently highlighted by kts
declare-option -hidden int kts_highlight_timestamp -1

# Highlight ranges used when highlighting buffers.
declare-option range-specs kts_highlighter_ranges

# Tree-sitter language to use to highlight buffers’ content.
#
# This option will expand its content.
declare-option str kts_lang

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

# Part of the trick below to avoid a shell to check a value
define-command -hidden kak-tree-sitter--nop-exists-if-zero-0 nop

# If the buffer has changed since this function was last called with the same option as first parameter,
# it will evaluate the commands given as second parameter
# Usage:
# declare-option int kts_highlight_timestamp -1
# kak-tree-sitter-if-changed-since kts_highlight_timestamp %{
#   echo "Changed!"
# }
define-command -hidden kak-tree-sitter-if-changed-since -params 2 %{
  # this subtracts the timestamp from the current value in the option
  set -remove buffer %arg{1} %val{timestamp}
  try %{
    # This only succeeds if it finds the above function, i.e. if the value of the option in %arg{1} is now 0
    eval "eval ""kak-tree-sitter--nop-exists-if-zero-%%opt{%arg{1}}"""
    set buffer %arg{1} %val{timestamp}
  } catch %{
    set buffer %arg{1} %val{timestamp}
    eval %arg{2}
  }
}

# Send a single request to highlight the current buffer.
#
# This will first send the command to highlight the buffer to KTS and then will write the content of the buffer through
# the same FIFO.
define-command kak-tree-sitter-req-highlight-buffer -docstring 'Highlight the current buffer' %{
  kak-tree-sitter-if-changed-since kts_highlight_timestamp %{
    evaluate-commands -no-hooks %{
      echo -to-file %opt{kts_cmd_fifo_path} -- "{ ""type"": ""highlight"", ""client"": ""%val{client}"", ""buffer"": ""%val{bufname}"", ""lang"": ""%opt{kts_lang}"", ""timestamp"": %val{timestamp} }"
      write %opt{kts_buf_fifo_path}
      set-option buffer kts_highlight_timestamp %val{timestamp}
    }
  }
}

define-command kak-tree-sitter-req-textobjects -params 1 %{
  evaluate-commands -no-hooks %{
    echo -to-file %opt{kts_cmd_fifo_path} -- "{ ""type"": ""text_objects"", ""client"": ""%val{client}"", ""buffer"": ""%val{bufname}"", ""lang"": ""%opt{kts_lang}"", ""timestamp"": %val{timestamp}, ""selections"": ""%val{selections_desc}"", ""textobject_type"": ""%arg{1}"", ""object_flags"": ""%val{object_flags}"", ""select_mode"": ""%val{select_mode}"" }"
    write %opt{kts_buf_fifo_path}
  }
}

define-command kak-tree-sitter-req-select -params 1 %{
  evaluate-commands -no-hooks %{
    echo -to-file %opt{kts_cmd_fifo_path} -- "{ ""type"": ""select_text_objects"", ""client"": ""%val{client}"", ""buffer"": ""%val{bufname}"", ""lang"": ""%opt{kts_lang}"", ""timestamp"": %val{timestamp}, ""selections"": ""%val{selections_desc}"", ""textobject_type"": ""%arg{1}"" }"
    write %opt{kts_buf_fifo_path}
  }
}

declare-user-mode kak-tree-sitter

# Enable textobject mappings
define-command -hidden kak-tree-sitter-textobjects-enable %{
  map -docstring 'function (tree-sitter)'         buffer object f   '<a-;> kak-tree-sitter-req-textobjects function<ret>'
  map -docstring 'class (tree-sitter)'            buffer object t   '<a-;> kak-tree-sitter-req-textobjects class<ret>'
  map -docstring 'parameter (tree-sitter)'        buffer object v '<a-;> kak-tree-sitter-req-textobjects parameter<ret>'
  map -docstring 'comment (tree-sitter)'          buffer object '#' '<a-;> kak-tree-sitter-req-textobjects comment<ret>'
  
  map -docstring 'narrow selection to functions'  buffer kak-tree-sitter f   ':kak-tree-sitter-req-select function<ret>'
  map -docstring 'narrow selection to classs'     buffer kak-tree-sitter t   ':kak-tree-sitter-req-select class<ret>'
  map -docstring 'narrow selection to parameters' buffer kak-tree-sitter v   ':kak-tree-sitter-req-select parameter<ret>'
  map -docstring 'narrow selection to comments'   buffer kak-tree-sitter '#' ':kak-tree-sitter-req-select comment<ret>'
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
  kak-tree-sitter-text-objects-enable
}

# Initiate request.
#
# This is used to ask the server to tell us where to write commands and other various data.
define-command -hidden kak-tree-sitter-req-init %{
  nop %sh{
    kak-tree-sitter -r "{ \"type\": \"register_session\", \"name\": \"$kak_session\", \"client\": \"$kak_client\" }"
  }
}

# Wait to have a client and then ask the server to initiate.
hook -group kak-tree-sitter global -once ClientCreate .* %{
  kak-tree-sitter-req-init

  # Enable tree-sitter once we open a new window.
  hook -group kak-tree-sitter global WinCreate .* %{
    hook -group kak-tree-sitter buffer -once WinDisplay .* kak-tree-sitter-req-enable
  }

  # Make kak-tree-sitter know the session has ended whenever we end it.
  hook -group kak-tree-sitter global KakEnd .* kak-tree-sitter-req-end-session
}

#set-face global ts_unknown                     red+ub
set-face global ts_attribute                    "%opt{kts_blue}"
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
set-face global ts_markup_raw                   "%opt{kts_green}"
set-face global ts_markup_raw_block             "%opt{kts_green}"
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
set-face global ts_tag                          "%opt{kts_mauve}"
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
