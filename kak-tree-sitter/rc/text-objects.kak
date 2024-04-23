declare-user-mode tree-sitter
declare-user-mode tree-sitter-search
declare-user-mode tree-sitter-search-rev
declare-user-mode tree-sitter-search-extend
declare-user-mode tree-sitter-search-extend-rev
declare-user-mode tree-sitter-find
declare-user-mode tree-sitter-find-rev
declare-user-mode tree-sitter-find-extend
declare-user-mode tree-sitter-find-extend-rev

map global tree-sitter /     ':enter-user-mode tree-sitter-search<ret>'            -docstring 'search next'
map global tree-sitter <a-/> ':enter-user-mode tree-sitter-search-rev<ret>'        -docstring 'search prev'
map global tree-sitter ?     ':enter-user-mode tree-sitter-search-extend<ret>'     -docstring 'search(extend) next'
map global tree-sitter <a-?> ':enter-user-mode tree-sitter-search-extend-rev<ret>' -docstring 'search(extend) prev'
map global tree-sitter f     ':enter-user-mode tree-sitter-find<ret>'              -docstring 'find next'
map global tree-sitter <a-f> ':enter-user-mode tree-sitter-find-rev<ret>'          -docstring 'find prev'
map global tree-sitter F     ':enter-user-mode tree-sitter-find-extend<ret>'       -docstring 'find(extend) next'
map global tree-sitter <a-F> ':enter-user-mode tree-sitter-find-extend-rev<ret>'   -docstring 'find(extend) prev'
map global tree-sitter s     ":kak-tree-sitter-req-nav '""parent""'<ret>"          -docstring 'select parent'
map global tree-sitter t     ":kak-tree-sitter-req-nav '""first_child""'<ret>"     -docstring 'select first child'
map global tree-sitter c     ":kak-tree-sitter-req-nav '{ ""prev_sibling"": { ""cousin"": false } }'<ret>" -docstring 'select previous sibling'
map global tree-sitter r     ":kak-tree-sitter-req-nav '{ ""next_sibling"": { ""cousin"": false } }'<ret>" -docstring 'select next sibling'
map global tree-sitter C     ":kak-tree-sitter-req-nav '{ ""prev_sibling"": { ""cousin"": true } }'<ret>"  -docstring 'select previous sibling (cousin)'
map global tree-sitter R     ":kak-tree-sitter-req-nav '{ ""next_sibling"": { ""cousin"": true } }'<ret>"  -docstring 'select next sibling (cousin)'

map global tree-sitter-search f ':kak-tree-sitter-req-text-objects function.around search_next<ret>'  -docstring 'function'
map global tree-sitter-search a ':kak-tree-sitter-req-text-objects parameter.around search_next<ret>' -docstring 'parameter'
map global tree-sitter-search t ':kak-tree-sitter-req-text-objects class.around search_next<ret>' -docstring 'class'
map global tree-sitter-search c ':kak-tree-sitter-req-text-objects comment.around search_next<ret>' -docstring 'comment'
map global tree-sitter-search T ':kak-tree-sitter-req-text-objects test.around search_next<ret>' -docstring 'test'

map global tree-sitter-search-rev f ':kak-tree-sitter-req-text-objects function.around search_prev<ret>'  -docstring 'function'
map global tree-sitter-search-rev a ':kak-tree-sitter-req-text-objects parameter.around search_prev<ret>' -docstring 'parameter'
map global tree-sitter-search-rev t ':kak-tree-sitter-req-text-objects class.around search_prev<ret>' -docstring 'class'
map global tree-sitter-search-rev T ':kak-tree-sitter-req-text-objects test.around search_prev<ret>' -docstring 'test'

map global tree-sitter-search-extend f ':kak-tree-sitter-req-text-objects function.around search_extend_next<ret>'  -docstring 'function'
map global tree-sitter-search-extend a ':kak-tree-sitter-req-text-objects parameter.around search_extend_next<ret>' -docstring 'parameter'
map global tree-sitter-search-extend t ':kak-tree-sitter-req-text-objects class.around search_extend_next<ret>' -docstring 'class'
map global tree-sitter-search-extend T ':kak-tree-sitter-req-text-objects test.around search_extend_next<ret>' -docstring 'test'

map global tree-sitter-search-extend-rev f ':kak-tree-sitter-req-text-objects function.around search_extend_prev<ret>'  -docstring 'function'
map global tree-sitter-search-extend-rev a ':kak-tree-sitter-req-text-objects parameter.around search_extend_prev<ret>' -docstring 'parameter'
map global tree-sitter-search-extend-rev t ':kak-tree-sitter-req-text-objects class.around search_extend_prev<ret>' -docstring 'class'
map global tree-sitter-search-extend-rev T ':kak-tree-sitter-req-text-objects test.around search_extend_prev<ret>' -docstring 'test'

map global tree-sitter-find f ':kak-tree-sitter-req-text-objects function.around find_next<ret>'  -docstring 'function'
map global tree-sitter-find a ':kak-tree-sitter-req-text-objects parameter.around find_next<ret>' -docstring 'parameter'
map global tree-sitter-find t ':kak-tree-sitter-req-text-objects class.around find_next<ret>' -docstring 'class'
map global tree-sitter-find T ':kak-tree-sitter-req-text-objects test.around find_next<ret>' -docstring 'test'

map global tree-sitter-find-rev f ':kak-tree-sitter-req-text-objects function.around find_prev<ret>'  -docstring 'function'
map global tree-sitter-find-rev a ':kak-tree-sitter-req-text-objects parameter.around find_prev<ret>' -docstring 'parameter'
map global tree-sitter-find-rev t ':kak-tree-sitter-req-text-objects class.around find_prev<ret>' -docstring 'class'
map global tree-sitter-find-rev T ':kak-tree-sitter-req-text-objects test.around find_prev<ret>' -docstring 'test'

map global tree-sitter-find-extend f ':kak-tree-sitter-req-text-objects function.around extend_next<ret>'  -docstring 'function'
map global tree-sitter-find-extend a ':kak-tree-sitter-req-text-objects parameter.around extend_next<ret>' -docstring 'parameter'
map global tree-sitter-find-extend t ':kak-tree-sitter-req-text-objects class.around extend_next<ret>' -docstring 'class'
map global tree-sitter-find-extend T ':kak-tree-sitter-req-text-objects test.around extend_next<ret>' -docstring 'test'

map global tree-sitter-find-extend-rev f ':kak-tree-sitter-req-text-objects function.around extend_prev<ret>'  -docstring 'function'
map global tree-sitter-find-extend-rev a ':kak-tree-sitter-req-text-objects parameter.around extend_prev<ret>' -docstring 'parameter'
map global tree-sitter-find-extend-rev t ':kak-tree-sitter-req-text-objects class.around extend_prev<ret>' -docstring 'class'
map global tree-sitter-find-extend-rev T ':kak-tree-sitter-req-text-objects test.around extend_prev<ret>' -docstring 'test'

map global object f '<a-;>kak-tree-sitter-req-object-text-objects function<ret>' -docstring 'function (tree-sitter)'
map global object t '<a-;>kak-tree-sitter-req-object-text-objects class<ret>' -docstring 'type (tree-sitter)'
map global object a '<a-;>kak-tree-sitter-req-object-text-objects parameter<ret>' -docstring 'argument (tree-sitter)'
map global object T '<a-;>kak-tree-sitter-req-object-text-objects test<ret>' -docstring 'test (tree-sitter)'
