#  This file should be sourced only once by session. It is not recommended to
# source it yourself; instead, when starting the KTS server, the binary will
# inject it directly into the session.

# FIFO buffer path; this is used by Kakoune to write the content of buffers to
# update the tree-sitter representation on KTS side.
#
# Should only be set KTS side by buffer.
declare-option str tree_sitter_buf_fifo_path /dev/null

# Sentinel code used to delimit buffers in FIFOs.
declare-option str tree_sitter_buf_sentinel

# Highlight ranges used when highlighting buffers.
declare-option range-specs tree_sitter_hl_ranges

# Internal verbosity; used when sending requests to KTS.
declare-option str tree_sitter_verbose '-vvvvv'

# Language a buffer uses. That option should be set at the buffer level.
declare-option str tree_sitter_lang

# Last known timestamp of previouses buffer updates.
declare-option int tree_sitter_buf_update_timestamp -1

# Create a command to send to Kakoune for the current session.
#
# The parameter is the string to be used as payload.
define-command -hidden tree-sitter-request-with-session -params 1 %{
  evaluate-commands -no-hooks %sh{
    kak-tree-sitter -vvv -kr "{ \"session\": \"$kak_session\", \"payload\": { \"type\": \"$1\" } }"
  }
}

# Create a command to send to Kakoune for the current session and client.
#
# The parameter is the string to be used as payload.
define-command -hidden tree-sitter-request-with-session-client -params 1 %{
  evaluate-commands -no-hooks %sh{
    kak-tree-sitter -vvv -kr "{ \"session\": \"$kak_session\", \"client\": \"$kak_client\", \"payload\": $1 }"
  }
}

# Create a command to send to Kakoune for the current session and buffer.
#
# The parameter is the string to be used as payload.
define-command -hidden tree-sitter-request-with-session-buffer -params 1 %{
  evaluate-commands -no-hooks %sh{
    kak-tree-sitter -vvv -kr "{ \"session\": \"$kak_session\", \"buffer\": \"$kak_bufname\", \"payload\": $1 }"
  }
}

# Notify KTS that a session exists.
define-command tree-sitter-session-begin %{
  tree-sitter-request-with-session 'session_begin'
}

# Notify KTS that the session is exiting.
define-command tree-sitter-session-end %{
  tree-sitter-request-with-session 'session_end'
  tree-sitter-remove-all
}

# Request KTS to reload its configuration (grammar, queries, etc.).
define-command tree-sitter-reload %{
  tree-sitter-request-with-session 'reload'
  tree-sitter-session-end
  tree-sitter-session-begin
}

# Request KTS to completely shutdown.
define-command tree-sitter-shutdown %{
  tree-sitter-request-with-session 'shutdown'
}

# Request KTS to update its metadata regarding a buffer.
define-command tree-sitter-buffer-metadata %{
  tree-sitter-request-with-session-buffer "{ ""type"": ""buffer_metadata"", ""lang"": ""%opt{tree_sitter_lang}"" }"
}

# Request KTS to update its buffer representation of the current buffer.
#
# The parameter is the language the buffer is formatted in.
define-command tree-sitter-buffer-update %{
  evaluate-commands -no-hooks %{
    write "%opt{tree_sitter_buf_fifo_path}"
    echo -to-file "%opt{tree_sitter_buf_fifo_path}" -- "%opt{tree_sitter_buf_sentinel}"
  }
}

# Request KTS to clean up resources of a closed buffer.
define-command tree-sitter-buffer-close %{
  tree-sitter-request-with-session-buffer "{ ""type"": ""buffer_close"" }"
}

# Request KTS to apply text-objects on selections.
#
# First parameter is the pattern.
# Second parameter is the operation mode.
define-command tree-sitter-text-objects -params 2 %{
  tree-sitter-request-with-session-client "{ ""type"": ""text_objects"", ""buffer"": ""%val{bufname}"", ""pattern"": ""%arg{1}"", ""selections"": ""%val{selections_desc}"", ""mode"": ""%arg{2}"" }"
}

