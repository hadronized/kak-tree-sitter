; kak-tree-sitter notes: taken from helix-editor/helix

; From nvim-treesitter/nvim-treesitter

(fenced_code_block
  (code_fence_content) @injection.shebang @injection.content
  (#set! injection.include-children))

(fenced_code_block
  (info_string
    (language) @injection.language)
  (code_fence_content) @injection.content (#set! injection.include-children))

((html_block) @injection.content
 (#set! injection.language "html")
 (#set! injection.include-children)
 (#set! injection.combined))

((pipe_table_cell) @injection.content (#set! injection.language "markdown.inline") (#set! injection.include-children))

((minus_metadata) @injection.content (#set! injection.language "yaml") (#set! injection.include-children))
((plus_metadata) @injection.content (#set! injection.language "toml") (#set! injection.include-children))

((inline) @injection.content (#set! injection.language "markdown.inline") (#set! injection.include-children))
