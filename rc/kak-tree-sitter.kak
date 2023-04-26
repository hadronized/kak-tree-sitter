# This file should be read only once. Either place it in your autoload/, or use the more practical --kakoune option when
# invoking kak-tree-sitter.

define-command -override kak-tree-sitter-highlight-request -docstring 'Send a JSON-formatted request to kak-tree-sitter' %{
  nop %sh{
    kak-tree-sitter -s $kak_session -c $kak_client -r "{\"type\":\"highlight\",\"session_name\":\"$kak_session\",\"buffer_name\":\"$kak_bufname\",\"lang\":\"$kak_opt_filetype\",\"path\":\"$kak_buffile\"}"
  }
}
