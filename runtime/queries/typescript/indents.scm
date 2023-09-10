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

; typescript
; Parse the contents of tagged template literals using
; a language inferred from the tag.

(call_expression
  function: [
    (identifier) @injection.language
    (member_expression
      property: (property_identifier) @injection.language)
  ]
  arguments: (template_string) @injection.content)

; Parse the contents of gql template literals

((call_expression
   function: (identifier) @_template_function_name
   arguments: (template_string) @injection.content)
 (#eq? @_template_function_name "gql")
 (#set! injection.language "graphql"))

; Parse regex syntax within regex literals

((regex_pattern) @injection.content
 (#set! injection.language "regex"))

; Parse JSDoc annotations in multiline comments

((comment) @injection.content
 (#set! injection.language "jsdoc")
 (#match? @injection.content "^/\\*+"))

; Parse general tags in single line comments

((comment) @injection.content
 (#set! injection.language "comment")
 (#match? @injection.content "^//"))