declare-user-mode tree-sitter
declare-user-mode tree-sitter-search
declare-user-mode tree-sitter-search-rev
declare-user-mode tree-sitter-search-extend
declare-user-mode tree-sitter-search-extend-rev
declare-user-mode tree-sitter-find
declare-user-mode tree-sitter-find-rev
declare-user-mode tree-sitter-find-extend
declare-user-mode tree-sitter-find-extend-rev
declare-user-mode tree-sitter-select

map global tree-sitter /     ':enter-user-mode tree-sitter-search<ret>'                            -docstring 'search next'
map global tree-sitter <a-/> ':enter-user-mode tree-sitter-search-rev<ret>'                        -docstring 'search prev'
map global tree-sitter ?     ':enter-user-mode tree-sitter-search-extend<ret>'                     -docstring 'search(extend) next'
map global tree-sitter <a-?> ':enter-user-mode tree-sitter-search-extend-rev<ret>'                 -docstring 'search(extend) prev'
map global tree-sitter f     ':enter-user-mode tree-sitter-find<ret>'                              -docstring 'find next'
map global tree-sitter <a-f> ':enter-user-mode tree-sitter-find-rev<ret>'                          -docstring 'find prev'
map global tree-sitter F     ':enter-user-mode tree-sitter-find-extend<ret>'                       -docstring 'find(extend) next'
map global tree-sitter <a-F> ':enter-user-mode tree-sitter-find-extend-rev<ret>'                   -docstring 'find(extend) prev'
map global tree-sitter k     ':enter-user-mode tree-sitter-select<ret>'                            -docstring 'select'
map global tree-sitter s     ":tree-sitter-nav '""parent""'<ret>"                                  -docstring 'select parent'
map global tree-sitter t     ":tree-sitter-nav '""first_child""'<ret>"                             -docstring 'select first child'
map global tree-sitter <c-t> ":tree-sitter-nav '""last_child""'<ret>"                              -docstring 'select last child'
map global tree-sitter c     ":tree-sitter-nav '{ ""prev_sibling"": { ""cousin"": false } }'<ret>" -docstring 'select previous sibling'
map global tree-sitter r     ":tree-sitter-nav '{ ""next_sibling"": { ""cousin"": false } }'<ret>" -docstring 'select next sibling'
map global tree-sitter C     ":tree-sitter-nav '{ ""prev_sibling"": { ""cousin"": true } }'<ret>"  -docstring 'select previous sibling (cousin)'
map global tree-sitter R     ":tree-sitter-nav '{ ""next_sibling"": { ""cousin"": true } }'<ret>"  -docstring 'select next sibling (cousin)'
map global tree-sitter (     ":tree-sitter-nav '""first_sibling""'<ret>"                           -docstring 'select first sibling'
map global tree-sitter )     ":tree-sitter-nav '""last_sibling""'<ret>"                            -docstring 'select last sibling'
map global tree-sitter T     ':enter-user-mode tree-sitter-nav-sticky<ret>'                        -docstring 'sticky tree navigation'

map global tree-sitter-search f ':tree-sitter-text-objects function.around search_next<ret>'  -docstring 'function'
map global tree-sitter-search a ':tree-sitter-text-objects parameter.around search_next<ret>' -docstring 'parameter'
map global tree-sitter-search t ':tree-sitter-text-objects class.around search_next<ret>'     -docstring 'class'
map global tree-sitter-search c ':tree-sitter-text-objects comment.around search_next<ret>'   -docstring 'comment'
map global tree-sitter-search T ':tree-sitter-text-objects test.around search_next<ret>'      -docstring 'test'

map global tree-sitter-search-rev f ':tree-sitter-text-objects function.around search_prev<ret>'  -docstring 'function'
map global tree-sitter-search-rev a ':tree-sitter-text-objects parameter.around search_prev<ret>' -docstring 'parameter'
map global tree-sitter-search-rev t ':tree-sitter-text-objects class.around search_prev<ret>'     -docstring 'class'
map global tree-sitter-search-rev T ':tree-sitter-text-objects test.around search_prev<ret>'      -docstring 'test'

map global tree-sitter-search-extend f ':tree-sitter-text-objects function.around search_extend_next<ret>'  -docstring 'function'
map global tree-sitter-search-extend a ':tree-sitter-text-objects parameter.around search_extend_next<ret>' -docstring 'parameter'
map global tree-sitter-search-extend t ':tree-sitter-text-objects class.around search_extend_next<ret>'     -docstring 'class'
map global tree-sitter-search-extend T ':tree-sitter-text-objects test.around search_extend_next<ret>'      -docstring 'test'

map global tree-sitter-search-extend-rev f ':tree-sitter-text-objects function.around search_extend_prev<ret>'  -docstring 'function'
map global tree-sitter-search-extend-rev a ':tree-sitter-text-objects parameter.around search_extend_prev<ret>' -docstring 'parameter'
map global tree-sitter-search-extend-rev t ':tree-sitter-text-objects class.around search_extend_prev<ret>'     -docstring 'class'
map global tree-sitter-search-extend-rev T ':tree-sitter-text-objects test.around search_extend_prev<ret>'      -docstring 'test'

map global tree-sitter-find f ':tree-sitter-text-objects function.around find_next<ret>'  -docstring 'function'
map global tree-sitter-find a ':tree-sitter-text-objects parameter.around find_next<ret>' -docstring 'parameter'
map global tree-sitter-find t ':tree-sitter-text-objects class.around find_next<ret>'     -docstring 'class'
map global tree-sitter-find T ':tree-sitter-text-objects test.around find_next<ret>'      -docstring 'test'

