# This file should be read only once. Either place it in your autoload/, or use the more practical --kakoune option when
# invoking kak-tree-sitter.

define-command -override kak-tree-sitter-stop -docstring 'Ask the daemon to shutdown' %{
  nop %sh{
    kak-tree-sitter -s $kak_session -r '{"type":"shutdown"}'
  }
}

define-command -override kak-tree-sitter-enable -docstring 'Enable tree-sitter highlighting for this buffer' %{
  # Hooks
  hook -group kak-tree-sitter buffer InsertIdle   .* kak-tree-sitter-highlight-buffer
  hook -group kak-tree-sitter buffer InsertChar   .* kak-tree-sitter-highlight-buffer
  hook -group kak-tree-sitter buffer InsertDelete .* kak-tree-sitter-highlight-buffer
  hook -group kak-tree-sitter buffer BufReload    .* kak-tree-sitter-highlight-buffer

  # remove regular highlighting, if any
  try %{
    remove-highlighter window/%opt{filetype}
  }

  # trigger the first highlight
  kak-tree-sitter-highlight-buffer
}

define-command -hidden -override kak-tree-sitter-highlight-buffer -docstring 'Highlight the current buffer' %{
  declare-option str stream_name %sh{tr '/' '-' <<< "$kak_timestamp-$kak_bufname"}
  evaluate-commands -draft -no-hooks write -force "%opt{kak_tree_sitter_stream_dir}/%opt{stream_name}"
  nop %sh{
    kak-tree-sitter -s $kak_session -c $kak_client -r "{\"type\":\"highlight\",\"buffer_id\":{\"session\":\"$kak_session\",\"buffer\":\"$kak_bufname\"},\"lang\":\"$kak_opt_filetype\",\"timestamp\":$kak_timestamp,\"stream_name\":\"$kak_opt_stream_name\"}"
  }
}

# Faces definition
#set-face global ts_unknown                red+ub
set-face global ts_attribute              red
set-face global ts_comment                'rgb:7f849c+i'
set-face global ts_constant               'rgb:f5a97f'
set-face global ts_constructor            'rgb:eed49f'
set-face global ts_function_builtin       red
set-face global ts_function               'rgb:8aadf4'
set-face global ts_function_macro         'rgb:8aadf4+b'
set-face global ts_function_method        'rgb:8aadf4'
set-face global ts_keyword                'rgb:c6a0f6'
set-face global ts_keyword_control_conditional 'rgb:cba6f7+i'
set-face global ts_keyword_function       'rgb:c6a0f6'
set-face global ts_label                  'rgb:7dc4e4'
set-face global ts_namespace              'rgb:89b4fa+i'
set-face global ts_operator               'rgb:8bd5ca'
set-face global ts_property               attribute
set-face global ts_punctuation            'rgb:939ab7'
set-face global ts_punctuation_bracket    'rgb:939ab7'
set-face global ts_punctuation_delimiter  'rgb:939ab7'
set-face global ts_special                'rgb:89b4fa'
set-face global ts_string                 'rgb:a6da95'
set-face global ts_string_special         meta
set-face global ts_tag                    builtin
set-face global ts_type                   'rgb:eed49f'
set-face global ts_type_builtin           'rgb:eed49f'
set-face global ts_variable               'rgb:cdd6f4'
set-face global ts_variable_builtin       red
set-face global ts_variable_other_member  'rgb:94e2d5'
set-face global ts_variable_parameter     'rgb:ee99a0+i'