# Request KTS to apply “object-mode” text-objects on selections.
#
# First parameter is the pattern.
define-command tree-sitter-object-text-objects -params 1 %{
  tree-sitter-request-with-session-client "{ ""type"": ""text_objects"", ""buffer"": ""%val{bufname}"", ""pattern"": ""%arg{1}"", ""selections"": ""%val{selections_desc}"", ""mode"": { ""object"": { ""mode"": ""%val{select_mode}"", ""flags"": ""%val{object_flags}"" } } }"
}

# Request KTS to navigate the tree-sitter tree on selections.
#
# The first parameter is the direction to move to.
define-command tree-sitter-nav -params 1 %{
  tree-sitter-request-with-session-client "{ ""type"": ""nav"", ""buffer"": ""%val{bufname}"", ""selections"": ""%val{selections_desc}"", ""dir"": ""%arg{1}"" }"
}

# Install main hook for a given language.
#
# That hook reacts to setting the tree_sitter_lang option to enable
# tree-sitter support.
#
# The first parameter is the language.
# The second parameter is a boolean stating whether we should remove the default
# highlighter.
define-command -hidden tree-sitter-hook-install-lang -params 2 %{
  hook -group tree-sitter global BufSetOption "tree_sitter_lang=%arg{1}" %{
    tree-sitter-buffer-metadata
    add-highlighter -override buffer/tree-sitter-highlighter ranges tree_sitter_hl_ranges
  }
}

# Install main hooks.
define-command -hidden tree-sitter-hook-install-session %{
  # Hook that runs when the session ends.
  hook -group tree-sitter global KakEnd .* %{
    tree-sitter-session-end
  }

  # HACK: this is temporary; only used to ensure %opt{tree_sitter_lang} works
  # as expected; in the end, users should do that on their own
  hook -group tree-sitter global BufSetOption filetype=(.*) %{
    # Forward the filetype as tree-sitter language.
    set-option buffer tree_sitter_lang "%opt{filetype}"
  }
}

# Install a hook that updates buffer content if it has changed.
define-command -hidden tree-sitter-hook-install-update %{
  # Since this hook can be installed several times (after each changes of the
  # tree_sitter_lang option; see tree-sitter-hook-install-main), it’s better
  # to first try to remove the hooks.
  remove-hooks buffer tree-sitter-update

  # Buffer update
  hook -group tree-sitter-update buffer NormalIdle .* %{ tree-sitter-exec-if-changed tree-sitter-buffer-update }
  hook -group tree-sitter-update buffer InsertIdle .* %{ tree-sitter-exec-if-changed tree-sitter-buffer-update }

  # Initial highlight
  tree-sitter-buffer-update

  # Buffer close
  hook -group tree-sitter-update buffer BufClose .* %{ tree-sitter-buffer-close }
}

# A helper function that executes its argument only if the buffer has changed.
define-command -hidden tree-sitter-exec-if-changed -params 1 %{
  set-option -remove buffer tree_sitter_buf_update_timestamp %val{timestamp}

  try %{
    evaluate-commands "tree-sitter-exec-nop-%opt{tree_sitter_buf_update_timestamp}"
    set-option buffer tree_sitter_buf_update_timestamp %val{timestamp}
  } catch %{
    # Actually run the command
    set-option buffer tree_sitter_buf_update_timestamp %val{timestamp}
    evaluate-commands %arg{1}
  }
}

# A helper function that does nothing.
#
# Used with tree-sitter-exec-if-changed to have a fallback when the buffer has
# not changed.
define-command -hidden tree-sitter-exec-nop-0 nop

# Remove every tree-sitter commands, hooks, options, etc.
define-command tree-sitter-remove-all %{
  remove-hooks global tree-sitter

  evaluate-commands -buffer * %{
    try %{
      remove-highlighter buffer/tree-sitter-highlighter
    }

    try %{
      remove-hooks buffer tree-sitter-update
    }
  }
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
