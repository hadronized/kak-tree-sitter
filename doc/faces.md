# Faces used by `kak-tree-sitter` highlight requests

This document provides a list of common highlight faces used by `kak-tree-sitter` when highlighting a buffer. However,
due to the nature of tree-sitter queries, it is possible that a specific query adds a capture group that is not known to
us. In that case, it is suggested to open a PR to add it here / to the `rc` file.

> However, please note that every one can in theory create any kind of capture group they want and thus, we will only
> accept capture groups that make sense in terms of sharing and language support. We will not support a specific
> capture group for a subset of users, but we will accept anything that makes sense for a specific language, even if the
> capture group is only defined for that language.

- `ts_unknown` _(note: you should ignore this face and only define it to debug)_
- `ts_attribute`
- `ts_comment`
- `ts_constant`
- `ts_constructor`
- `ts_function_builtin`
- `ts_function`
- `ts_function_macro`
- `ts_function_method`
- `ts_keyword`
- `ts_keyword_control_conditional`
- `ts_keyword_function`
- `ts_label`
- `ts_namespace`
- `ts_operator`
- `ts_property`
- `ts_punctuation`
- `ts_punctuation_bracket`
- `ts_punctuation_delimiter`
- `ts_special`
- `ts_string`
- `ts_string_special`
- `ts_tag`
- `ts_type`
- `ts_type_builtin`
- `ts_variable`
- `ts_variable_builtin`
- `ts_variable_other_member`
- `ts_variable_parameter`
