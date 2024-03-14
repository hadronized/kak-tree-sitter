; ecma
; Special identifiers
;--------------------

([
    (identifier)
    (shorthand_property_identifier)
    (shorthand_property_identifier_pattern)
 ] @constant
 (#match? @constant "^[A-Z_][A-Z\\d_]+$"))


((identifier) @constructor
 (#match? @constructor "^[A-Z]"))

((identifier) @variable.builtin
 (#match? @variable.builtin "^(arguments|module|console|window|document)$")
 (#is-not? local))

((identifier) @function.builtin
 (#eq? @function.builtin "require")
 (#is-not? local))

; Function and method definitions
;--------------------------------

(function
  name: (identifier) @function)
(function_declaration
  name: (identifier) @function)
(method_definition
  name: (property_identifier) @function.method)

(pair
  key: (property_identifier) @function.method
  value: [(function) (arrow_function)])

(assignment_expression
  left: (member_expression
    property: (property_identifier) @function.method)
  right: [(function) (arrow_function)])

(variable_declarator
  name: (identifier) @function
  value: [(function) (arrow_function)])

(assignment_expression
  left: (identifier) @function
  right: [(function) (arrow_function)])

; Function and method parameters
;-------------------------------

; Arrow function parameters in the form `p => ...` are supported by both
; javascript and typescript grammars without conflicts.
(arrow_function
  parameter: (identifier) @variable.parameter)

; Function and method calls
;--------------------------

(call_expression
  function: (identifier) @function)

(call_expression
  function: (member_expression
    property: (property_identifier) @function.method))

; Variables
;----------

(identifier) @variable

; Properties
;-----------

(property_identifier) @variable.other.member
(shorthand_property_identifier) @variable.other.member
(shorthand_property_identifier_pattern) @variable.other.member

; Literals
;---------

(this) @variable.builtin
(super) @variable.builtin

[
  (true)
  (false)
  (null)
  (undefined)
] @constant.builtin

(comment) @comment

[
  (string)
  (template_string)
] @string

(regex) @string.regexp
(number) @constant.numeric.integer

; Tokens
;-------

(template_substitution
  "${" @punctuation.special
  "}" @punctuation.special) @embedded

[
  ";"
  (optional_chain) ; ?.
  "."
  ","
] @punctuation.delimiter

[
  "-"
  "--"
  "-="
  "+"
  "++"
  "+="
  "*"
  "*="
  "**"
  "**="
  "/"
  "/="
  "%"
  "%="
  "<"
  "<="
  "<<"
  "<<="
  "="
  "=="
  "==="
  "!"
  "!="
  "!=="
  "=>"
  ">"
  ">="
  ">>"
  ">>="
  ">>>"
  ">>>="
  "~"
  "^"
  "&"
  "|"
  "^="
  "&="
  "|="
  "&&"
  "||"
  "??"
  "&&="
  "||="
  "??="
  "..."
] @operator

(ternary_expression ["?" ":"] @operator)

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
]  @punctuation.bracket

[
  "async"
  "debugger"
  "delete"
  "extends"
  "from"
  "get"
  "new"
  "set"
  "target"
  "typeof"
  "instanceof"
  "void"
  "with"
] @keyword

[
  "of"
  "as"
  "in"
] @keyword.operator

[
  "function"
] @keyword.function

[
  "class"
  "let"
  "var"
] @keyword.storage.type

[
  "const"
  "static"
] @keyword.storage.modifier

[
  "default"
  "yield"
  "finally"
  "do"
  "await"
] @keyword.control

[
  "if"
  "else"
  "switch"
  "case"
  "while"
] @keyword.control.conditional

[
  "for"
] @keyword.control.repeat

[
  "import"
  "export"
] @keyword.control.import

[
  "return"
  "break"
  "continue"
] @keyword.control.return

[
  "throw"
  "try"
  "catch"
] @keyword.control.exception

; typescript
; Namespaces
; ----------

(internal_module
  [((identifier) @namespace) ((nested_identifier (identifier) @namespace))])

(ambient_declaration "global" @namespace)

; Parameters
; ----------
; Javascript and Typescript Treesitter grammars deviate when defining the
; tree structure for parameters, so we need to address them in each specific
; language instead of ecma.

; (p: t)
; (p: t = 1)
(required_parameter
  (identifier) @variable.parameter)

; (...p: t)
(required_parameter
  (rest_pattern
    (identifier) @variable.parameter))

; ({ p }: { p: t })
(required_parameter
  (object_pattern
    (shorthand_property_identifier_pattern) @variable.parameter))

; ({ a: p }: { a: t })
(required_parameter
  (object_pattern
    (pair_pattern
      value: (identifier) @variable.parameter)))

; ([ p ]: t[])
(required_parameter
  (array_pattern
    (identifier) @variable.parameter))

; (p?: t)
; (p?: t = 1) // Invalid but still posible to hihglight.
(optional_parameter
  (identifier) @variable.parameter)

; (...p?: t) // Invalid but still posible to hihglight.
(optional_parameter
  (rest_pattern
    (identifier) @variable.parameter))

; ({ p }: { p?: t})
(optional_parameter
  (object_pattern
    (shorthand_property_identifier_pattern) @variable.parameter))

; ({ a: p }: { a?: t })
(optional_parameter
  (object_pattern
    (pair_pattern
      value: (identifier) @variable.parameter)))

; ([ p ]?: t[]) // Invalid but still posible to hihglight.
(optional_parameter
  (array_pattern
    (identifier) @variable.parameter))

; Punctuation
; -----------

[
  ":"
] @punctuation.delimiter

(optional_parameter "?" @punctuation.special)
(property_signature "?" @punctuation.special)

(conditional_type ["?" ":"] @operator)

; Keywords
; --------

[
  "abstract"
  "declare"
  "export"
  "infer"
  "implements"
  "keyof"
  "namespace"
  "override"
  "satisfies"
] @keyword

[
  "type"
  "interface"
  "enum"
] @keyword.storage.type

[
  "public"
  "private"
  "protected"
  "readonly"
] @keyword.storage.modifier

; Types
; -----

(type_identifier) @type
(predefined_type) @type.builtin

; Type arguments and parameters
; -----------------------------

(type_arguments
  [
    "<"
    ">"
  ] @punctuation.bracket)

(type_parameters
  [
    "<"
    ">"
  ] @punctuation.bracket)

; Literals
; --------

[
  (template_literal_type)
] @string

; Tokens
; ------

(template_type
  "${" @punctuation.special
  "}" @punctuation.special) @embedded
