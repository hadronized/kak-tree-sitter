; ecma
; Scopes
;-------

[
  (statement_block)
  (function)
  (arrow_function)
  (function_declaration)
  (method_definition)
] @local.scope

; Definitions
;------------

; ...i
(rest_pattern
  (identifier) @local.definition)

; { i }
(object_pattern
  (shorthand_property_identifier_pattern) @local.definition)

; { a: i }
(object_pattern
  (pair_pattern
    value: (identifier) @local.definition))

; [ i ]
(array_pattern
  (identifier) @local.definition)

; i => ...
(arrow_function
  parameter: (identifier) @local.definition)

; const/let/var i = ...
(variable_declarator
  name: (identifier) @local.definition)

; References
;------------

(identifier) @local.reference

; jsx

; typescript
; Definitions
;------------

; Javascript and Typescript Treesitter grammars deviate when defining the
; tree structure for parameters, so we need to address them in each specific
; language instead of ecma.

; (i: t)
; (i: t = 1)
(required_parameter
  (identifier) @local.definition)

; (i?: t)
; (i?: t = 1) // Invalid but still posible to hihglight.
(optional_parameter
  (identifier) @local.definition)