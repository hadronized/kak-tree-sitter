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
