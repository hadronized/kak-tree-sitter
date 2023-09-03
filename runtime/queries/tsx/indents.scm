; ecma
[
  (array)
  (object)
  (arguments)
  (formal_parameters)

  (statement_block)
  (switch_statement)
  (object_pattern)
  (class_body)
  (named_imports)

  (binary_expression)
  (return_statement)
  (template_substitution)
  (export_clause)
] @indent

[
  (switch_case)
  (switch_default)
] @indent @extend

[
  "}"
  "]"
  ")"
] @outdent

; jsx
[
  (jsx_element)
  (jsx_self_closing_element)
] @indent

(parenthesized_expression) @indent

; typescript
[
  (enum_declaration) 
  (interface_declaration)
  (object_type)
] @indent