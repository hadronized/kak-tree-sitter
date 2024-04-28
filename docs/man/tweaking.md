# Tweaking

If you are using the [provided config.toml](https://git.sr.ht/~hadronized/kak-tree-sitter/tree/master/item/kak-tree-sitter-config/default-config.toml),
some languages might require more setup. They are listed in this section.

- [JSX](#jsx]

## JSX

JSX grammars and queries are a bit weird, as they are not part of a specific language (i.e. `kts_lang`), but can work
with Javascript, Typescript, etc. For this reason, extra setup is required. This page explains how to setup your
configuration.

> Note: this grammar / query set is incompatible with `javascript`. If you plan on using both Javascript and JSX, you
> should only install the `jsx` grammar, as it also can work with regular Javascript files. See below for the setup.

- [Fetch the grammar and queries](#fetch-the-grammar-and-queries)
- [Patch the grammar](#fetch-the-grammar)
- [Compile and install](#compile-and-install)
- [Useful hook](#useful-hook)

### Fetch the grammar and queries

```bash
ktsctl -f jsx
```

### Patch the grammar

This is the tricky part. You need to replace all `javascript` occurrences with `jsx`. Go in
`$XDG_DATA_DIR/ktsctl/grammars/javascript` (or `$TMPDIR/ktsctl/grammars/javascript` on macOS if you do not have the XDG
environment variables), and run this:

```bash
grep -rl 'javascript' . | xargs sed -i '' -e 's/javascript/jsx/g'
```

### Compile and install

Simply compile and install normally as you would do with `ktsctl`:

```bash
ktsctl -ci jsx
```

### Useful hook

If you still want to work with regular Javascript files, then you need to tell Kakoune to interpret them as if they were
using the JSX grammars. You can do it with a simple hook translating `kts_lang=javascript` to `kts_lang=jsx`, such as:

```bash
hook global BufSetOption kts_lang=(javascript|typescript) %{
  eval %sh{
    case $kak_bufname in
      (*\.jsx) echo "set-option buffer kts_lang jsx";;
      (*\.tsx) echo "set-option buffer kts_lang tsx";;
    esac
  }
}
```
