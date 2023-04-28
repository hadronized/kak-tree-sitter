# This file should be read only once. Either place it in your autoload/, or use the more practical --kakoune option when
# invoking kak-tree-sitter.

define-command -override kak-tree-sitter-stop -docstring 'Ask the daemon to shutdown' %{
  nop %sh{
    kak-tree-sitter -s $kak_session -r '{"type":"shutdown"}'
  }
}

define-command -override kak-tree-sitter-highlight-request -docstring 'Highlight request' %{
  nop %sh{
    kak-tree-sitter -s $kak_session -c $kak_client -r "{\"type\":\"highlight\",\"buffer_id\":{\"session\":\"$kak_session\",\"buffer\":\"$kak_bufname\"},\"lang\":\"$kak_opt_filetype\",\"path\":\"$kak_buffile\"}"
  }
}

# Faces definition
 set-face global ts_unknown                red+ub
 set-face global ts_attribute              attribute
 set-face global ts_constant               red
 set-face global ts_constructor            function
 set-face global ts_function_builtin       function
 set-face global ts_function               function
 set-face global ts_function_macro         yellow
 set-face global ts_function_method        magenta
 set-face global ts_keyword                keyword
 set-face global ts_label                  blue
 set-face global ts_operator               link
 set-face global ts_property               attribute
 set-face global ts_punctuation            link
 set-face global ts_punctuation_bracket    link
 set-face global ts_punctuation_delimiter  link
 set-face global ts_string                 string
 set-face global ts_string_special         meta
 set-face global ts_tag                    builtin
 set-face global ts_type                   type
 set-face global ts_type_builtin           type
 set-face global ts_variable               variable
 set-face global ts_variable_builtin       variable
 set-face global ts_variable_parameter     variable
