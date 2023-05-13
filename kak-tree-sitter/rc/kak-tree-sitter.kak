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

# Highlight ranges used when highlighting buffers.
declare-option range-specs kak_tree_sitter_highlighter_ranges

# Faces definition; defaults to catppuccin_macchiato
declare-option str rosewater 'rgb:f4dbd6'
declare-option str flamingo 'rgb:f0c6c6'
declare-option str pink 'rgb:f5bde6'
declare-option str mauve 'rgb:c6a0f6'
declare-option str red 'rgb:ed8796'
declare-option str maroon 'rgb:ee99a0'
declare-option str peach 'rgb:f5a97f'
declare-option str yellow 'rgb:eed49f'
declare-option str green 'rgb:a6da95'
declare-option str teal 'rgb:8bd5ca'
declare-option str sky 'rgb:91d7e3'
declare-option str sapphire 'rgb:7dc4e4'
declare-option str blue 'rgb:8aadf4'
declare-option str lavender 'rgb:b7bdf8'
declare-option str text 'rgb:cad3f5'
declare-option str subtext1 'rgb:b8c0e0'
declare-option str subtext0 'rgb:a5adcb'
declare-option str overlay2 'rgb:939ab7'
declare-option str overlay1 'rgb:8087a2'
declare-option str overlay0 'rgb:6e738d'
declare-option str surface2 'rgb:5b6078'
declare-option str surface1 'rgb:494d64'
declare-option str surface0 'rgb:363a4f'

#set-face global ts_unknown                    red+ub
set-face global ts_attribute                   "%opt{red}"
set-face global ts_comment                     "%opt{overlay1}+i"
set-face global ts_conceal                     "%opt{mauve}+i"
set-face global ts_constant                    "%opt{peach}"
set-face global ts_constructor                 "%opt{sapphire}"
set-face global ts_function                    "%opt{blue}"
set-face global ts_function_builtin            "%opt{blue}"
set-face global ts_function_macro              "%opt{blue}+b"
set-face global ts_function_method             "%opt{blue}"
set-face global ts_keyword                     "%opt{mauve}"
set-face global ts_keyword_control_conditional "%opt{mauve}+i"
set-face global ts_keyword_function            "%opt{mauve}"
set-face global ts_label                       "%opt{sapphire}"
set-face global ts_namespace                   "%opt{blue}+i"
set-face global ts_operator                    "%opt{sky}"
set-face global ts_property                    "%opt{sky}"
set-face global ts_punctuation                 "%opt{overlay2}"
set-face global ts_punctuation_bracket         "%opt{overlay2}"
set-face global ts_punctuation_delimiter       "%opt{overlay2}"
set-face global ts_punctuation_special         "%opt{sky}"
set-face global ts_special                     "%opt{blue}"
set-face global ts_spell                       "%opt{red}+ub"
set-face global ts_string                      "%opt{green}"
set-face global ts_string_regexp               "%opt{peach}"
set-face global ts_string_escape               "%opt{mauve}"
set-face global ts_string_special              "%opt{blue}"
set-face global ts_tag                         "%opt{mauve}"
set-face global ts_text                        "%opt{text}"
set-face global ts_text_literal                "%opt{rosewater}"
set-face global ts_text_reference              "%opt{teal}"
set-face global ts_text_title                  "%opt{mauve}"
set-face global ts_text_quote                  "%opt{red}+ub"
set-face global ts_text_uri                    "%opt{maroon}+u"
set-face global ts_type                        "%opt{yellow}"
set-face global ts_type_builtin                "%opt{yellow}"
set-face global ts_variable                    "%opt{text}"
set-face global ts_variable_builtin            "%opt{red}"
set-face global ts_variable_other_member       "%opt{teal}"
set-face global ts_variable_parameter          "%opt{maroon}+i"