map global tree-sitter-find-rev f ':tree-sitter-text-objects function.around find_prev<ret>'  -docstring 'function'
map global tree-sitter-find-rev a ':tree-sitter-text-objects parameter.around find_prev<ret>' -docstring 'parameter'
map global tree-sitter-find-rev t ':tree-sitter-text-objects class.around find_prev<ret>'     -docstring 'class'
map global tree-sitter-find-rev T ':tree-sitter-text-objects test.around find_prev<ret>'      -docstring 'test'

map global tree-sitter-find-extend f ':tree-sitter-text-objects function.around extend_next<ret>'  -docstring 'function'
map global tree-sitter-find-extend a ':tree-sitter-text-objects parameter.around extend_next<ret>' -docstring 'parameter'
map global tree-sitter-find-extend t ':tree-sitter-text-objects class.around extend_next<ret>'     -docstring 'class'
map global tree-sitter-find-extend T ':tree-sitter-text-objects test.around extend_next<ret>'      -docstring 'test'

map global tree-sitter-find-extend-rev f ':tree-sitter-text-objects function.around extend_prev<ret>'  -docstring 'function'
map global tree-sitter-find-extend-rev a ':tree-sitter-text-objects parameter.around extend_prev<ret>' -docstring 'parameter'
map global tree-sitter-find-extend-rev t ':tree-sitter-text-objects class.around extend_prev<ret>'     -docstring 'class'
map global tree-sitter-find-extend-rev T ':tree-sitter-text-objects test.around extend_prev<ret>'      -docstring 'test'

map global tree-sitter-select f ':tree-sitter-text-objects function.around select<ret>'  -docstring 'function'
map global tree-sitter-select a ':tree-sitter-text-objects parameter.around select<ret>' -docstring 'parameter'
map global tree-sitter-select t ':tree-sitter-text-objects class.around select<ret>'     -docstring 'class'
map global tree-sitter-select T ':tree-sitter-text-objects test.around select<ret>'      -docstring 'test'

map global object f '<a-;>tree-sitter-object-text-objects function<ret>'  -docstring 'function (tree-sitter)'
map global object t '<a-;>tree-sitter-object-text-objects class<ret>'     -docstring 'type (tree-sitter)'
map global object a '<a-;>tree-sitter-object-text-objects parameter<ret>' -docstring 'argument (tree-sitter)'
map global object T '<a-;>tree-sitter-object-text-objects test<ret>'      -docstring 'test (tree-sitter)'

# sticky mode for navigation
declare-user-mode tree-sitter-nav-sticky

define-command -hidden tree-sitter-nav-sticky-undo %{
  execute-keys "<a-u>"
  enter-user-mode tree-sitter-nav-sticky
}

define-command -hidden tree-sitter-nav-sticky-parent %{
  tree-sitter-nav '"parent"'
  enter-user-mode tree-sitter-nav-sticky
}

define-command -hidden tree-sitter-nav-sticky-first-child %{
  tree-sitter-nav '"first_child"'
  enter-user-mode tree-sitter-nav-sticky
}

define-command -hidden tree-sitter-nav-sticky-last-child %{
  tree-sitter-nav '"last_child"'
  enter-user-mode tree-sitter-nav-sticky
}

define-command -hidden tree-sitter-nav-sticky-prev-sibling -params 1 %{
  tree-sitter-nav "{ ""prev_sibling"": { ""cousin"": %arg{1} }}"
  enter-user-mode tree-sitter-nav-sticky
}

define-command -hidden tree-sitter-nav-sticky-next-sibling -params 1 %{
  tree-sitter-nav "{ ""next_sibling"": { ""cousin"": %arg{1} }}"
  enter-user-mode tree-sitter-nav-sticky
}

define-command -hidden tree-sitter-nav-sticky-first-sibling %{
  tree-sitter-nav '"first_sibling"'
  enter-user-mode tree-sitter-nav-sticky
}

define-command -hidden tree-sitter-nav-sticky-last-sibling %{
  tree-sitter-nav '"last_sibling"'
  enter-user-mode tree-sitter-nav-sticky
}

map global tree-sitter-nav-sticky s     ':tree-sitter-nav-sticky-parent<ret>'             -docstring 'select parent'
map global tree-sitter-nav-sticky t     ':tree-sitter-nav-sticky-first-child<ret>'        -docstring 'select first child'
map global tree-sitter-nav-sticky <c-t> ':tree-sitter-nav-sticky-first-child<ret>'        -docstring 'select last child'
map global tree-sitter-nav-sticky C     ':tree-sitter-nav-sticky-prev-sibling true<ret>'  -docstring 'select previous sibling (cousin)'
map global tree-sitter-nav-sticky R     ':tree-sitter-nav-sticky-next-sibling true<ret>'  -docstring 'select next sibling (cousin)'
map global tree-sitter-nav-sticky c     ':tree-sitter-nav-sticky-prev-sibling false<ret>' -docstring 'select previous sibling'
map global tree-sitter-nav-sticky r     ':tree-sitter-nav-sticky-next-sibling false<ret>' -docstring 'select next sibling'
map global tree-sitter-nav-sticky (     ':tree-sitter-nav-sticky-first-sibling<ret>'      -docstring 'select first sibling'
map global tree-sitter-nav-sticky )     ':tree-sitter-nav-sticky-last-sibling<ret>'       -docstring 'select last sibling'
map global tree-sitter-nav-sticky u     ':tree-sitter-nav-sticky-undo<ret>'               -docstring 'undo selection'
