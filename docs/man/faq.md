# Frequently Asked Questions

## I don’t want to deal with options; give me a oneliner to install a language

```sh
ktsctl -fci yaml
```

## How do I install all the languages at once?

This is currently, unfortunately, not supported. You will have to run `ktsctl -fci` for all the languages you want.
See [#30](https://github.com/hadronized/kak-tree-sitter/issues/30).

## Something broke and there is no highlighting anymore

You can have a look at the log files in `$XDG_RUNTIME_DIR/kak-tree-sitter/{stdout.txt,stderr.txt}` and open an issue.
If the server crashed, you can simply restart a server; it will automatically recollect all the live Kakoune sessions
and should work again.
