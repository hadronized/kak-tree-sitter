# Frequently Asked Questions

## I don’t want to deal with options; give me a oneliner to install a language

```sh
ktsctl manage -sl yaml
```

## How do I install all the languages at once?

```sh
ktsctl manage -sa
```

## Something broke and there is no highlighting anymore

You can have a look at the log files in
`$XDG_RUNTIME_DIR/kak-tree-sitter/{stdout.txt,stderr.txt}` and open an issue.
