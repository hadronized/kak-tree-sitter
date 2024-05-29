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
  %sh{
    kak-tree-sitter -vvvvv -kr "{ \"session\": \"$kak_session\", \"payload\": { \"type\": \"$1\" } }"
  }
}

# Create a command to send to Kakoune for the current session and client.
#
# The parameter is the string to be used as payload.
define-command -hidden tree-sitter-request-with-session-client -params 1 %{
  %sh{
    kak-tree-sitter -vvvvv -kr "{ \"session\": \"$kak_session\", \"client\": \"$kak_client\", \"payload\": $1 }"
  }
}

# Create a command to send to Kakoune for the current session and buffer.
#
# The parameter is the string to be used as payload.
define-command -hidden tree-sitter-request-with-session-buffer -params 1 %{
  %sh{
    kak-tree-sitter -vvvvv -kr "{ \"session\": \"$kak_session\", \"buffer\": \"$kak_bufname\", \"payload\": $1 }"
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
  tree-sitter-request-with-session-client "{ ""type"": ""text_objects"", ""buffer"": ""%val{bufname}"", ""pattern"": ""%arg{1}"", ""selections"": ""%val{selections_desc}"", ""mode"": { ""object"": { ""mode"": ""%val{select_mode}"", ""flags"": ""%val{object_flags}"" }}"
}

# Request KTS to navigate the tree-sitter tree on selections.
#
# The first parameter is the direction to move to.
define-command tree-sitter-nav -params 1 %{
  tree-sitter-request-with-session-client "{ ""type"": ""nav"", ""buffer"": ""%val{bufname}"", ""selections"": ""%val{selections_desc}"", ""dir"": ""%arg{1}"" }}"
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

  # Hook that removes the default highlighter 

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

  hook -group tree-sitter-update buffer NormalIdle .* %{ tree-sitter-exec-if-changed tree-sitter-buffer-update }
  hook -group tree-sitter-update buffer InsertIdle .* %{ tree-sitter-exec-if-changed tree-sitter-buffer-update }
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

    # == EVERYTHING AFTER THAT IS LEGACY ==

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
  add-highlighter -override buffer/kak-tree-sitter-highlighter ranges tree_sitter_hl_ranges

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
#hook -group kak-tree-sitter global -once ClientCreate .* %{
#  kak-tree-sitter-req-init
#
#  # Make kak-tree-sitter know the session has ended whenever we end it.
#  hook -group kak-tree-sitter global KakEnd .* kak-tree-sitter-req-end-session
#}

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
