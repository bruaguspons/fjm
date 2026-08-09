---
"fjm": minor
---

Add `--lts`/`--latest` to `fjm default`, `fjm uninstall`, and `fjm exec --using`, so every command that resolves a version against what's already installed (not just `fjm install`/`fjm use`, which can also download) supports the same shorthand.

```sh-session
$ fjm default --lts
$ fjm uninstall --latest
$ fjm exec --lts -- java --version
```

`fjm alias` intentionally keeps its current two-positional-argument shape (`fjm alias <TO_VERSION> <NAME>`) — making `TO_VERSION` optional to fit `--lts`/`--latest` would make it ambiguous with the required `NAME` positional that follows it. Use the existing `lts-latest` alias string there instead (e.g. `fjm alias lts-latest mylabel`).

Internally, the `--lts`/`--latest` resolution logic (previously duplicated between `install` and `use`) is now centralized in `lts_latest_selector::resolve`.
