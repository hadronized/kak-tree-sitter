# This file should be read only once. Either place it in your autoload/, or use the more practical --kakoune option when
# invoking kak-tree-sitter.

# Mark the session as non-active.
#
# This is typically sent when a session is about to die; see KakEnd for further details.
define-command -override kak-tree-sitter-end-session -docstring 'Mark the session as ended' %{
  nop %sh{
    kak-tree-sitter -s $kak_session -r '{"type":"session_end"}'
  }
}

# Stop the kak-tree-sitter daemon.
#
# To restart the daemon, the daemon must explicitly be recreated with %sh{kak-tree-sitter -d -s $kak_session}.
define-command -override kak-tree-sitter-stop -docstring 'Ask the daemon to shutdown' %{
  evaluate-commands -no-hooks -buffer * %{
    remove-hooks buffer kak-tree-sitter
  }

  remove-hooks global kak-tree-sitter

  nop %sh{
    kak-tree-sitter -s $kak_session -r '{"type":"shutdown"}'
  }
}

# Enabling highlighting for the current buffer.
# 
# This command does a couple of things, among removing the « default » highlighting (Kakoune based) of the buffer and
# installing some hooks to automatically highlight the buffer.
define-command -override kak-tree-sitter-highlight-enable -docstring 'Enable tree-sitter highlighting for this buffer' %{
  # remove regular highlighting, if any; we wrap this with try %{} because the highlighter might not even exist or is
  # named differently; in such a case we should probably have a mapping or something
  try %{
    remove-highlighter "window/%opt{filetype}"
  }

  hook -group kak-tree-sitter buffer InsertIdle .* kak-tree-sitter-highlight-buffer
  hook -group kak-tree-sitter buffer NormalIdle .* kak-tree-sitter-highlight-buffer
}

# Send a single request to highlight the current buffer.
define-command -override kak-tree-sitter-highlight-buffer -docstring 'Highlight the current buffer' %{
  nop %sh{
    echo "evaluate-commands -no-hooks -verbatim write $kak_response_fifo" > $kak_command_fifo
    kak-tree-sitter -s $kak_session -c $kak_client -r "{\"type\":\"highlight\",\"buffer\":\"$kak_bufname\",\"lang\":\"$kak_opt_filetype\",\"timestamp\":$kak_timestamp,\"read_fifo\":\"$kak_response_fifo\"}"
  }
}

# Enable automatic tree-sitter highlights.
hook -group kak-tree-sitter global WinCreate .* %{
  hook -group kak-tree-sitter buffer -once WinDisplay .* %{
    # Check whether this filetype is supported
    nop %sh{
      kak-tree-sitter -s "$kak_session" -c "$kak_client" -r "{\"type\":\"try_enable_highlight\",\"lang\":\"$kak_opt_filetype\"}"
    }
  }
}

# Make kak-tree-sitter know the session has ended whenever we end it.
hook -group kak-tree-sitter global KakEnd .* kak-tree-sitter-end-session

# Faces definition
#set-face global ts_unknown                    red+ub
set-face global ts_attribute                   red
set-face global ts_comment                     'rgb:7f849c+i'
set-face global ts_conceal                     'bright-magenta+i'
set-face global ts_constant                    'rgb:f5a97f'
set-face global ts_constructor                 'rgb:eed49f'
set-face global ts_function_builtin            red
set-face global ts_function                    'rgb:8aadf4'
set-face global ts_function_macro              'rgb:8aadf4+b'
set-face global ts_function_method             'rgb:8aadf4'
set-face global ts_keyword                     'rgb:c6a0f6'
set-face global ts_keyword_control_conditional 'rgb:cba6f7+i'
set-face global ts_keyword_function            'rgb:c6a0f6'
set-face global ts_label                       'rgb:7dc4e4'
set-face global ts_namespace                   'rgb:89b4fa+i'
set-face global ts_operator                    'rgb:8bd5ca'
set-face global ts_property                    attribute
set-face global ts_punctuation                 'rgb:939ab7'
set-face global ts_punctuation_bracket         'rgb:939ab7'
set-face global ts_punctuation_delimiter       'rgb:939ab7'
set-face global ts_punctuation_special         blue
set-face global ts_special                     'rgb:89b4fa'
set-face global ts_spell                       bright-green+i
set-face global ts_string                      'rgb:a6da95'
set-face global ts_string_escape               magenta
set-face global ts_string_special              yellow
set-face global ts_tag                         builtin
set-face global ts_text                        blue
set-face global ts_text_literal                green
set-face global ts_text_reference              magenta+i
set-face global ts_text_title                  red
set-face global ts_text_quote                  yellow+b
set-face global ts_text_uri                    cyan
set-face global ts_type                        'rgb:eed49f'
set-face global ts_type_builtin                'rgb:eed49f'
set-face global ts_variable                    'rgb:cdd6f4'
set-face global ts_variable_builtin            red
set-face global ts_variable_other_member       'rgb:94e2d5'
set-face global ts_variable_parameter          'rgb:ee99a0+i'
